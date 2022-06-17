 use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use serde_derive::Deserialize;

use crate::comment::Comment;
use crate::repository::CommentRepository;
use chrono::DateTime;


#[derive(Debug, Deserialize)]
struct CommentRecord
{
    timestamp: String,
    path: String,
    author_name: String,
    author_email: String,
    text: String,
}


pub fn run(filename: &str, repo: CommentRepository) -> Result<(), Box<dyn Error>>
{
    let file = File::open(filename)?;
    let mut reader = csv::ReaderBuilder::new().delimiter(b',').from_reader(BufReader::new(file));

    println!("\nIMPORTING COMMENTS\n");
    for result in reader.deserialize() {
        let r: CommentRecord = result?;
        let timestamp = DateTime::from(DateTime::parse_from_rfc3339(&r.timestamp).unwrap()); // TODO: there must be a better way
        let author_name: Option<&str> = if r.author_name == "" { None } else { Some(&r.author_name) };
        let author_email: Option<&str> = if r.author_email == "" { None } else { Some(&r.author_email) };
        let mut comment = Comment::new(&r.path, &r.text, author_name, author_email);
        comment.timestamp = timestamp;
        println!("{:?}", comment);
        repo.save_comment(&comment);
    }

    Ok(())
}

