use hyper::http::request::Builder as RequestBuilder;
use hyper::{HeaderMap, Request};

use crate::dsn;


/// Copy the relevant parts from `req` into a new request that can be sent
/// to the outbound DSN
pub fn make_outbound_request(headers: &HeaderMap, inbound: &dsn::Dsn, outbound: &dsn::Dsn) -> RequestBuilder {
    // TODO need to pass in URL context so we know which endpoint to use.
    let mut builder = Request::builder()
        // TODO need URL from original request.
        .uri("https://ingest.sentry.io/api/1/envelope/");

    let outbound_headers = builder.headers_mut().unwrap();
    for (key, value) in headers.iter() {
        if key == dsn::AUTHORIZATION_HEADER || key == dsn::SENTRY_X_AUTH_HEADER {
            let updated_value = value.to_str().unwrap().replace(&inbound.public_key, &outbound.public_key);
            outbound_headers.insert(key, updated_value.parse().unwrap());
        } else if key == "host" {
            outbound_headers.insert(key, outbound.host.parse().unwrap());
        } else {
            outbound_headers.insert(key, value.clone());
        }
    }

    builder
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn make_outbound_request_replace_sentry_auth_header() {
        let inbound = dsn::Dsn::from_string("https://abcdef@o123.ingest.sentry.io/12345".to_string()).unwrap();
        let outbound = dsn::Dsn::from_string("https://outbound@o123.ingest.sentry.io/6789".to_string()).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("Origin", "example.com".parse().unwrap());
        headers.insert("X-Sentry-Auth", "sentry_key=abcdef".parse().unwrap());

        let builder = make_outbound_request(&headers, &inbound, &outbound);
        let res = builder.body("");

        assert_eq!(res.is_ok(), true);
        let req = res.unwrap();
        let header_val = req.headers().get("X-Sentry-Auth").unwrap();
        assert_eq!(header_val, "sentry_key=outbound");
        assert_eq!(req.headers().contains_key("Origin"), true);
    }

    #[test]
    fn make_outbound_request_replace_authorization_header() {
        let inbound = dsn::Dsn::from_string("https://abcdef@o123.ingest.sentry.io/12345".to_string()).unwrap();
        let outbound = dsn::Dsn::from_string("https://outbound@o789.ingest.sentry.io/6789".to_string()).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Authorization", "sentry_version=7,sentry_key=abcdef".parse().unwrap());

        let builder = make_outbound_request(&headers, &inbound, &outbound);
        let res = builder.body("");

        assert_eq!(res.is_ok(), true);
        let req = res.unwrap();

        let mut header_val = req.headers().get("Authorization").unwrap();
        assert_eq!(header_val, "sentry_version=7,sentry_key=outbound");

        header_val = req.headers().get("Content-Type").unwrap();
        assert_eq!(header_val, "application/json");
    }

    #[test]
    fn make_outbound_request_replace_path_and_host() {
        let inbound = dsn::Dsn::from_string("https://abcdef@o123.ingest.sentry.io/12345".to_string()).unwrap();
        let outbound = dsn::Dsn::from_string("https://outbound@o789.ingest.sentry.io/6789".to_string()).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("Host", "o555.ingest.sentry.io".parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Authorization", "sentry_version=7,sentry_key=abcdef".parse().unwrap());

        let builder = make_outbound_request(&headers, &inbound, &outbound);
        let res = builder.body("");
        assert_eq!(res.is_ok(), true);
        let req = res.unwrap();

        let header_val = req.headers().get("Host").unwrap();
        assert_eq!(header_val, "o789.ingest.sentry.io");
    }
}
