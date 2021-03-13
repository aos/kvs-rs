use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Command {
    Set(String, String), // (key, value)
    Rm(String),
}

#[derive(Serialize, Deserialize)]
pub enum Request {
    Get(String),
    Set(String, String),
    Rm(String),
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    OK(String),
    Error(String),
    NotFound,
}
