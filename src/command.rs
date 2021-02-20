use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Set(String, String),
    Get(String),
    Rm(String),
}
