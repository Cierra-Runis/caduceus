use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    username: String,
    nickname: String,
    password: String,
}
