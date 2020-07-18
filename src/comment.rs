use serde_derive::*;

use crate::utils;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment
{
    pub path: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub content: String
}


impl Comment
{
    pub fn new(path: &str, content: &str) -> Comment {
        Comment {
            path: path.to_owned(),
            author_name: None,
            author_email: None,
            content: content.to_owned()
        }
    }

    pub fn from_json(json: &str) -> Comment {
        utils::from_json(json)
    }

    pub fn to_json(&self) -> String {
        utils::to_json(self)
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_comment_from_json() {
        let comment = Comment::from_json(r#"{
                "path": "/test/",
                "content": "This is a comment."
            }"#);

        assert_eq!("/test/", comment.path);
        assert_eq!("This is a comment.", comment.content);
    }

    #[test]
    fn creates_json_for_comment() {
        let comment = Comment::new("/test/", "This is a comment.");

        let json = comment.to_json();

        // make sure it looks like pretty printed json
        assert_eq!("{\n  \"path\": \"/t", &json[..15]);
    }
}