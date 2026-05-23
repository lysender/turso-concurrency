use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostDto {
    pub id: i64,
    pub title: String,
    pub content: String,
}
