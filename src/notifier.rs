use crate::sendmail;
use crate::comment::Comment;

#[derive(Clone)]
pub struct Notifier
{
    recipient: String
}

impl Notifier
{
    pub fn new(recipient: &str) -> Self {
        Notifier {
            recipient: recipient.to_string()
        }
    }

    pub fn notify(&self, comment: &Comment)
    {
        let result = sendmail::send(&self.recipient, &"New comment posted", &format!("{:?}", comment));
        if let Err(message) = result {
            println!("Error when sending mail: {}", message);
        }
    }

}
