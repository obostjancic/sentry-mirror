use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use log::{debug, warn};
use std::{collections::HashMap, sync::Arc};

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use hyper_tls::HttpsConnector;

use crate::dsn;
use crate::request;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;
type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub async fn handle_request(
    req: Request<Incoming>,
    keymap: Arc<HashMap<String, dsn::DsnKeyRing>>,
) -> Result<Response<BoxBody>> {
    let method = req.method();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    // All store/envelope requests are POST
    if method != Method::POST {
        debug!("Received a non POST request");
        let res = Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(full("Method not allowed"))
            .unwrap();
        return Ok(res);
    }
    // Find DSN public key in request
    let found_dsn = dsn::from_request(&uri, &headers);
    if found_dsn.is_none() {
        debug!("Could not find a DSN in the request headers or URI");
        return Ok(bad_request_response());
    }
    // Match the public key with registered keys
    let public_key = found_dsn.unwrap();
    let keyring = match keymap.get(&public_key) {
        Some(v) => v,
        // If a DSN cannot be found -> empty response
        None => {
            debug!("Could not find a match DSN in the configured keys");
            return Ok(bad_request_response());
        }
    };
    let mut body_bytes = req.collect().await?.to_bytes();

    // Bodies can be compressed
    if headers.contains_key("content-encoding") {
        let request_encoding = headers.get("content-encoding").unwrap();
        body_bytes = match request::decode_body(request_encoding, &body_bytes) {
            Ok(decompressed) => decompressed,
            Err(e) => {
                warn!("Could not decode request body: {0:?}", e);
                return Ok(bad_request_response());
            },
        }
    }

    for outbound_dsn in keyring.outbound.iter() {
        debug!("Creating outbound request for {0}", &outbound_dsn.host);
        let request_builder = request::make_outbound_request(&uri, &headers, outbound_dsn);

        let body_out = match request::replace_envelope_dsn(&body_bytes, outbound_dsn) {
            Some(new_body) => new_body,
            None => body_bytes.clone(),
        };
        let request = request_builder.body(Full::new(body_out));
        if let Ok(outbound_request) = request {
            send_request(outbound_request).await.map_or_else(
                |e| warn!("Request failed: {0}", e),
                |res| debug!("request complete: {0}", res),
            );
        } else {
            warn!("Could not build request {0:?}", request.err());
        }
    }

    // TODO need an event id to match return of relay
    Ok(Response::new(full("ok")))
}

fn bad_request_response() -> Response<BoxBody> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(full("No DSN found"))
        .unwrap()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

/// Send a request to its destination async
async fn send_request(req: Request<Full<Bytes>>) -> Result<String> {
    let https = HttpsConnector::new();
    let client = Client::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(https);
    let resp = client.request(req).await?;
    if !resp.status().is_success() {
        warn!("Request did not succeed");

        let headers = resp.headers().clone();
        let body = resp.collect().await?.to_bytes();
        warn!("Response Headers: {0:?}", headers);
        warn!("Response body: {:?}", body);
    }

    Ok("ok".to_string())
}
