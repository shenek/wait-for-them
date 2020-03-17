use std::{future::Future, pin::Pin, time::Duration};

use tokio::{net::TcpStream, time};

const RETRY_TIMEOUT: u64 = 100_u64;

pub struct Wait {
    address: String,
    timeout: Option<Duration>,
    success_callback: Pin<Box<dyn Future<Output = ()>>>,
    error_callback: Pin<Box<dyn Future<Output = ()>>>,
}

impl Wait {
    pub fn new(
        address: String,
        timeout: Option<Duration>,
        success_callback: Pin<Box<dyn Future<Output = ()>>>,
        error_callback: Pin<Box<dyn Future<Output = ()>>>,
    ) -> Self {
        Self {
            address,
            timeout,
            success_callback,
            error_callback,
        }
    }

    async fn wait_for_connection(&self) {
        while TcpStream::connect(&self.address).await.is_err() {
            time::delay_for(Duration::from_millis(RETRY_TIMEOUT)).await;
        }
    }

    pub async fn wait(self) -> bool {
        if let Some(timeout) = self.timeout {
            let res = time::timeout(timeout, self.wait_for_connection()).await;
            if res.is_ok() {
                self.success_callback.await;
                true
            } else {
                self.error_callback.await;
                false
            }
        } else {
            self.wait_for_connection().await;
            self.success_callback.await;
            true
        }
    }
}
