use halo2_proofs_distributed::dispatcher::{WorkerMethod, WorkerStatus};
use once_cell::sync::Lazy;
use std::{io, net::SocketAddr, sync::Arc};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpListener;

pub static WORKERS: Lazy<[SocketAddr; 2]> = Lazy::new(|| {
    [
        "127.0.0.1:8081".parse().unwrap(),
        "127.0.0.1:8082".parse().unwrap(),
    ]
});

pub struct PlonkImplInner {}

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

            tokio::spawn(async move {
                loop {
                    let (read, write) = stream.split();

                    let mut req = BufReader::new(read);
                    let mut res = BufWriter::new(write);

                    match req.read_u8().await {
                        Ok(method) => {
                            let method: WorkerMethod = method.try_into().unwrap();
                            println!("{} -> {}: {}", peer_addr, addr, method);
                            res.write_u8(WorkerStatus::Ok as u8).await?;
                            res.flush().await?;
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
