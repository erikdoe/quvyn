use chrono::{DateTime, Duration, Utc};
use serde_derive::*;
use uuid::Uuid;

use crate::gravatar::gravatar_url_for_email;
use crate::markdown::md_to_html;
use crate::utils::calculate_hash;

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Comment
{
    pub id: Uuid,
    pub idh: u64,
    pub timestamp: DateTime<Utc>,
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
        let mut timestamp = Utc::now();
        timestamp = timestamp - Duration::microseconds(timestamp.timestamp_subsec_micros() as i64);
        Comment {
            id,
            idh: calculate_hash(&id),
            timestamp,
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
    fn adds_current_time_with_limited_precision() {
        let comment = Comment::new("", "content", None, None);
        assert_eq!(0, (Utc::now() - comment.timestamp).num_seconds()); // TODO: not ideal...
        assert_eq!(0, comment.timestamp.timestamp_subsec_micros());
    }

    #[test]
    fn adds_gravatar_url_for_email() {
        let comment = Comment::new("", "", Some("Joe Bloggs"), Some("joe@example.org"));
        assert_eq!("https://secure.gravatar.com/avatar/4efa9f57993995b10a03a415f0e62883", comment.author_gravatar)
    }

    #[test]
    fn adds_gravatar_url_when_no_email_given() {
        let comment = Comment::new("", "", Some("Joe Bloggs"), None);
        assert_eq!("https://secure.gravatar.com/avatar/4988a16beb097f6c7ec78816872ddd13", comment.author_gravatar)
    }

    #[test]
    fn adds_html_for_content() {
        let comment = Comment::new("", "_foo_", None, None);
        assert_eq!("<p><em>foo</em></p>", comment.content_html.trim());
    }
}
