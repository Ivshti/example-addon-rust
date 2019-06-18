use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::codec;
use futures::future;

const SINGLE_RESP: &[u8] = b"HTTP/1.1 200 OK\r\nConnection: Keep-Alive\r\nContent-Length: 17\r\n\r\n{\"success\":true}\n";

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
                    // @TODO proper parsing here
                    let n = bytes.len() / 32;
                    // @TODO remove unwrap
                    let resp = (0..n)
                            .map(|_| SINGLE_RESP)
                            .flatten()
                            .cloned()
                            .collect::<Vec<u8>>();
                    writer.write(&resp);
                    //writer.write(&many_resps[0..n*SINGLE_RESP.len()]).unwrap();
                    future::ok(())
                });
            
            tokio::spawn(task
                // @TODO: handle errors and etc
                .then(|_| future::ok(()))
            )
        });

    tokio::run(done);
    Ok(())
}

