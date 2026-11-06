use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

const FOCUS_MESSAGE: &str = "focus";
const CLIENT_PATIENCE: Duration = Duration::from_secs(1);

pub(crate) enum Claim {
    AlreadyRunning,
    Primary(FocusListener),
}

pub(crate) struct FocusListener {
    listener: UnixListener,
}

impl FocusListener {
    pub(crate) fn into_focus_requests(self) -> Receiver<()> {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            for stream in self.listener.incoming().flatten() {
                if asks_for_focus(&stream) && sender.send(()).is_err() {
                    break;
                }
            }
        });
        receiver
    }
}

fn asks_for_focus(stream: &UnixStream) -> bool {
    if stream.set_read_timeout(Some(CLIENT_PATIENCE)).is_err() {
        return false;
    }
    let mut message = String::new();
    BufReader::new(stream).read_line(&mut message).is_ok() && message.trim_end() == FOCUS_MESSAGE
}

pub(crate) fn socket_path(app_id: &str, runtime_dir: Option<PathBuf>, user_id: u32) -> PathBuf {
    match runtime_dir {
        Some(dir) => dir.join(format!("{app_id}.sock")),
        None => PathBuf::from(format!("/tmp/{app_id}-{user_id}.sock")),
    }
}

pub(crate) fn claim(socket: &Path) -> io::Result<Claim> {
    match UnixStream::connect(socket) {
        Ok(mut stream) => {
            writeln!(stream, "{FOCUS_MESSAGE}")?;
            Ok(Claim::AlreadyRunning)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => bind(socket),
        Err(error) if error.kind() == ErrorKind::ConnectionRefused => {
            std::fs::remove_file(socket)?;
            bind(socket)
        }
        Err(error) => Err(error),
    }
}

fn bind(socket: &Path) -> io::Result<Claim> {
    Ok(Claim::Primary(FocusListener {
        listener: UnixListener::bind(socket)?,
    }))
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;
    use std::time::Duration;

    use super::{Claim, claim, socket_path};

    const WAIT: Duration = Duration::from_secs(5);
    const QUIET: Duration = Duration::from_millis(300);

    fn primary(socket: &std::path::Path) -> super::FocusListener {
        match claim(socket).unwrap() {
            Claim::Primary(listener) => listener,
            Claim::AlreadyRunning => panic!("expected to become the primary instance"),
        }
    }

    #[test]
    fn socket_lives_in_the_runtime_dir_named_after_the_app_id() {
        let path = socket_path("bourbon", Some(PathBuf::from("/run/user/1000")), 1000);
        assert_eq!(path, PathBuf::from("/run/user/1000/bourbon.sock"));
    }

    #[test]
    fn socket_falls_back_to_tmp_with_the_user_id() {
        let path = socket_path("bourbon-dev", None, 1000);
        assert_eq!(path, PathBuf::from("/tmp/bourbon-dev-1000.sock"));
    }

    #[test]
    fn first_instance_becomes_primary() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("a.sock");
        let _listener = primary(&socket);
        assert!(socket.exists());
    }

    #[test]
    fn second_instance_is_detected_and_asks_for_focus() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("a.sock");
        let requests = primary(&socket).into_focus_requests();
        assert!(matches!(claim(&socket).unwrap(), Claim::AlreadyRunning));
        assert!(requests.recv_timeout(WAIT).is_ok());
    }

    #[test]
    fn stale_socket_is_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("a.sock");
        drop(std::os::unix::net::UnixListener::bind(&socket).unwrap());
        assert!(socket.exists());
        let requests = primary(&socket).into_focus_requests();
        assert!(matches!(claim(&socket).unwrap(), Claim::AlreadyRunning));
        assert!(requests.recv_timeout(WAIT).is_ok());
    }

    #[test]
    fn unknown_messages_do_not_request_focus() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("a.sock");
        let requests = primary(&socket).into_focus_requests();
        let mut stream = UnixStream::connect(&socket).unwrap();
        stream.write_all(b"shout\n").unwrap();
        drop(stream);
        assert!(requests.recv_timeout(QUIET).is_err());
    }
}
