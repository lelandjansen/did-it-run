pub mod emails;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, PartialEq, Serialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
}
