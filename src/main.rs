use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod config;
mod dsn;
mod request;
mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read command line option for config/logging?
    let config_path = Path::new("./example.yaml");

    // Parse the configuration file
    let configdata = match config::load_config(config_path) {
        Ok(keys) => keys,
        Err(_) => {
            println!("Invalid configuration file");
            panic!("Could not parse configuration file");
        },
    };
    // Create keymap that we need to match incoming requests
    let keymap = dsn::make_key_map(configdata.keys);

    // Bind to a port
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    let arcmap = Arc::new(keymap);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let arcmap_loop = arcmap.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req: Request<Incoming>| {
                    service::handle_request(req, arcmap_loop.clone())
                }))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}


