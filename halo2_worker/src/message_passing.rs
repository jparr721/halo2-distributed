//! This is just a simple module for holding worker-related messages for use by
//! other crates in this project.

use serde_derive::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct Call {
    name: String,
    args: Vec<String>,
}

impl Call {
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
