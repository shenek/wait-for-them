#[cfg(feature = "http")]
use bytes::Bytes;
#[cfg(feature = "http")]
use http_body_util::Empty;
#[cfg(feature = "http")]
use hyper::StatusCode;
#[cfg(feature = "http")]
use hyper_tls::HttpsConnector;
#[cfg(feature = "http")]
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
#[cfg(feature = "ui")]
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
#[cfg(feature = "ui")]
use std::sync::Arc;
use std::{
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};
#[cfg(feature = "ui")]
use tokio::sync::Mutex;
use tokio::{self, net::TcpStream, time};

use crate::ToCheck;

const RETRY_TIMEOUT: u64 = 100_u64;
const NO_RESPONSE_TIMEOUT: u64 = 1000_u64;

#[cfg(not(feature = "ui"))]
pub fn wait(
    hosts_ports_or_http_urls: &[ToCheck],
    timeout: Option<u64>,
    instant: Instant,
) -> Vec<Pin<Box<dyn Future<Output = Option<u64>>>>> {
    hosts_ports_or_http_urls
        .iter()
        .map(|to_check| {
            let generator = ProgressGenerator {
                to_check: to_check.clone(),
                instant,
            };
            Wait::new(
                to_check.clone(),
                timeout.map(Duration::from_millis),
                Box::new(generator),
            )
            .wait_future()
        })
        .collect()
}

#[cfg(feature = "ui")]
pub fn wait(
    hosts_ports_or_http_urls: &[ToCheck],
    timeout: Option<u64>,
    instant: Instant,
) -> Vec<Pin<Box<dyn Future<Output = Option<u64>>>>> {
    let multiple = MultiProgress::new();
    hosts_ports_or_http_urls
        .iter()
        .map(|to_check| {
            let pb = if let Some(timeout) = timeout {
                multiple.add(ProgressBar::new(timeout).with_finish(ProgressFinish::AndLeave))
            } else {
                multiple.add(ProgressBar::new_spinner().with_finish(ProgressFinish::AndLeave))
            };
            let sty = if timeout.is_some() {
                ProgressStyle::default_bar()
                    .template(&format!(
                        "{} {}",
                        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}", to_check
                    ))
                    .unwrap()
                    .progress_chars("##-")
            } else {
                ProgressStyle::default_spinner()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                    .template(&format!(
                        "{} {}",
                        "[{elapsed_precise}] {spinner} {msg}", to_check
                    ))
                    .unwrap()
            };
            pb.set_style(sty);
            pb.set_message(" ");
            pb.tick();

            let generator = ProgressGenerator {
                instant,
                progress: Arc::new(Mutex::new(pb)),
            };
            Wait::new(
                to_check.clone(),
                timeout.map(Duration::from_millis),
                Box::new(generator),
            )
            .wait_future()
        })
        .collect()
}

pub fn wait_silent(
    hosts_ports_or_http_urls: &[ToCheck],
    timeout: Option<u64>,
    instant: Instant,
) -> Vec<Pin<Box<dyn Future<Output = Option<u64>>>>> {
    hosts_ports_or_http_urls
        .iter()
        .map(|to_check| {
            let progress = SilentGenerator::new(instant);
            Wait::new(
                to_check.clone(),
                timeout.map(Duration::from_millis),
                Box::new(progress),
            )
            .wait_future()
        })
        .collect()
}

struct Wait {
    to_check: ToCheck,
    timeout: Option<Duration>,
    generator: Box<dyn Generator>,
}

impl Wait {
    pub fn new(
        to_check: ToCheck,
        timeout: Option<Duration>,
        generator: Box<dyn Generator>,
    ) -> Self {
        Self {
            to_check,
            timeout,
            generator,
        }
    }

    async fn wait_for_connection_tcp(&mut self) {
        loop {
            self.generator.generate_tick().await;
            let ToCheck::HostnameAndPort(ref domain, port) = self.to_check else {
                unreachable!()
            };
            let timeout = time::timeout(
                Duration::from_millis(NO_RESPONSE_TIMEOUT),
                TcpStream::connect((domain.as_str(), port)),
            )
            .await;
            if timeout.is_err() {
                continue;
            }
            if timeout.unwrap().is_err() {
                time::sleep(Duration::from_millis(RETRY_TIMEOUT)).await;
            } else {
                break;
            }
        }
    }

