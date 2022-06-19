use std::pin::Pin;
use chrono::{DateTime, Utc};
use futures_util::{future, FutureExt};
use gotham::handler::HandlerFuture;
use gotham::handler::FileOptions;
use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::{new_pipeline, single_pipeline};
use gotham::router::builder::{build_router, DrawRoutes};
use gotham::router::builder::DefineSingleRoute;
use gotham::router::Router;
use gotham::state::{FromState, State};
use gotham::prelude::*;
use gotham::hyper::{Body, Response, StatusCode, Uri};
use serde_derive::*;
use uuid::Uuid;

use crate::comment::Comment;
use crate::gotham_json::{create_json_response, create_json_response_with_headers, get_json_body};
use crate::markdown::md_to_html;
use crate::repository::CommentRepository;
use crate::gotham_cors::CorsMiddleware;

pub fn run(repo: CommentRepository, app_path: &str, addr: &str, origin: &Option<String>) {
    println!("Listening for requests at http://{}", addr);
    let _ = gotham::start(addr.to_string(), router(app_path, origin, repo));
}

pub fn router(app_path: &str, origin: &Option<String>, repo: CommentRepository) -> Router {
    let pipeline1 = new_pipeline()
        .add(StateMiddleware::new(repo))
        .add(CorsMiddleware::new(origin))  // TODO: should only add middleware when needed
        .build();
    let (chain, pipelines) = single_pipeline(pipeline1);
    build_router(chain, pipelines, |route| {
        route.get("/ping")
            .to(get_ping);
        route.get("/comments")
            .with_query_string_extractor::<CommentsQueryStringExtractor>()
            .to(get_comments);
        route.get("/comments/:id")
            .with_path_extractor::<IdParam>()
            .to(get_comment);
        route.delete("/comments/:id")
            .with_path_extractor::<IdParam>()
            .to(delete_comment);
        route.options("/comments/:id")
            .to(cors_preflight);
        route.post("/comments")
            .to(post_comment);
        route.options("/comments")
            .to(cors_preflight);
        route.post("/preview")
            .to(post_preview);
        route.options("/preview")
            .to(cors_preflight);
        route.get("/favicon.png")
            .to_file(&format!("{}/favicon.png", app_path));
        route.get("/app/*")
            .to_dir(FileOptions::new(app_path)
                        .with_cache_control("no-cache")
                        .with_gzip(true)
                        .build(),
            );
    })
}


#[derive(Serialize)]
struct PingResponse {
    status: String,
}

fn get_ping(state: State) -> (State, Response<Body>) {
    let response_obj = PingResponse { status: "ok".to_owned() };
    let response = create_json_response(&state, StatusCode::OK, &response_obj).unwrap();
    (state, response)
}


#[derive(Deserialize, StateData, StaticResponseExtender)]
struct IdParam {
    id: Uuid,
}

fn get_comment(mut state: State) -> (State, Response<Body>) {
    let p = IdParam::take_from(&mut state);
    let repository = CommentRepository::borrow_from(&state);

    let response = match repository.comment_with_id(p.id) {
        Some(comment) => create_json_response(&state, StatusCode::OK, &comment).unwrap(),
        None => create_response(&state, StatusCode::NOT_FOUND, mime::TEXT_PLAIN, "Comment not found")
    };
    (state, response)
}


fn delete_comment(mut state: State) -> (State, Response<Body>) {
    let p = IdParam::take_from(&mut state);
    let repository = CommentRepository::borrow_from(&state);
    let response = if let Some(comment) = repository.comment_with_id(p.id) {
        repository.delete_comment(&comment); // TODO: error handling?
        create_response(&state, StatusCode::OK, mime::TEXT_PLAIN, "Deleted comment")
    } else {
        create_response(&state, StatusCode::NOT_FOUND, mime::TEXT_PLAIN, "Comment not found")
    };
    (state, response)
}


#[derive(Deserialize)]
struct CommentPostDoc {
    path: String,
    text: String,
    #[serde(rename = "authorName")]
    author_name: Option<String>,
    #[serde(rename = "authorEmail")]
    author_email: Option<String>,
}

impl CommentPostDoc {
    fn to_comment(&self) -> Comment {
        Comment::new(&self.path, &self.text,
                     self.author_name.as_ref().map(String::as_str),
                     self.author_email.as_ref().map(String::as_str)) // TODO: better way?
    }
}

// fn post_comment(state: State) -> Box<HandlerFuture> {
//     Box::new(state.json::<CommentPostDoc>().and_then(|(state, doc)| {
//         let comment = doc.to_comment();
//         let response = if comment.text_html == "" {
//             create_response(&state, StatusCode::BAD_REQUEST, mime::TEXT_PLAIN, "No visible text")
//         } else {
//             CommentRepository::borrow_from(&state).save_comment(&comment);
//             let location = format!("{}/{}", Uri::borrow_from(&state), comment.id);
//             let headers = vec![("Location", location)].into_iter().collect(); // TODO: better way?
//             let resp_doc = CommentDisplayDoc::from_comment(&comment);
//             create_json_response_with_headers(&state, StatusCode::CREATED, headers, &resp_doc).unwrap()
//         };
//         Ok((state, response))
//     }))
// }

