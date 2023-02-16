use wait_for_them::ToCheck;

#[derive(Default)]
pub struct Options {
    pub to_check: Vec<ToCheck>,
    pub timeout: Option<u64>,
    pub command: Option<Vec<String>>,
    pub silent: bool,
}

enum ParseState {
    Host,
    Timeout,
    Command,
}

pub fn parse(args: Vec<String>) -> Result<Options, Option<String>> {
    let mut options = Options::default();

    let mut state = ParseState::Host;

    // parse standard options
    for arg in args {
        match state {
            ParseState::Command => {
                let mut command = if let Some(command) = options.command.take() {
                    command
                } else {
                    vec![]
                };
                command.push(arg);
                options.command = Some(command);
            }
            ParseState::Timeout => {
                options.timeout = Some(
                    arg.parse()
                        .map_err(|_| Some("Failed to parse timeout".to_string()))?,
                );
                state = ParseState::Host;
            }
            ParseState::Host => match arg.as_ref() {
                "-t" | "--timeout" => state = ParseState::Timeout,
                "-s" | "--silent" => options.silent = true,
                "-h" | "--help" => return Err(None),
                "--" => {
                    state = ParseState::Command;
                }
                _ => {
                    options.to_check.push(arg.parse::<ToCheck>()?);
                }
            },
        }
    }

    if options.to_check.is_empty() {
        Err(Some(
            "You need to set at least one item to verify".to_string(),
        ))
    } else {
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn format() {
        assert!(parse(vec!["ok:33:888".into()]).is_err());
        assert!(parse(vec!["ok:aa:888".into()]).is_err());
        assert!(parse(vec!["www.example.com".into()]).is_err());
    }

    #[test]
    fn domains() {
        assert!(parse(vec!["www.example.com:22".into()]).is_ok());
        assert!(parse(vec!["ok:888".into(), "err/or:22".into()]).is_err());
        assert!(parse(vec!["ok:888".into(), "err or:22".into()]).is_err());
        assert!(parse(vec!["ok:888".into(), "[error]:22".into()]).is_err());
    }

    #[test]
    fn ports() {
        assert!(parse(vec!["last:65535".into()]).is_ok());
        assert!(parse(vec!["ok:888".into(), "error:-1".into()]).is_err());
        assert!(parse(vec!["ok:888".into(), "zero:0".into()]).is_err());
        assert!(parse(vec!["error:65536".into(), "ok:888".into()]).is_err());
    }

    #[test]
    fn timeout() {
        assert!(parse(vec!["-t".into(), "1".into(), "ok:888".into()]).is_ok());
        assert!(parse(vec!["-t".into(), "ok:888".into()]).is_err());
        assert!(parse(vec!["-t".into(), "-1".into(), "ok:888".into()]).is_err());
        assert!(parse(vec![
            "-t".into(),
            "18446744073709551615".into(),
            "ok:888".into()
        ])
        .is_ok());
        assert!(parse(vec![
            "-t".into(),
            "18446744073709551616".into(),
            "ok:888".into()
        ])
        .is_err());
    }

    #[test]
    fn silent() {
        let options = parse(vec!["www.example.com:888".into()]);
        assert!(!options.unwrap().silent);

        let options = parse(vec!["-s".into(), "www.example.com:888".into()]);
        assert!(options.unwrap().silent);
    }

    #[cfg(feature = "http")]
    #[test]
    fn uri() {
        assert!(parse(vec!["https://www.example.com".into()]).is_ok());
        assert!(parse(vec!["http://www.example.com".into()]).is_ok());
        assert!(parse(vec!["ftp://www.example.com".into()]).is_err());
        assert!(parse(vec!["https://www.example.com:11/long?x=1&y=2#frag".into()]).is_ok());
        assert!(parse(vec!["http://www.example.com:22/long?x=1&y=2#frag".into()]).is_ok());
    }
}
