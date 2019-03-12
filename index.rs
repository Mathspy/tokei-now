use git2::Repository;
use http::{header, Request, Response, StatusCode};
use std::collections::HashMap;
use tokei::{Config, Languages};
use url::Url;

fn handler(request: Request<()>) -> http::Result<Response<String>> {
    let url = Url::parse(&request.uri().to_string()).unwrap();
    let hash_query: HashMap<_, _> = url.query_pairs().to_owned().collect();

    match (hash_query.get("user"), hash_query.get("repo")) {
        (Some(user), Some(repo)) => {
            let repo_url = format!("https://github.com/{}/{}", user, repo);

            match Repository::clone(&repo_url, "./repo") {
                Ok(repo) => repo,
                Err(e) => panic!("failed to clone: {}", e),
            };

            let mut languages = Languages::new();
            languages.get_statistics(&["./repo"], &[".git"], &Config::default());

            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(serde_json::to_string(&languages).unwrap())
                .expect("failed to render response");

            Ok(response)
        }

        _ => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("BAD REQUEST.\nUsage instruction: /<github_username>/<github_repo>/".to_string()),
    }
}

