use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::io::{AsyncRead, AsyncWrite};
use futures::{try_ready, Future, Poll};
use std::io;
use lazy_static::*;
use tokio::executor::current_thread::spawn;
use tokio::runtime::current_thread::run;
//use tokio::{run, spawn};

const SINGLE_RESP: &[u8] = b"HTTP/1.1 200 OK\r\nConnection: Keep-Alive\r\nContent-Length: 17\r\n\r\n{\"success\":true}\n";
lazy_static! {
    static ref RESPONSES: Vec<u8> = std::iter::repeat(SINGLE_RESP).take(256).flatten().cloned().collect();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let socket = TcpListener::bind(&addr)?;
    println!("Listening on: {}", addr);

    let task = socket
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |socket| {
            let (reader, writer) = socket.split();
            // Mapping the err to unit cause this task may be aborted
            // we do not care if it does
            spawn(responder(reader, writer).map_err(|_| ()));
            future::ok(())
        });

    run(task);
    Ok(())
}

// Responder
// We use Options so we can free them
#[derive(Debug)]
pub struct PipelineResponder<R, W> {
    reader: R,
    writer: W,
    read_done: bool,
    buf: Box<[u8]>,
    counter: HttpReqCounter,
    pos: usize,
    cap: usize,
}

// @TODO play with other BUF_SIZE
const BUF_SIZE: usize = 1024 * 16;
pub fn responder<R, W>(reader: R, writer: W) -> PipelineResponder<R, W>
where
    R: AsyncRead,
    W: AsyncWrite,
{
    PipelineResponder {
        reader,
        writer,
        read_done: false,
        buf: Box::new([0; BUF_SIZE]),
        counter: HttpReqCounter::default(),
        cap: 0,
        pos: 0,
    }
}

impl<R, W> Future for PipelineResponder<R, W>
where
    R: AsyncRead,
    W: AsyncWrite,
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            // similar to tokio::io::copy
            if self.pos == self.cap && !self.read_done {
                let n = try_ready!(self.reader.poll_read(&mut self.buf));
                if n == 0 {
                    self.read_done = true;
                } else {
                    self.counter.feed(&self.buf[0..n]);
                    self.pos = 0;
                    self.cap = self.counter.reset() * SINGLE_RESP.len();
                }
            }

            if self.pos < self.cap {
                let to_write = std::cmp::min(RESPONSES.len(), self.cap - self.pos);
                let i = try_ready!(self.writer.poll_write(&RESPONSES[0..to_write]));
                if i == 0 {
                    // This will cause the future to get stuck, so it's an error
                    return Err(io::Error::new(io::ErrorKind::WriteZero, "zero bytes written"));
                } else {
                    self.pos += i;
                }
            }

            if self.pos == self.cap && self.read_done {
                try_ready!(self.writer.poll_flush());
                return Ok(().into());
            }
        }
    }
}

// HTTP request counter
// This will likely be easier with nom
#[derive(Default, Debug)]
pub struct HttpReqCounter {
    state: u8,
    pub cnt: usize,
}
impl HttpReqCounter {
    pub fn feed(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.state = match (b, self.state) {
                (b'\r', 0) => 1,
                (b'\n', 1) => 2,
                (b'\r', 2) => 3,
                (b'\n', 3) => {
                    self.cnt = self.cnt + 1;
                    0
                }
                _ => 0
            }
        }
    }
    pub fn reset(&mut self) -> usize {
        let cnt = self.cnt;
        self.cnt = 0;
        cnt
    }
}

