use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::codec;
use futures::future;

// @TODO generate a single &[u8] with many responses
const SINGLE_RESP: &[u8] = b"HTTP/1.1 200 OK\r\nConnection: Keep-Alive\r\nContent-Length: 17\r\n\r\n{\"success\":true}\n";
const RESPONSES: &[u8] = include_bytes!("./responses");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let socket = TcpListener::bind(&addr)?;
    println!("Listening on: {}", addr);

    let done = socket
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |socket| {
            let (reader, mut writer) = socket.split();
            let task = codec::FramedRead::new(reader, codec::BytesCodec::new())
                .for_each(move |bytes| {
                    // @TODO get rid of the unwrap; and handle out of bounds here
                    writer.write(&RESPONSES[0..(bytes.len()/32)*SINGLE_RESP.len()]).unwrap();
                    future::ok(())
                });
            
            tokio::spawn(task.map_err(|_| ()))
        });

    tokio::run(done);
    Ok(())
}

