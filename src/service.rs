use std::{collections::HashMap, sync::Arc};
use std::future::Future;
use std::pin::Pin;

use http_body_util::Full;
use hyper::{Method, StatusCode};
use hyper::{body::{Bytes, Incoming as IncomingBody}, service::Service, Request, Response};

use crate::dsn::Dsn;

#[derive(Debug, Clone)]
pub struct MirrorService {
    pub keymap: Arc<HashMap<String, Vec<Dsn>>>,
}

impl Service<Request<IncomingBody>> for MirrorService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        // All store/envelope requests are POST
        if req.method() != Method::POST {
            let mut res = Response::new(Full::new(Bytes::from("Method not allowed")));
            *res.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            return Box::pin(async { Ok(res) })
        }

        // Find DSN in request
        // If a DSN cannot be found -> empty response

        // Match the inbound DSN to one we have in our configuration
        // If a DSN cannot be matched -> empty response

        // Async send a modified request to each outbound DSN

        let res = Response::new(Full::new(Bytes::from("Hello world")));
        return Box::pin(async { Ok(res) })
    }
}
