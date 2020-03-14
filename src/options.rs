pub struct Options {
    pub hosts: Vec<String>,
    pub timeout: Option<u64>,
    pub command: Option<Vec<String>>,
}

enum ParseState {
    Host,
    Timeout,
    Command,
}

pub fn parse(args: Vec<String>) -> Result<Options, Option<String>> {
    let mut options = Options {
        hosts: vec![],
        timeout: None,
        command: None,
    };

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
                "---" => {
                    state = ParseState::Command;
                }
                _ => {
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
