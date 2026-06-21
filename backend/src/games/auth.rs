use rand::random;
use sha2::{Digest, Sha256};

pub fn generate_admin_token() -> String {
    let bytes: [u8; 32] = random();
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn generate_player_token() -> String {
    generate_admin_token()
}

pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn hash_admin_token(token: &str) -> String {
    hash_token(token)
}

pub fn bearer_token(value: Option<&str>) -> Option<&str> {
    value?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_hash_is_sha256_hex() {
        let hash = hash_token("secret");

        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b"
        );
    }
}
