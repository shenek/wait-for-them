use futures::future::join_all;
#[cfg(feature = "ui")]
use indicatif::{MultiProgress, ProgressBar, ProgressStyle, TickTimeLimit};
#[cfg(feature = "ui")]
use std::sync::{Arc, Mutex};
use std::{
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};
use tokio::{self, net::TcpStream, time};

const RETRY_TIMEOUT: u64 = 100_u64;
const NO_RESPONSE_TIMEOUT: u64 = 1000_u64;

pub async fn perform(
    hosts: &[String],
    timeout: Option<u64>,
    instant: Instant,
    silent: bool,
) -> Vec<Option<u64>> {
    let futures = if silent {
        wait_silent(hosts, timeout, instant)
    } else {
        wait(hosts, timeout, instant)
    };

    join_all(futures).await
}

#[cfg(not(feature = "ui"))]
pub fn wait(
    hosts: &[String],
    timeout: Option<u64>,
    instant: Instant,
) -> Vec<Pin<Box<dyn Future<Output = Option<u64>>>>> {
    hosts
        .iter()
        .map(|addr| {
            let generator = ProgressGenerator {
                address: addr.clone(),
                instant,
            };
            Wait::new(
                addr.clone(),
                timeout.map(Duration::from_millis),
                Box::new(generator),
            )
            .wait_future()
        })
        .collect()
}

#[cfg(feature = "ui")]
pub fn wait(
    hosts: &[String],
    timeout: Option<u64>,
    instant: Instant,
) -> Vec<Pin<Box<dyn Future<Output = Option<u64>>>>> {
    let multiple = Arc::new(Mutex::new(MultiProgress::new()));
    hosts
        .iter()
        .map(|addr| {
            let pb = if let Some(timeout) = timeout {
                multiple.lock().unwrap().add(ProgressBar::new(timeout))
            } else {
                multiple.lock().unwrap().add(ProgressBar::new_spinner())
            };
            let sty = if timeout.is_some() {
                ProgressStyle::default_bar()
                    .template(&format!(
                        "{} {}",
                        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}", addr
                    ))
                    .progress_chars("##-")
            } else {
                ProgressStyle::default_spinner()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                    .template(&format!(
                        "{} {}",
                        "[{elapsed_precise}] {spinner} {msg}", addr
                    ))
            };
            pb.set_style(sty);
            pb.set_message(" ");
            pb.tick();

            let generator = ProgressGenerator {
                instant,
                multiple: multiple.clone(),
                progress: Arc::new(Mutex::new(pb)),
            };
            Wait::new(
                addr.clone(),
                timeout.map(Duration::from_millis),
                Box::new(generator),
            )
            .wait_future()
        })
        .collect()
}

pub fn wait_silent(
    hosts: &[String],
    timeout: Option<u64>,
    instant: Instant,
) -> Vec<Pin<Box<dyn Future<Output = Option<u64>>>>> {
    hosts
        .iter()
        .map(|addr| {
            let progress = SilentGenerator::new(instant);
            Wait::new(
                addr.clone(),
                timeout.map(Duration::from_millis),
                Box::new(progress),
            )
            .wait_future()
        })
        .collect()
}

struct Wait {
    address: String,
    timeout: Option<Duration>,
    generator: Box<dyn Generator>,
}

impl Wait {
    pub fn new(address: String, timeout: Option<Duration>, generator: Box<dyn Generator>) -> Self {
        Self {
            address,
            timeout,
            generator,
        }
    }

    async fn wait_for_connection(&mut self) {
        loop {
            self.generator.generate_tick().await;
            let timeout = time::timeout(
                Duration::from_millis(NO_RESPONSE_TIMEOUT),
                TcpStream::connect(&self.address),
            )
            .await;
            if timeout.is_err() {
                continue;
            }
            if timeout.unwrap().is_err() {
                time::delay_for(Duration::from_millis(RETRY_TIMEOUT)).await;
            } else {
                break;
            }
        }
    }

    fn wait_future(mut self) -> Pin<Box<dyn Future<Output = Option<u64>>>> {
        Box::pin(async move {
            if let Some(timeout) = self.timeout {
                let res = time::timeout(timeout, self.wait_for_connection()).await;
                if res.is_ok() {
                    Some(self.generator.generate_success().await)
                } else {
                    self.generator.generate_error().await;
                    None
                }
            } else {
                self.wait_for_connection().await;
                Some(self.generator.generate_success().await)
            }
        })
    }
}

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
    instant: Instant,
    address: String,
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
        let address = self.address.clone();
        let instant = self.instant;

        Box::pin(async move {
            println!(
                "Failed to connect to '{}' in {:.3} seconds",
                address,
                instant.elapsed().as_secs_f32()
            )
        })
    }

    fn generate_success(&mut self) -> Pin<Box<dyn Future<Output = u64>>> {
        let address = self.address.clone();
        let instant = self.instant;

        Box::pin(async move {
            println!(
                "Successfully connected to '{}' in {:.3} seconds",
                address,
                instant.elapsed().as_secs_f32()
            );
            instant.elapsed().as_millis() as u64
        })
    }
}

#[cfg(feature = "ui")]
pub struct ProgressGenerator {
    instant: Instant,
    multiple: Arc<Mutex<MultiProgress>>,
    progress: Arc<Mutex<ProgressBar>>,
}

#[cfg(feature = "ui")]
impl Generator for ProgressGenerator {
    fn generate_tick(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        let progress = self.progress.clone();
        let multiple = self.multiple.clone();
        let instant = self.instant;
        self.instant = Instant::now();
        Box::pin(async move {
            progress
                .lock()
                .unwrap()
                .inc(instant.elapsed().as_millis() as u64);
            multiple
                .lock()
                .unwrap()
                .tick(TickTimeLimit::Indefinite)
                .unwrap_or(());
        })
    }

    fn generate_start(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async {}) // TODO something more reasonable
    }

    fn generate_error(&mut self) -> Pin<Box<dyn Future<Output = ()>>> {
        let progress = self.progress.clone();
        let multiple = self.multiple.clone();
        Box::pin(async move {
            let unlocked = progress.lock().unwrap();
            unlocked.finish_with_message("✘");
            multiple
                .clone()
                .lock()
                .unwrap()
                .tick(TickTimeLimit::Indefinite)
                .unwrap_or(());
        })
    }

    fn generate_success(&mut self) -> Pin<Box<dyn Future<Output = u64>>> {
        let progress = self.progress.clone();
        let multiple = self.multiple.clone();
        let instant = self.instant;
        Box::pin(async move {
            let unlocked = progress.lock().unwrap();
            unlocked.set_message("✔");
            unlocked.finish_at_current_pos();
            multiple
                .clone()
                .lock()
                .unwrap()
                .tick(TickTimeLimit::Indefinite)
                .unwrap_or(());
            instant.elapsed().as_millis() as u64
        })
    }
}
