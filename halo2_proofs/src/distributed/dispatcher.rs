//! Dispatcher api
use futures::future::join_all;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use std::{io, net::SocketAddr, thread, time::Duration};
use stubborn_io::{tokio::StubbornIo, StubbornTcpStream};
use tokio::net::TcpStream;

pub static WORKERS: Lazy<[SocketAddr; 2]> = Lazy::new(|| {
    [
        "127.0.0.1:8081".parse().unwrap(),
        "127.0.0.1:8082".parse().unwrap(),
    ]
});

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, strum::Display, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize,
)]
pub enum WorkerMethod {
    KeyGen = 0x00,
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

#[allow(missing_debug_implementations)]
pub struct Dispatcher {
    pub workers: Vec<StubbornIo<TcpStream, SocketAddr>>,
}

impl Dispatcher {
    pub async fn keygen(&self) {}

    pub async fn init_worker_pool(&mut self) {
        self.workers = join_all(WORKERS.iter().map(|worker| async move {
            let stream = loop {
                match StubbornTcpStream::connect(worker.clone()).await {
                    Ok(stream) => break stream,
                    Err(_) => {
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            };
            stream.set_nodelay(true).unwrap();
            stream
        }))
        .await;
    }
}
