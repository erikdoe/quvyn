use serde_derive::*;

use crate::utils;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment
{
    pub topic_id: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub content: String
}


impl Comment
{
    pub fn new(topic_id: &str, content: &str) -> Comment {
        Comment {
            topic_id: topic_id.to_owned(),
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
                "topic_id": "/test/",
                "content": "This is a comment."
            }"#);

        assert_eq!("/test/", comment.topic_id);
        assert_eq!("This is a comment.", comment.content);
    }

    #[test]
    fn creates_json_for_comment() {
        let comment = Comment::new("/test/", "This is a comment.");

        let json = comment.to_json();

        // make sure it looks like pretty printed json
        assert_eq!("{\n  \"topic_id\": \"/te", &json[..20]);
    }
}