use core::fmt::Display;
use git2::Repository;
use http::{header, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::ErrorKind};
use tokei::{Config, Language, Languages};
use url::Url;

#[derive(Serialize, Deserialize)]
struct Lang {
    name: String,
    files: usize,
    lines: usize,
    code: usize,
    comments: usize,
    blanks: usize,
}
impl Lang {
    fn from_language<S: Display>((name, language): (&S, &Language)) -> Lang {
        Lang {
            name: name.to_string(),
            files: language.stats.len(),
            lines: language.lines,
            code: language.code,
            comments: language.comments,
            blanks: language.blanks,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Output {
    languages: Vec<Lang>,
    total: Lang,
}

fn make_json(languages: Languages) -> Output {
    Output {
        languages: languages
            .iter()
            .filter(|(_, language)| !language.is_empty())
            .map(Lang::from_language)
            .collect(),
        total: Lang::from_language((
            &"Total",
            &languages
                .into_iter()
                .fold(Language::new(), |mut total, (_, language)| {
                    total += language;
                    total
                }),
        )),
    }
}

fn handler(request: Request<()>) -> http::Result<Response<String>> {
    let url = Url::parse(&request.uri().to_string()).unwrap();
    let hash_query: HashMap<_, _> = url.query_pairs().to_owned().collect();

    match (hash_query.get("user"), hash_query.get("repo")) {
        (Some(user), Some(repo)) => {
            let repo_url = format!("https://github.com/{}/{}", user, repo);

            match fs::remove_dir_all("/tmp/repo") {
                Ok(_) => (),
                Err(e) => {
                    if e.kind() != ErrorKind::NotFound {
                        return Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(format!("Failed to remove directory\n> {}", e,));
                    }
                }
            };

            match Repository::clone(&repo_url, "/tmp/repo") {
                Ok(_) => (),
                Err(e) => {
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(format!(
                            "Failed to clone {}\n> {:?}: {}",
                            repo_url,
                            e.code(),
                            e.to_string(),
                        ));
                }
            };

            let mut languages = Languages::new();
            languages.get_statistics(&["/tmp/repo"], &[".git"], &Config::default());

            let data = make_json(languages);

            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(serde_json::to_string_pretty(&data).unwrap())
                .expect("Failed to render response");

            Ok(response)
        }

        _ => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("BAD REQUEST.\nUsage instruction: /<github_username>/<github_repo>/".to_string()),
    }
}

// For local testing:
// fn main() {
//     let lol = handler(
//         Request::get("https://tokei-now-awqpllhtf.now.sh/?user=mathspy&repo=binary-clock")
//             .body(())
//             .unwrap(),
//     );

//     println!("{}", lol.unwrap().body());
// }
