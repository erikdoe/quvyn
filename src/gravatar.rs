use gravatar::Gravatar;

pub fn gravatar_url_for_email(email: Option<&str>) -> String {
    match email {
        Some(email) => Gravatar::new(&email).image_url().to_string(),
        None =>Gravatar::new(&"https://secure.gravatar.com/generic").image_url().to_string()
    }
}
