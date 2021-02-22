use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Set(String, String, u64), // (key, value, timestamp)
    Get(String),
    Rm(String),
}
