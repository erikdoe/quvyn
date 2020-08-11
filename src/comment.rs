use serde_derive::*;
use uuid::Uuid;

use crate::markdown::md_to_html;
use crate::gravatar::gravatar_url_for_email;
use crate::utils::calculate_hash;

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Comment
{
    pub id: Uuid,
    pub idh: u64,
    pub path: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_gravatar: String,
    pub content: String,
    pub content_html: String,
}


impl Comment
{
    pub fn new(path: &str, content: &str, author_name: Option<&str>, author_email: Option<&str>) -> Comment {
        let id = Uuid::new_v4();
        Comment {
            id,
            idh: calculate_hash(&id),
            path: path.to_owned(),
            author_name: author_name.map(|n| n.to_owned()),
            author_email: author_email.map(|e| e.to_owned()),
            author_gravatar: gravatar_url_for_email(author_email),
            content: content.to_owned(),
            content_html: md_to_html(content),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_gravatar_url_for_email() {
        let comment = Comment::new("", "", Some("Joe Bloggs"), Some("joe@example.org"));
        assert_eq!(comment.author_gravatar, "https://secure.gravatar.com/avatar/4efa9f57993995b10a03a415f0e62883")
    }

    #[test]
    fn adds_gravatar_url_when_no_email_given() {
        let comment = Comment::new("", "", Some("Joe Bloggs"), None);
        assert_eq!(comment.author_gravatar, "https://secure.gravatar.com/generic")
    }

    #[test]
    fn adds_html_for_content() {
        let comment = Comment::new("", "_foo_", None, None);
        assert_eq!(comment.content_html, "<p><em>foo</em></p>\n");
    }
}
