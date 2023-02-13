use regex::Regex;
use std::net::IpAddr;

static DOMAIN_REGEX: &str =
    r"^(([a-zA-Z_\-]{1,63}\.)*?)*?([a-zA-Z_\-]{1,63})(\.[a-zA-Z_\-]{1,63})?$";

#[derive(Default)]
pub struct Options {
    pub hosts: Vec<String>,
    pub timeout: Option<u64>,
    pub command: Option<Vec<String>>,
    pub silent: bool,
}

fn validate_domain_and_port(domain_and_port: &str) -> Result<(String, u16), String> {
    let parts: Vec<String> = domain_and_port.split(':').map(String::from).collect();
    if parts.len() != 2 {
        return Err(format!(
            "'{}' doesn't match <hostname>:<port> pattern",
            domain_and_port
        ));
    }

    // check port
    let port: u16 = parts[1]
        .parse()
        .map_err(|err| format!("'{}', port error: {}", domain_and_port, err))?;

    if port == 0 {
        return Err("dynamic port number (0) can't be used here".into());
    }

    // check hostname
    let hostname = parts[0].clone();
    let regex = Regex::new(DOMAIN_REGEX).unwrap();
    let ip: Result<IpAddr, _> = hostname.parse();

    if !regex.is_match(&hostname) && ip.is_err() {
        return Err(format!("'{}' is not a valid hostname", hostname));
    }
    Ok((hostname, port))
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
                    validate_domain_and_port(&arg)?;
                    options.hosts.push(arg);
                }
            },
        }
    }

    if options.hosts.is_empty() {
        Err(Some(
            "You need to set at least one host and port".to_string(),
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
}
