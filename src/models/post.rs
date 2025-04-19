use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a tweet post with user information and content
#[derive(Serialize, Deserialize, ToSchema)]
pub struct Post {
    /// Username of the person posting the tweet
    pub user: String,
    /// Content of the tweet
    pub content: String,
}
