use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use rand::random;

pub fn generate_admin_token() -> String {
    let bytes: [u8; 32] = random();
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn hash_admin_token(token: &str) -> String {
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn bearer_token(value: Option<&str>) -> Option<&str> {
    value?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}
