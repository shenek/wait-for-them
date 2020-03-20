use std::{future::Future, pin::Pin, time::Instant};

pub struct Callback {
    pub success: Pin<Box<dyn Future<Output = ()>>>,
    pub error: Pin<Box<dyn Future<Output = ()>>>,
}

pub fn silent() -> Callback {
    Callback {
        success: Box::pin(async {}),
        error: Box::pin(async {}),
    }
}

pub fn simple(address: String, instant: Instant) -> Callback {
    let address_cloned = address.clone();
    Callback {
        success: Box::pin(async move {
            println!(
                "Successfully connected to '{}' in {:.3} seconds",
                address_cloned,
                instant.elapsed().as_secs_f32()
            )
        }),
        error: Box::pin(async move {
            println!(
                "Failed to connected to '{}' in {:.3} seconds",
                address,
                instant.elapsed().as_secs_f32()
            )
        }),
    }
}
