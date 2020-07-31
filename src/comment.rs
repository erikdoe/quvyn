use serde_derive::*;

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Comment
{
    pub path: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub content: String,
}


impl Comment
{
    pub fn new(path: &str, content: &str) -> Comment {
        Comment {
            path: path.to_owned(),
            author_name: None,
            author_email: None,
            content: content.to_owned(),
        }
    }

}
