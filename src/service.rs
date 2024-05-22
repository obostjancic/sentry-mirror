use std::{collections::HashMap, sync::Arc};

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Method, StatusCode};
use hyper::{Request, Response};

use crate::dsn;
use crate::request;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;
type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub async fn handle_request(
    req: Request<Incoming>, keymap: Arc<HashMap<String, dsn::DsnKeyRing>>
) -> Result<Response<BoxBody>> {
    let method = req.method();
    let uri = req.uri();
    let headers = req.headers();

    // All store/envelope requests are POST
    if method != Method::POST {
        let res = Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(full("Method not allowed"))
            .unwrap();
        return Ok(res);
    }
    // Find DSN public key in request
    let found_dsn = dsn::from_request(uri, headers);
    if found_dsn.is_none() {
        return Ok(bad_request_response());
    }
    // Match the public key with registered keys
    let public_key = found_dsn.unwrap();
    let keyring = match keymap.get(&public_key) {
        Some(v) => v,
        // If a DSN cannot be found -> empty response
        None => return Ok(bad_request_response())
    };
    for outbound_dsn in keyring.outbound.iter() {
        let _outbound_request = request::make_outbound_request(headers, &keyring.inbound, &outbound_dsn);
    }
    let whole_body = req.collect().await?.to_bytes();
    println!("body!! {:?}", whole_body);

    let res_body = full("hello");
    Ok(Response::new(res_body))
}


fn bad_request_response() -> Response<BoxBody> {
    return Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(full("No DSN found"))
        .unwrap();
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

/*
/// Send a request to its destination async
async fn send_request(req: Request<()>) -> Result<Response<BoxBody>> {
    let host = req.uri().host().expect("request uri has no host");
    let port = req.uri().port_u16().unwrap_or(80);

    /*
    let stream = TcpStream::connect((host, port)).await?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let resp = sender.send_request(req).await?;
    */
    Ok(Response::builder().body(full("works?")).unwrap())
}
*/
