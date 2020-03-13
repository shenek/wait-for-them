pub struct Options {
    pub hosts: Vec<String>,
    pub timeout: Option<u64>,
}

pub fn parse_options(args: Vec<String>) -> Result<Options, ()> {
    let mut options = Options {
        hosts: vec![],
        timeout: None,
    };
    let mut read_timeout: bool = false;
    for arg in args {
        if read_timeout {
            options.timeout = Some(arg.parse().map_err(|_| ())?);
            read_timeout = false;
            continue;
        }
        if arg == "-t" || arg == "--timeout" {
            read_timeout = true;
            continue;
        }
        options.hosts.push(arg);
    }
    Ok(options)
}