fn post_comment(mut state: State) -> Pin<Box<HandlerFuture>> {
    let f = get_json_body::<CommentPostDoc>(&mut state).then(|result| {
        let response = match result {
            Ok(doc) => {
                let comment = doc.to_comment();
                if comment.text_html == "" {
                    create_response(&state, StatusCode::BAD_REQUEST, mime::TEXT_PLAIN, "No visible text")
                } else {
                    CommentRepository::borrow_from(&state).save_comment(&comment);
                    let location = format!("{}/{}", Uri::borrow_from(&state), comment.id);
                    let headers = vec![("Location", location)].into_iter().collect(); // TODO: better way?
                    let resp_doc = CommentDisplayDoc::from_comment(&comment);
                    create_json_response_with_headers(&state, StatusCode::CREATED, headers, &resp_doc).unwrap()
                }
            }
            Err(_) => {
                create_response(&state, StatusCode::BAD_REQUEST, mime::TEXT_PLAIN, "Invalid JSON")
            }
        };
        future::ok((state, response))
    });
    f.boxed()
}



#[derive(Deserialize, StateData, StaticResponseExtender)]
struct CommentsQueryStringExtractor {
    p: Option<String>,
}

#[derive(Serialize, Clone)]
struct CommentListWrapper {
    comments: Vec<CommentDisplayDoc>
}

#[derive(Serialize, Clone)]
struct CommentDisplayDoc {
    idh: u64,
    timestamp: DateTime<Utc>,
    path: String,
    #[serde(rename = "textHtml")]
    text_html: String,
    #[serde(rename = "authorName")]
    author_name: Option<String>,
    #[serde(rename = "authorGravatar")]
    author_gravatar: String,

}

impl CommentDisplayDoc {
    pub fn from_comment(comment: &Comment) -> CommentDisplayDoc {
        CommentDisplayDoc {
            idh: comment.idh,
            timestamp: comment.timestamp,
            path: comment.path.clone(),
            text_html: comment.text_html.clone(),
            author_name: comment.author_name.clone(),
            author_gravatar: comment.author_gravatar.clone(),
        }
    }
}

fn get_comments(mut state: State) -> (State, Response<Body>) {
    let query_param = CommentsQueryStringExtractor::take_from(&mut state);
    let repository = CommentRepository::borrow_from(&state);

    let comments = match query_param.p {
        Some(p) => repository.comments_for_path(&p),
        None => repository.all_comments()
    };
    let display_comments = comments.iter().map(CommentDisplayDoc::from_comment).collect();
    let wrapper = CommentListWrapper { comments: display_comments };
    let response = create_json_response(&state, StatusCode::OK, &wrapper).unwrap();
    (state, response)
}


#[derive(Deserialize)]
struct CommentPreviewDoc {
    text: String,
}

// fn post_preview(state: State) -> Box<HandlerFuture> {
//     Box::new(state.json::<CommentPreviewDoc>().and_then(|(state, doc)| {
//         let body = md_to_html(&doc.text);
//         let response = create_response(&state, StatusCode::OK, mime::TEXT_HTML, body);
//         Ok((state, response))
//     }))
// }

fn post_preview(mut state: State) -> Pin<Box<HandlerFuture>> {
    let f = get_json_body::<CommentPreviewDoc>(&mut state).then(|result| {
        let response = match result {
            Ok(doc) => {
                let body = md_to_html(&doc.text);
                create_response(&state, StatusCode::OK, mime::TEXT_HTML, body)
            },
            Err(_) => {
                create_response(&state, StatusCode::BAD_REQUEST, mime::TEXT_PLAIN, "Invalid JSON")
            }
        };
        future::ok((state, response))
    });
    f.boxed()
}


fn cors_preflight(state: State) -> (State, Response<Body>) {
    let response = create_response(&state, StatusCode::NO_CONTENT, mime::TEXT_PLAIN, "");
    (state, response)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_comment_from_dto() {
        let dto = CommentPostDoc {
            path: String::from("/a/"),
            text: String::from("First comment"),
            author_name: Some(String::from("Joe Bloggs")),
            author_email: Some(String::from("joe@example.org")),
        };
        let comment = dto.to_comment();
        assert_eq!(comment.path, "/a/");
        assert_eq!(comment.text, "First comment");
        assert_eq!(comment.author_name, Some(String::from("Joe Bloggs")));
        assert_eq!(comment.author_email, Some(String::from("joe@example.org")));
    }

    #[test]
    fn creates_dto_from_comment() {
        let comment = Comment::new("/t/", "Nice work!", Some("Joe Bloggs"), Some("joe@example.org"));
        let dto = CommentDisplayDoc::from_comment(&comment);

        assert_eq!(dto.idh, comment.idh);
        assert_eq!(dto.path, comment.path);
        assert_eq!(dto.text_html, comment.text_html);
        assert_eq!(dto.author_name, comment.author_name);
    }
}

