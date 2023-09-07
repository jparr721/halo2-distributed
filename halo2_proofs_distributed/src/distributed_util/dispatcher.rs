//! Dispatcher api
use futures::future::join_all;
use halo2curves::{
    bn256::{Bn256, G1Affine},
    CurveAffine,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use std::{io, net::SocketAddr, thread, time::Duration};
use stubborn_io::{tokio::StubbornIo, StubbornTcpStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    plonk::{
        create_domain, permutation::Argument, permutation::ProvingKey, permutation::VerifyingKey,
    },
    poly::{commitment::Params, kzg::commitment::ParamsKZG, EvaluationDomain},
};

use super::{net::to_bytes, plonk::permutation::keygen::KeygenTaskKZG, utils::CastSlice};

pub static WORKERS: Lazy<[SocketAddr; 1]> = Lazy::new(|| {
    [
        "127.0.0.1:8081".parse().unwrap(),
        // "127.0.0.1:8082".parse().unwrap(),
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
    pub async fn new() -> Self {
        let mut dispatcher = Dispatcher {
            workers: Vec::new(),
        };

        // Set up the active worker connections
        dispatcher.init_worker_pool().await;
        dispatcher
    }

    /// Initiates the distributed keygen operation.
    pub async fn keygen<'params, C: CurveAffine, P: Params<'params, C>>(
        &mut self,
        params: &'params P,
        domain: &'params EvaluationDomain<C::Scalar>,
        p: &'params Argument,
        mapping: &Vec<Vec<(usize, usize)>>,
    ) -> Vec<C> {
        let task = KeygenTaskKZG::<C, P>::new(params, domain, p, mapping.clone());
        let commitments = join_all(self.workers.iter_mut().map(|worker| async {
            // Dump the method over
            worker.write_u8(WorkerMethod::KeyGen as u8).await.unwrap();

            // Drop the payload
            worker
                .write_all(to_bytes(task.clone()).as_slice())
                .await
                .unwrap();

            // Flush the buffer
            worker.flush().await.unwrap();

            // Prepare to receive the commitments
            let mut cs = [0u8; core::mem::size_of::<G1Affine>()];

            // Read the output from the worker
            worker.read_exact(&mut cs).await.unwrap();

            // NOTE: This [0] will be removed later when we recieve from multiple sources
            // This method will need to handle proper ordering as well of the commitments
            cs.cast::<C>()[0]
        }))
        .await;

        commitments
    }

    async fn init_worker_pool(&mut self) {
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
        .await
    }
}
