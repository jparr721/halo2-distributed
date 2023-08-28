//! Dispatcher api
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde_derive::{Deserialize, Serialize};

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, strum::Display, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize,
)]
pub enum WorkerMethod {
    KeyGenCommit = 0x00,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, strum::Display, TryFromPrimitive, IntoPrimitive)]
pub enum WorkerStatus {
    Ok = 0x00,
    ErrorInvalidMethod = 0x01,
    ErrorUnkown = 0x02,
}

pub trait Taskable {
    fn method(&self) -> WorkerMethod;
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(&self, bytes: Vec<u8>) -> Self;
}
