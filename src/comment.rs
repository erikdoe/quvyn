use serde_derive::*;
use uuid::Uuid;

use crate::markdown::md_to_html;
use crate::utils;

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Comment
{
    pub id: Uuid,
    pub idh: u64,
    pub path: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub content: String,
    pub content_html: String,
}


impl Comment
{
    pub fn new(path: &str, content: &str, author_name: Option<&str>, author_email: Option<&str>) -> Comment {
        let id = Uuid::new_v4();
        Comment {
            id,
            idh: utils::calculate_hash(&id),
            path: path.to_owned(),
            author_name: author_name.map(|n| n.to_owned()),
            author_email: author_email.map(|e| e.to_owned()),
            content: content.to_owned(),
            content_html: md_to_html(content),
        }
    }
}
