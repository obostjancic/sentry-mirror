use std::{collections::HashMap, sync::Arc};
use std::future::Future;
use std::pin::Pin;

use http_body_util::Full;
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

    fn call(&self, _req: Request<IncomingBody>) -> Self::Future {
        let res = Response::new(Full::new(Bytes::from("Hello world")));

        return Box::pin(async { Ok(res) })
    }
}
