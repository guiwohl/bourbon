use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const APP_BINARY: &str = "bourbon-app";

pub fn launch_app() -> io::Result<()> {
    let executable = std::env::current_exe().ok();
    let directory = executable.as_deref().and_then(Path::parent);
    spawn_detached(&program_in(directory)).map(drop)
}

fn program_in(directory: Option<&Path>) -> PathBuf {
    directory
        .map(|directory| directory.join(APP_BINARY))
        .filter(|candidate| candidate.is_file())
        .unwrap_or_else(|| PathBuf::from(APP_BINARY))
}

fn spawn_detached(program: &Path) -> io::Result<Child> {
    Command::new(program)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn fake_app(directory: &Path, script: &str) -> PathBuf {
        let path = directory.join(APP_BINARY);
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[test]
    fn the_app_next_to_the_executable_is_preferred() {
        let directory = tempfile::tempdir().unwrap();
        let app = fake_app(directory.path(), "#!/bin/sh\n");

        assert_eq!(program_in(Some(directory.path())), app);
    }

    #[test]
    fn without_a_sibling_the_app_is_looked_up_on_path() {
        let directory = tempfile::tempdir().unwrap();

        assert_eq!(
            program_in(Some(directory.path())),
            PathBuf::from(APP_BINARY)
        );
        assert_eq!(program_in(None), PathBuf::from(APP_BINARY));
    }

    #[test]
    fn the_app_runs_with_its_output_discarded() {
        let directory = tempfile::tempdir().unwrap();
        let marker = directory.path().join("ran");
        let app = fake_app(
            directory.path(),
            &format!(
                "#!/bin/sh\necho out\necho err >&2\ntouch '{}'\n",
                marker.display()
            ),
        );

        let status = spawn_detached(&app).unwrap().wait().unwrap();

        assert!(status.success());
        assert!(marker.exists());
    }

    #[test]
    fn a_missing_app_is_an_error() {
        let directory = tempfile::tempdir().unwrap();

        let error = spawn_detached(&directory.path().join(APP_BINARY)).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }
}