    #[cfg(feature = "http")]
    async fn wait_for_connection_http(&mut self) {
        let https_or_http = HttpsConnector::new();

        let client: Client<_, Empty<Bytes>> =
            Client::builder(TokioExecutor::new()).build(https_or_http);

        let ToCheck::HttpOrHttpsUrl(ref url) = self.to_check else {
            unreachable!()
        };

        loop {
            self.generator.generate_tick().await;
            let timeout = time::timeout(
                Duration::from_millis(NO_RESPONSE_TIMEOUT),
                client.get(url.clone()),
            )
            .await;

            match timeout {
                Ok(Err(_)) => {
                    time::sleep(Duration::from_millis(RETRY_TIMEOUT)).await;
                }
                Ok(Ok(resp)) => {
                    if resp.status() == StatusCode::OK {
                        break;
                    } else {
                        time::sleep(Duration::from_millis(RETRY_TIMEOUT)).await;
                    }
                }
                Err(_) => continue,
            }
        }
    }

    #[cfg(not(feature = "http"))]
    async fn wait_for_connection_http(&mut self) {
        panic!("Not compiled with 'http' feature")
    }

    fn wait_future(mut self) -> Pin<Box<dyn Future<Output = Option<u64>>>> {
        Box::pin(async move {
            if let Some(timeout) = self.timeout {
                let res = if let ToCheck::HostnameAndPort(..) = self.to_check {
                    time::timeout(timeout, self.wait_for_connection_tcp()).await
                } else {
                    time::timeout(timeout, self.wait_for_connection_http()).await
                };
                if res.is_ok() {
                    Some(self.generator.generate_success().await)
                } else {
                    self.generator.generate_error().await;
                    None
                }
            } else {
                if let ToCheck::HostnameAndPort(..) = self.to_check {
                    self.wait_for_connection_tcp().await;
                } else {
                    self.wait_for_connection_http().await;
                };
                Some(self.generator.generate_success().await)
            }
        })
    }
}

#[allow(dead_code)]
pub trait Generator {
    fn generate_tick(&mut self) -> Pin<Box<dyn Future<Output = ()>>>;
    fn generate_error(&mut self) -> Pin<Box<dyn Future<Output = ()>>>;
    fn generate_start(&mut self) -> Pin<Box<dyn Future<Output = ()>>>;
    fn generate_success(&mut self) -> Pin<Box<dyn Future<Output = u64>>>;
}

pub struct SilentGenerator {
    instant: Instant,
}

impl SilentGenerator {
    pub fn new(instant: Instant) -> Self {
        Self { instant }
    }
}

impl Generator for SilentGenerator {
    fn generate_tick(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async {})
    }
    fn generate_error(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async {})
    }
    fn generate_start(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async {})
    }
    fn generate_success(&mut self) -> Pin<Box<dyn Future<Output = u64>>> {
        let instant = self.instant;
        Box::pin(async move { instant.elapsed().as_millis() as u64 })
    }
}

#[cfg(not(feature = "ui"))]
pub struct ProgressGenerator {
    to_check: ToCheck,
    instant: Instant,
}

#[cfg(not(feature = "ui"))]
impl Generator for ProgressGenerator {
    fn generate_tick(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async {})
    }

    fn generate_start(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async {}) // TODO something more reasonable
    }

    fn generate_error(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        let to_check = self.to_check.clone();
        let instant = self.instant;

        Box::pin(async move {
            println!(
                "Failed to connect to '{}' in {:.3} seconds",
                to_check,
                instant.elapsed().as_secs_f32()
            )
        })
    }

    fn generate_success(&mut self) -> Pin<Box<dyn Future<Output = u64>>> {
        let to_check = self.to_check.clone();
        let instant = self.instant;

        Box::pin(async move {
            println!(
                "Successfully connected to '{}' in {:.3} seconds",
                to_check,
                instant.elapsed().as_secs_f32()
            );
            instant.elapsed().as_millis() as u64
        })
    }
}

#[cfg(feature = "ui")]
pub struct ProgressGenerator {
    instant: Instant,
    progress: Arc<Mutex<ProgressBar>>,
}

#[cfg(feature = "ui")]
impl Generator for ProgressGenerator {
    fn generate_tick(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        let progress = self.progress.clone();
        let instant = self.instant;
        self.instant = Instant::now();
        Box::pin(async move {
            progress
                .lock()
                .await
                .inc(instant.elapsed().as_millis() as u64);
        })
    }

    fn generate_start(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async {}) // TODO something more reasonable
    }

    fn generate_error(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        let progress = self.progress.clone();
        Box::pin(async move {
            let unlocked = progress.lock().await;
            unlocked.finish_with_message("✘");
        })
    }

    fn generate_success(&mut self) -> Pin<Box<dyn Future<Output = u64>>> {
        let progress = self.progress.clone();
        let instant = self.instant;
        Box::pin(async move {
            let milis: u64 = instant.elapsed().as_millis() as u64;
            let unlocked = progress.lock().await;
            unlocked.set_message("✔");
            unlocked.abandon();
            milis
        })
    }
}
