use futures::io::ReadHalf;
use halo2_proofs::distributed::dispatcher::{Taskable, WorkerMethod, WorkerStatus};
use halo2_proofs::distributed::plonk::permutation::keygen::KeygenTask;
use halo2_proofs::distributed::utils::CastSlice;
use once_cell::sync::Lazy;
use std::{io, net::SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpListener;

pub static WORKERS: Lazy<[SocketAddr; 2]> = Lazy::new(|| {
    [
        "127.0.0.1:8081".parse().unwrap(),
        "127.0.0.1:8082".parse().unwrap(),
    ]
});

pub struct PlonkImplInner {}

#[derive(Clone, Debug)]
pub struct Worker {
    id: usize,
    // inner: Arc<PlonkImplInner>,
}

impl Worker {
    pub fn new(id: usize) -> Self {
        assert!(
            id < 2,
            "Only two workers allowed, got id {} which is > 2.",
            id
        );
        Self { id }
    }

    pub async fn start(&self) -> io::Result<()> {
        let addr = WORKERS[self.id];
        let name = format!("worker_{}", self.id);

        let listener = TcpListener::bind(addr).await.unwrap();

        println!("{} listening on: {}", name, addr);

        while let Ok((mut stream, addr)) = listener.accept().await {
            let peer_addr = addr.ip();
            println!("Connection from {}", peer_addr);

            // Set nodelay to always just send whatever data is available.
            stream.set_nodelay(true).unwrap();

            let this_worker = self.clone();

            tokio::spawn(async move {
                loop {
                    let (read, write) = stream.split();

                    let mut req = BufReader::new(read);
                    let res = BufWriter::new(write);

                    // This should not just unpack the method, it should unpack the entire type.
                    match req.read_u8().await {
                        Ok(method) => {
                            let method: WorkerMethod = method.try_into().unwrap();
                            println!("{} -> {}: {}", peer_addr, addr, method);
                            this_worker.handle(method, req, res).await.unwrap();
                        }
                        Err(_) => {
                            println!("Connection from {} disconnected prematurely", addr.ip());
                            break;
                        }
                    }
                }
                Ok::<(), io::Error>(())
            });
        }

        Ok(())
    }

    async fn handle<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
        &self,
        method: WorkerMethod,
        req: BufReader<R>,
        res: BufWriter<W>,
    ) -> io::Result<()> {
        match method {
            WorkerMethod::KeyGenCommit => self.commit(req, res).await,
        }
    }

    async fn commit<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
        &self,
        mut req: BufReader<R>,
        mut res: BufWriter<W>,
    ) -> io::Result<()> {
        // Allocate an empty buffer
        let mut buf = [0u8; core::mem::size_of::<KeygenTask>()];

        // Suck down the rest of the payload
        req.read_exact(&mut buf).await.unwrap();

        // Cast the buffer to the distributed request type.
        let buf = buf.cast::<KeygenTask>();

        // Handle the payload

        res.write_u8(WorkerStatus::Ok as u8).await?;
        res.flush().await?;
        Ok(())
    }
}

fn help() -> &'static str {
    "usage: worker <worker_id|usize>"
}

#[tokio::main]
async fn main() {
    let worker_id: usize = match std::env::args().nth(1) {
        Some(worker_id) => worker_id
            .parse()
            .expect("Invalid worker id provided, must be an integer"),
        None => panic!("{}", help()),
    };
    let w = Worker::new(worker_id);
    w.start().await.unwrap();
}
