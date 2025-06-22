use serde::{Deserialize, Serialize};

use crate::sources::Post;

#[derive(Serialize, Deserialize)]
pub struct JsonResponse {
    pub response: Vec<Post>,
}
