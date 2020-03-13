pub struct Options {
    pub hosts: Vec<String>,
    pub timeout: Option<u64>,
}

pub fn parse_options(args: Vec<String>) -> Result<Options, Option<String>> {
    let mut options = Options {
        hosts: vec![],
        timeout: None,
    };
    let mut read_timeout: bool = false;
    for arg in args {
        if read_timeout {
            options.timeout = Some(
                arg.parse()
                    .map_err(|_| Some("Failed to parse timeout".to_string()))?,
            );
            read_timeout = false;
            continue;
        }
        if arg == "-t" || arg == "--timeout" {
            read_timeout = true;
            continue;
        }
        options.hosts.push(arg);
    }
    if options.hosts.is_empty() {
        Err(Some(
            "You need to set at least one host and port".to_string(),
        ))
    } else {
        Ok(options)
    }
}
