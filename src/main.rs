
use {
    hyper::{
        Body, Request, Response, Server, Error,
        service::service_fn,
        rt::run,
    },
    futures::{Future, future},
    std::net::SocketAddr,
};

const RESP_BODY: &[u8] = include_bytes!("./resp.json");

fn on_request(_: Request<Body>) -> impl Future<Item=Response<Body>, Error=Error> {
    future::ok(Response::new(Body::from(RESP_BODY)))
}

fn run_server(addr: &SocketAddr) -> impl Future<Item=(), Error=Error> {
    Server::bind(addr).serve(|| service_fn(on_request))
}

fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Listening on http://{}", &addr);
    run(run_server(&addr)
        .then(|x| {
            // we can't .else and panic, cause rustc wants us to return a future
            // even though it'd be unreachable
            if let Err(e) = x {
                panic!(e);
            }
            future::ok(())
        })
    );
}

