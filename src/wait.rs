use async_std::future;
use async_std::net::TcpStream;
use std::{thread, time::Duration};

pub struct Wait {
    address: String,
    timeout: Option<Duration>,
}

impl Wait {
    pub fn new(address: String, timeout: Option<Duration>) -> Self {
        Self { address, timeout }
    }

    async fn wait_for_connection(&self) {
        while TcpStream::connect(&self.address).await.is_err() {
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub async fn wait(self) -> bool {
        if let Some(timeout) = self.timeout {
            let res = future::timeout(timeout, self.wait_for_connection()).await;
            res.is_ok()
        } else {
            self.wait_for_connection().await;
            true
        }
    }
}
