use http::{header, Request, Response, StatusCode};
use std::collections::HashMap;
use url::Url;

fn handler(request: Request<()>) -> http::Result<Response<String>> {
    let url = Url::parse(&request.uri().to_string()).unwrap();
    let hash_query: HashMap<_, _> = url.query_pairs().to_owned().collect();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(format!(
            "user = {}\nrepo = {}",
            hash_query.get("user").unwrap(),
            hash_query.get("repo").unwrap()
        ))
        .expect("failed to render response");

    Ok(response)
}
