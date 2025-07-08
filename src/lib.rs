//! Wait For Them library
//!
//! this library is used to asynchronously wait when
//! sockets or http(s) services become ready.
//!
//! # Example
//! ```no_run
//! use wait_for_them::{ToCheck, wait_for_them};
//!
//! #[tokio::main]
//! async fn main() {
//!     let res = wait_for_them(
//!         &[
//!             ToCheck::HostnameAndPort("localhost".into(), 8080),
//!             ToCheck::HttpOrHttpsUrl("https://example.com/".parse().unwrap()),
//!         ],
//!         Some(8000),  // 8 seconds
//!         None,  // time tracker
//!         true,  // silent
//!     ).await;
//! }
//! ```

mod scanner;

static DOMAIN_REGEX: &str =
    r"^(([a-zA-Z_\-]{1,63}\.)*?)*?([a-zA-Z_\-]{1,63})(\.[a-zA-Z_\-]{1,63})?$";

/// Wrapper around items which are going to be checked
///
/// it may be parsed from string
/// ```
/// let checks: Result<Vec<wait_for_them::ToCheck>, _> = [
///     "localhost:8000", "localhost:8080"
/// ].iter().map(|e| e.parse()).collect();
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum ToCheck {
    /// Hostname or IP address e.g. `127.0.0.1:8080` or `localhost:80`
    HostnameAndPort(String, u16),

    #[cfg(feature = "http")]
    #[allow(rustdoc::bare_urls)]
    /// Url with https or http `https://www.example.com:8080/some/?x=0&y=1#frag`
    HttpOrHttpsUrl(hyper::Uri),
}

impl std::fmt::Display for ToCheck {
    #[cfg(feature = "http")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HostnameAndPort(domain, port) => format!("{domain}:{port}").fmt(f),
            Self::HttpOrHttpsUrl(uri) => uri.fmt(f),
        }
    }

    #[cfg(not(feature = "http"))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HostnameAndPort(domain, port) => format!("{}:{}", domain, port).fmt(f),
        }
    }
}

impl ToCheck {
    fn from_host_and_port(domain_and_port: &str) -> Result<Self, String> {
        let parts: Vec<String> = domain_and_port.split(':').map(String::from).collect();
        if parts.len() != 2 {
            return Err(format!(
                "'{domain_and_port}' doesn't match <hostname>:<port> pattern"
            ));
        }

        // check port
        let port: u16 = parts[1]
            .parse()
            .map_err(|err| format!("'{domain_and_port}', port error: {err}"))?;

        if port == 0 {
            return Err("dynamic port number (0) can't be used here".into());
        }

        // check hostname
        let hostname = parts[0].clone();
        let regex = regex::Regex::new(DOMAIN_REGEX).unwrap();
        let ip: Result<std::net::IpAddr, _> = hostname.parse();

        if !regex.is_match(&hostname) && ip.is_err() {
            return Err(format!("'{hostname}' is not a valid hostname"));
        }
        Ok(Self::HostnameAndPort(hostname, port))
    }

    #[cfg(feature = "http")]
    fn from_http_url(http_url: &str) -> Result<Self, String> {
        Ok(Self::HttpOrHttpsUrl(
            http_url.parse::<hyper::Uri>().map_err(|e| e.to_string())?,
        ))
    }

    #[cfg(not(feature = "http"))]
    fn from_http_url(_uri: &str) -> Result<Self, String> {
        panic!("Not compiled with 'http' feature")
    }
}

impl std::str::FromStr for ToCheck {
    type Err = String;

    #[cfg(feature = "http")]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http://") || s.starts_with("https://") {
            Self::from_http_url(s)
        } else {
            Self::from_host_and_port(s)
        }
    }

    #[cfg(not(feature = "http"))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_host_and_port(s)
    }
}

/// Waits till all hostname and port combinations are opened
/// or until `200` status code is returned from http(s) URLs.
///
/// # Arguments
///
/// * `hosts_ports_or_http_urls` - items to be check
/// * `timeout` - `None` means that it it may wait forever or `Some(..)` set timeout in milis
/// * `start_time` - Optional time_tracker
/// * `silent` - suppresses output to console if true
///
/// # Returns
/// `Vec` with `Option` - `Some(..)` with elapsed time in milis on success `None` otherwise.
///
pub async fn wait_for_them(
    hosts_ports_or_http_urls: &[ToCheck],
    timeout: Option<u64>,
    start_time: Option<std::time::Instant>,
    silent: bool,
) -> Vec<Option<u64>> {
    let start_time = start_time.unwrap_or_else(std::time::Instant::now);
    let futures = if silent {
        scanner::wait_silent(hosts_ports_or_http_urls, timeout, start_time)
    } else {
        scanner::wait(hosts_ports_or_http_urls, timeout, start_time)
    };

    futures::future::join_all(futures).await
}
