use hyper::http::request::Builder as RequestBuilder;
use hyper::http::uri::PathAndQuery;
use hyper::{HeaderMap, Request, Uri};
use regex::Regex;

use crate::dsn;

/// Copy the relevant parts from `uri` and `headers` into a new request that can be sent
/// to the outbound DSN. This function returns `RequestBuilder` because the body types
/// are tedious to deal with.
pub fn make_outbound_request(
    uri: &Uri,
    headers: &HeaderMap,
    outbound: &dsn::Dsn,
) -> RequestBuilder {
    // Update project id in the path
    let mut new_path = uri.path().to_string();
    let path_parts: Vec<_> = uri.path().split('/').filter(|i| !i.is_empty()).collect();
    if path_parts.len() == 3 && path_parts[0] == "api" {
        let original_projectid = path_parts[1];
        let new_project_id = outbound.project_id.clone();
        new_path = new_path.replace(original_projectid, &new_project_id);
    }
    // Replace public keys in the query string
    let query = match uri.query() {
        Some(value) => replace_public_key(value, outbound),
        None => String::new(),
    };

    let path_query: PathAndQuery = if !query.is_empty() {
        format!("{new_path}?{query}").parse().unwrap()
    } else {
        new_path.parse().unwrap()
    };
    let new_uri = Uri::builder()
        .scheme(outbound.scheme.as_str())
        .authority(outbound.host.clone())
        .path_and_query(path_query)
        .build();

    let mut builder = Request::builder().method("POST").uri(new_uri.unwrap());

    let outbound_headers = builder.headers_mut().unwrap();
    for (key, value) in headers.iter() {
        if key == dsn::AUTHORIZATION_HEADER || key == dsn::SENTRY_X_AUTH_HEADER {
            let updated_value = replace_public_key(value.to_str().unwrap(), outbound);
            outbound_headers.insert(key, updated_value.parse().unwrap());
        } else if key == "host" {
            outbound_headers.insert(key, outbound.host.parse().unwrap());
        } else {
            outbound_headers.insert(key, value.clone());
        }
    }

    builder
}

fn replace_public_key(target: &str, outbound: &dsn::Dsn) -> String {
    let pattern = Regex::new(r"sentry_key=([a-f0-9]+)").unwrap();
    let public_key = &outbound.public_key;
    let replacement = format!("sentry_key={public_key}");
    let res = pattern.replace(target, replacement);

    res.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_outbound_request_replace_sentry_auth_header() {
        let outbound: dsn::Dsn = "https://outbound@o123.ingest.sentry.io/6789"
            .parse()
            .unwrap();
        let uri: Uri = "https://o123.ingest.sentry.io/api/1/envelope/"
            .parse()
            .unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("Origin", "example.com".parse().unwrap());
        headers.insert("X-Sentry-Auth", "sentry_key=abcdef".parse().unwrap());

        let builder = make_outbound_request(&uri, &headers, &outbound);
        let res = builder.body("");

        assert!(res.is_ok());
        let req = res.unwrap();
        let header_val = req.headers().get("X-Sentry-Auth").unwrap();
        assert_eq!(header_val, "sentry_key=outbound");
        assert!(req.headers().contains_key("Origin"));
        assert_eq!(req.method(), "POST");
    }

    #[test]
    fn make_outbound_request_replace_authorization_header() {
        let outbound: dsn::Dsn = "https://outbound@o789.ingest.sentry.io/6789"
            .parse()
            .unwrap();
        let uri: Uri = "https://o123.ingest.sentry.io/api/1/envelope/"
            .parse()
            .unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert(
            "Authorization",
            "sentry_version=7,sentry_key=abcdef".parse().unwrap(),
        );

        let builder = make_outbound_request(&uri, &headers, &outbound);
        let res = builder.body("");

        assert!(res.is_ok());
        let req = res.unwrap();

        let mut header_val = req.headers().get("Authorization").unwrap();
        assert_eq!(header_val, "sentry_version=7,sentry_key=outbound");

        header_val = req.headers().get("Content-Type").unwrap();
        assert_eq!(header_val, "application/json");
        assert_eq!(req.method(), "POST");
    }

    #[test]
    fn make_outbound_request_replace_query_key() {
        let outbound: dsn::Dsn = "https://outbound@o789.ingest.sentry.io/6789"
            .parse()
            .unwrap();
        let uri: Uri =
            "https://o123.ingest.sentry.io/api/1/envelope/?sentry_key=abcdef&sentry_version=7"
                .parse()
                .unwrap();

        let headers = HeaderMap::new();
        let builder = make_outbound_request(&uri, &headers, &outbound);
        let res = builder.body("");
        assert!(res.is_ok());
        let req = res.unwrap();

        let uri = req.uri();
        assert_eq!(
            uri,
            "https://o789.ingest.sentry.io/api/6789/envelope/?sentry_key=outbound&sentry_version=7"
        );
    }

    #[test]
    fn make_outbound_request_replace_path_host_and_scheme() {
        let outbound: dsn::Dsn = "https://outbound@o789.ingest.sentry.io/6789"
            .parse()
            .unwrap();
        let uri: Uri = "http://o123.ingest.sentry.io/api/1/envelope/"
            .parse()
            .unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("Host", "o555.ingest.sentry.io".parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert(
            "Authorization",
            "sentry_version=7,sentry_key=abcdef".parse().unwrap(),
        );

        let builder = make_outbound_request(&uri, &headers, &outbound);
        let res = builder.body("");
        assert!(res.is_ok());
        let req = res.unwrap();

        let header_val = req.headers().get("Host").unwrap();
        assert_eq!(header_val, "o789.ingest.sentry.io");
        let uri = req.uri();
        assert_eq!(uri, "https://o789.ingest.sentry.io/api/6789/envelope/");
    }
}
