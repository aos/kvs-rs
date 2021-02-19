use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Operation {
    Set(String, String),
    Get(String),
    Rm(String),
}

#[derive(Serialize, Deserialize)]
pub struct Command(pub Operation);
