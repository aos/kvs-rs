use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Command {
    Set(String, String), // (key, value, timestamp)
    Rm(String),
}
