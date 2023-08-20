use derive_more::Constructor;
use futures::future::join_all;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde_derive::{Deserialize, Serialize};
use std::future::IntoFuture;
use std::io;
use std::net::SocketAddr;
use stubborn_io::StubbornTcpStream;
use tokio::io::AsyncWriteExt;

#[repr(u8)]
#[derive(Clone, Copy, strum::Display, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
pub enum WorkerMethod {
    KeyGenPrepare = 0x00,
    KeyGenSetCk = 0x01,
    KeyGenCommit = 0x02,

    ProveInit = 0x0F,

    ProveRound1 = 0x10,

    ProveRound2Compute = 0x20,
    ProveRound2Exchange = 0x21,
    ProveRound2Commit = 0x22,

    ProveRound3Prepare = 0x30,
    ProveRound3ComputeTPart1Type1 = 0x31,
    ProveRound3ExchangeTPart1Type1 = 0x32,
    ProveRound3ExchangeW1 = 0x33,
    ProveRound3ComputeAndExchangeTPart1Type3AndPart2 = 0x34,
    ProveRound3ComputeAndExchangeTPart1Type2 = 0x35,
    ProveRound3Commit = 0x36,

    ProveRound4EvaluateW = 0x40,
    ProveRound4EvaluateSigmaOrZ = 0x41,

    ProveRound5Prepare = 0x50,
    ProveRound5Exchange = 0x51,
    ProveRound5Commit = 0x52,

    ProveRound2UpdateZ = 0x80,

    ProveRound3UpdateW1Product = 0x90,
    ProveRound3UpdateT = 0x92,
    ProveRound3GetW1Product = 0x93,
    ProveRound3GetW2ProductDelta = 0x94,
    ProveRound3ComputeW3 = 0x95,
    ProveRound3GetW3 = 0x96,
    ProveRound3GetZ = 0x97,

    ProveRound5Update = 0xA0,
}

#[repr(u8)]
#[derive(Clone, Copy, strum::Display, TryFromPrimitive, IntoPrimitive)]
pub enum WorkerStatus {
    Ok = 0x00,
    HashMismatch = 0x01,
}

pub struct Dispatcher {}

impl Dispatcher {}
