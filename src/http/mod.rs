use crate::pool::Pool;
use std::{
    error::Error,
    fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

pub struct Http {
    pub listener: Option<TcpListener>,
}

impl Http {
    pub fn new() -> Self {
        Self { listener: None }
    }

    pub fn listen(&mut self, port: &str) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(String::from("127.0.0.1:") + port)?;
        let pool = Pool::new(3)?;
        listener.incoming().take(2).for_each(|result| {
            let exec_result = pool.execute(|| {
                if let Err(err) = Self::handle_conn(result) {
                    eprintln!("handling connection failed: {:?}", err);
                }
            });
            if let Err(err) = exec_result {
                eprintln!("send msg failed: {:?}", err);
            }
        });
        self.listener = Some(listener);
        Ok(())
    }

    fn handle_conn(stream_result: Result<TcpStream, io::Error>) -> Result<(), Box<dyn Error>> {
        // read req
        let mut stream = stream_result?;
        let mut req_buf = [0; 1024];
        stream.read(&mut req_buf)?;

        // make res
        let (status, file_name) = Self::route(&req_buf);
        let contents = fs::read_to_string(file_name)?;
        let res = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status,
            contents.len(),
            contents
        );
        stream.write(res.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    // TODO: need to make router
    fn route(buf: &[u8]) -> (&str, &str) {
        let index_req = b"GET / HTTP/1.1\r\n";
        let sleep_req = b"GET /sleep HTTP/1.1\r\n";

        if buf.starts_with(index_req) {
            ("HTTP/1.1 200 OK", "index.html")
        } else if buf.starts_with(sleep_req) {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index.html")
        } else {
            ("HTTP/1.1 404 NOT FOUND", "404.html")
        }
    }
}
