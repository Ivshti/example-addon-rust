use hyper::{Body, Response, Server, Error};
use hyper::service::service_fn_ok;
use hyper::rt::{Future};
use tokio::runtime::current_thread;
use std::net::SocketAddr;

const RESP_BODY: &[u8] = include_bytes!("./resp.json");

fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    let exec = current_thread::TaskExecutor::current();

    let new_service = || service_fn_ok(|_req| Response::new(Body::from(RESP_BODY)));
    let server = Server::bind(&addr)
        .executor(exec)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);

    current_thread::Runtime::new()
        .expect("rt new")
        .spawn(server)
        .run()
        .expect("rt run");
}

