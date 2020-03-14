use std::time::Duration;

use tokio::{net::TcpStream, time};

const RETRY_TIMEOUT: u64 = 100_u64;

pub struct Wait {
    address: String,
    timeout: Option<Duration>,
}

impl Wait {
    pub const fn new(address: String, timeout: Option<Duration>) -> Self {
        Self { address, timeout }
    }

    async fn wait_for_connection(&self) {
        while TcpStream::connect(&self.address).await.is_err() {
            time::delay_for(Duration::from_millis(RETRY_TIMEOUT)).await;
        }
    }

    pub async fn wait(self) -> bool {
        if let Some(timeout) = self.timeout {
            let res = time::timeout(timeout, self.wait_for_connection()).await;
            res.is_ok()
        } else {
            self.wait_for_connection().await;
            true
        }
    }
}
