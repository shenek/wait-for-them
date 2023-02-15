use std::{
    io::{Read, Write},
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
                    Ok(mut strm) => {
                        let mut buff = Vec::with_capacity(1024);
                        while let Ok(size) = strm.read(&mut buff) {
                            if size == 0 {
                                let response = b"\
HTTP/1.1 200 OK
Date: Sat, 01 Jan 2020 00:00:00 GMT
Content-Type: text/html; charset=UTF-8
Server: Test/0.0.0 (Wait-For-It)
Connection: close

OK
";
                                let _ = strm.write_all(response);
                                break;
                            }
                        }
                    }
                    Err(_) => {}
                }
                if exitting.as_ref().load(Ordering::Relaxed) {
                    break;
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
