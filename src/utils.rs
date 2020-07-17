use serde::Deserialize;
use serde_json;

pub fn from_json<'a, T>(s: &'a str) -> T where T: Deserialize<'a> {
    serde_json::from_str(s).expect("Failed to parse JSON")
}
