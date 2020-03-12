use std::{
    net::TcpListener,
    ops::Drop,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

pub struct TestServer {
    exitting: Arc<AtomicBool>,
}

impl TestServer {
    pub fn new(port: u16, timeout: Duration) -> Self {
        let exitting = Arc::new(AtomicBool::new(false));
        let exitting_cloned = exitting.clone();
        thread::spawn(move || {
            let exitting = exitting_cloned;
            thread::sleep(timeout);
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).expect("can't connect");
            listener
                .set_nonblocking(true)
                .expect("Cant be non-blocking");
            for stream in listener.incoming() {
                match stream {
                    _ => {
                        if exitting.as_ref().load(Ordering::Relaxed) {
                            break;
                        }
                    }
                }
            }
        });
        Self { exitting }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.exitting.store(true, Ordering::Relaxed);
    }
}
