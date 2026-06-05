use rand::{RngExt, distr::Alphanumeric};

pub fn generate_game_id() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}

pub fn generate_admin_code() -> i32 {
    rand::rng().random_range(100_000..1_000_000)
}

pub fn generate_player_code() -> i32 {
    rand::rng().random_range(100_000..1_000_000)
}
