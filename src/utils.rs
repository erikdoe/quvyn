use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use serde::{Deserialize, Serialize};
use serde_json;

pub fn from_json<'a, T>(s: &'a str) -> T where T: Deserialize<'a> {
    serde_json::from_str(s).expect("Failed to parse JSON")
}

pub fn to_json<T>(val: T) -> String where T: Serialize {
    let result = serde_json::to_string_pretty(&val);
    result.ok().expect("Failed to produce JSON")
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
