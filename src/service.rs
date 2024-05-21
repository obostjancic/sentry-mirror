use std::{collections::HashMap, sync::Arc};
use std::future::Future;
use std::pin::Pin;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Method, StatusCode};
use hyper::{service::Service, Request, Response};

use crate::dsn;

#[derive(Debug, Clone)]
pub struct MirrorService {
    pub keymap: Arc<HashMap<String, Vec<dsn::Dsn>>>,
}

impl Service<Request<Incoming>> for MirrorService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        // All store/envelope requests are POST
        if req.method() != Method::POST {
            let mut res = Response::new(Full::new(Bytes::from("Method not allowed")));
            *res.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            return Box::pin(async { Ok(res) })
        }
        // Find DSN public key in request
        let found_dsn = dsn::from_request(req.uri(), req.headers());
        if found_dsn.is_none() {
            let res = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from("No DSN found")))
                .unwrap();
            return Box::pin(async { Ok(res) })
        }
        // Match the public key with registered keys

        // If a DSN cannot be found -> empty response

        // Match the inbound DSN to one we have in our configuration
        // If a DSN cannot be matched -> empty response

        // Async send a modified request to each outbound DSN

        let res = Response::new(Full::new(Bytes::from("Hello world")));
        return Box::pin(async { Ok(res) })
    }
}
