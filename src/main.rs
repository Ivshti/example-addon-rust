#![feature(async_await, await_macro, futures_api)]


use {
    hyper::{
        Body, Client, Request, Response, Server, Uri,

        service::service_fn,

        rt::run,
    },
    futures::{
        future::{FutureExt, TryFutureExt},
    },
    std::net::SocketAddr,

    tokio::await,
};

const MANIFEST: &[u8] = include_bytes!("./manifest.json");

async fn serve_req(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from(MANIFEST)))
}

async fn run_server(addr: SocketAddr) {
    println!("Listening on http://{}", addr);

    // Create a server bound on the provided address
    let serve_future = Server::bind(&addr)
        // Serve requests using our `async serve_req` function.
        // `serve` takes a closure which returns a type implementing the
        // `Service` trait. `service_fn` returns a value implementing the
        // `Service` trait, and accepts a closure which goes from request
        // to a future of the response. In order to use our `serve_req`
        // function with Hyper, we have to box it and put it in a compatability
        // wrapper to go from a futures 0.3 future (the kind returned by
        // `async fn`) to a futures 0.1 future (the kind used by Hyper).
        .serve(|| service_fn(|req|
            serve_req(req).boxed().compat()
        ));

    // Wait for the server to complete serving or exit with an error.
    // If an error occurred, print it to stderr.
    if let Err(e) = await!(serve_future) {
        eprintln!("server error: {}", e);
    }
}

fn main() {
    // Set the address to run our socket on.
    let addr = SocketAddr::from(([127, 0, 0, 1], 7778));

    let futures_03_future = run_server(addr);
    let futures_01_future =
        futures_03_future.unit_error().boxed().compat();

    // Finally, we can run the future to completion using the `run` function
    // provided by Hyper.
    run(futures_01_future);
}

