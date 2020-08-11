use gotham::handler::HandlerFuture;
use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline;
use gotham::router::builder::{build_router, DrawRoutes};
use gotham::router::builder::DefineSingleRoute;
use gotham::router::Router;
use gotham::state::{FromState, State};
use gotham_derive::*;
use hyper::{Body, Response, StatusCode, Uri};
use hyper::rt::Future;
use serde_derive::*;
use uuid::Uuid;

use crate::comment::Comment;
use crate::gotham_json::{create_json_response, JSONBody};
use crate::markdown::md_to_html;
use crate::repository::CommentRepository;

pub fn run(repo: CommentRepository, addr: String) {
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router(repo));
}

pub fn router(repo: CommentRepository) -> Router {
    let middleware = StateMiddleware::new(repo);
    let pipeline = pipeline::single_middleware(middleware);
    let (chain, pipelines) = pipeline::single::single_pipeline(pipeline);
    build_router(chain, pipelines, |route| {
        route.get("/ping")
            .to(get_ping);
        route.get("/comments")
            .with_query_string_extractor::<CommentsQueryStringExtractor>()
            .to(get_comments);
        route.post("/comments")
            .to(post_comment);
        route.get("/comments/:id")
            .with_path_extractor::<IdParam>()
            .to(get_comment);
        route.post("/preview")
            .to(post_preview);
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
        None => create_response(&state, StatusCode::NOT_FOUND, mime::TEXT_PLAIN, "Not found")
    };
    (state, response)
}


#[derive(Deserialize)]
struct CommentPostDoc {
    path: String,
    content: String,
    #[serde(rename = "authorName")]
    author_name: Option<String>,
    #[serde(rename = "authorEmail")]
    author_email: Option<String>,
}

impl CommentPostDoc {
    fn to_comment(&self) -> Comment {
        Comment::new(&self.path, &self.content,
                     self.author_name.as_ref().map(String::as_str),
                     self.author_email.as_ref().map(String::as_str)) // TODO: better way?
    }
}

fn post_comment(state: State) -> Box<HandlerFuture> {
    Box::new(state.json::<CommentPostDoc>().and_then(|(state, doc)| {
        let repo = CommentRepository::borrow_from(&state);
        let comment = doc.to_comment();
        repo.save_comment(&comment);
        let (code, body) = (StatusCode::CREATED, "Created comment\n");
        let mut response = create_response(&state, code, mime::TEXT_PLAIN, body);
        let location = format!("{}/{}", Uri::borrow_from(&state), comment.id);
        response.headers_mut().insert("Location", location.parse().unwrap());
        Ok((state, response))
    }))
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
    path: String,
    #[serde(rename = "contentHtml")]
    content_html: String,
    #[serde(rename = "authorName")]
    author_name: Option<String>,
    #[serde(rename = "authorGravatar")]
    author_gravatar: String,

}

impl CommentDisplayDoc {
    pub fn from_comment(comment: &Comment) -> CommentDisplayDoc {
        CommentDisplayDoc {
            idh: comment.idh,
            path: comment.path.clone(),
            content_html: comment.content_html.clone(),
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
    content: String,
}

fn post_preview(state: State) -> Box<HandlerFuture> {
    Box::new(state.json::<CommentPreviewDoc>().and_then(|(state, doc)| {
        let body = md_to_html(&doc.content);
        let response = create_response(&state, StatusCode::OK, mime::TEXT_HTML, body);
        Ok((state, response))
    }))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_comment_from_dto() {
        let dto = CommentPostDoc {
            path: String::from("/a/"),
            content: String::from("First comment"),
            author_name: Some(String::from("Joe Bloggs")),
            author_email: Some(String::from("joe@example.org")),
        };
        let comment = dto.to_comment();
        assert_eq!(comment.path, "/a/");
        assert_eq!(comment.content, "First comment");
        assert_eq!(comment.author_name, Some(String::from("Joe Bloggs")));
        assert_eq!(comment.author_email, Some(String::from("joe@example.org")));
    }

    #[test]
    fn creates_dto_from_comment() {
        let comment = Comment::new("/t/", "Nice work!", Some("Joe Bloggs"), Some("joe@example.org"));
        let dto = CommentDisplayDoc::from_comment(&comment);

        assert_eq!(dto.idh, comment.idh);
        assert_eq!(dto.path, comment.path);
        assert_eq!(dto.content_html, comment.content_html);
        assert_eq!(dto.author_name, comment.author_name);
    }
}

