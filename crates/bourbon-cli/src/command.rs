#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    LaunchApp,
    Get(String),
    Help,
    Version,
    Unknown,
}

pub fn parse(arguments: &[String]) -> Command {
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    match arguments.as_slice() {
        [] => Command::LaunchApp,
        ["--help" | "-h"] => Command::Help,
        ["--version" | "-V"] => Command::Version,
        ["get", id] => Command::Get((*id).to_owned()),
        _ => Command::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(arguments: &[&str]) -> Command {
        let owned: Vec<String> = arguments
            .iter()
            .map(|argument| (*argument).to_owned())
            .collect();
        parse(&owned)
    }

    #[test]
    fn no_arguments_launch_the_app() {
        assert_eq!(parsed(&[]), Command::LaunchApp);
    }

    #[test]
    fn get_takes_exactly_one_id() {
        assert_eq!(parsed(&["get", "abc"]), Command::Get("abc".to_owned()));
        assert_eq!(parsed(&["get"]), Command::Unknown);
        assert_eq!(parsed(&["get", "a", "b"]), Command::Unknown);
    }

    #[test]
    fn help_and_version_have_long_and_short_forms() {
        assert_eq!(parsed(&["--help"]), Command::Help);
        assert_eq!(parsed(&["-h"]), Command::Help);
        assert_eq!(parsed(&["--version"]), Command::Version);
        assert_eq!(parsed(&["-V"]), Command::Version);
    }

    #[test]
    fn anything_else_is_unknown() {
        assert_eq!(parsed(&["list"]), Command::Unknown);
        assert_eq!(parsed(&["--help", "extra"]), Command::Unknown);
    }
}
