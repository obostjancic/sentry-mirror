use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod config;
mod dsn;

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
    println!("{:?}", configdata);

    // Bind to a port
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            // Attach the mirror service function
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(hello))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    // Find DSN in request
    // If a DSN cannot be found -> empty response

    // Match the inbound DSN to one we have in our configuration
    // If a DSN cannot be matched -> empty response

    // Async send a modified request to each outbound DSN
    Ok(Response::new(Full::new(Bytes::from("Hello world"))))
}
