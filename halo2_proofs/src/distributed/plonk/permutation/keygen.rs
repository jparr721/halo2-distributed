//! Keygen Definitions

use serde_derive::{Deserialize, Serialize};

use crate::distributed::{
    dispatcher::{Taskable, WorkerMethod},
    net::{from_bytes, to_bytes},
};

/// Distributed request to perform keygen
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct KeygenTask {}

impl Taskable for KeygenTask {
    fn method(&self) -> WorkerMethod {
        WorkerMethod::KeyGenCommit
    }

    fn to_bytes(&self) -> Vec<u8> {
        to_bytes(self)
    }

    fn from_bytes(&self, bytes: Vec<u8>) -> Self {
        from_bytes(&bytes)
    }
}
