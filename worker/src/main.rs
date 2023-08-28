use halo2_proofs::arithmetic::{parallelize, Field};
use halo2_proofs::distributed::dispatcher::{WorkerMethod, WORKERS};
use halo2_proofs::distributed::net::to_bytes;
use halo2_proofs::distributed::plonk::permutation::keygen::KeygenTaskKZG;
use halo2_proofs::distributed::utils::CastSlice;
use halo2_proofs::halo2curves::bn256::G1Affine;
use halo2_proofs::halo2curves::group::Curve;
use halo2_proofs::poly::commitment::Blind;
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::timer;
use std::time::Instant;
use std::{io, net::SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpListener;

#[derive(Clone, Debug)]
pub struct WorkerKZG {
    id: usize,
}

impl WorkerKZG {
    pub fn new(id: usize) -> Self {
        assert!(
            id < WORKERS.len(),
            "Only {} workers allowed, got id {} which is > {}.",
            WORKERS.len(),
            id,
            WORKERS.len(),
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
            WorkerMethod::KeyGen => self.keygen(req, res).await,
        }
    }

    async fn keygen<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
        &self,
        mut req: BufReader<R>,
        mut res: BufWriter<W>,
    ) -> io::Result<()> {
        // Allocate an empty buffer
        let mut task = [0u8; core::mem::size_of::<KeygenTaskKZG<G1Affine>>()];

        // Suck down the rest of the payload
        req.read_exact(&mut task).await.unwrap();

        // Cast the buffer to the distributed request type.
        let task = &task.cast::<KeygenTaskKZG<G1Affine>>()[0];

        // Handle the payload
        // Compute [omega^0, omega^1, ..., omega^{params.n - 1}]
        let mut omega_powers = vec![task.zero(); task.params.n() as usize];
        timer!("worker commit omega_powers", {
            let omega = task.domain.get_omega();
            parallelize(&mut omega_powers, |o, start| {
                let mut cur = omega.pow_vartime(&[start as u64]);
                for v in o.iter_mut() {
                    *v = cur;
                    cur *= &omega;
                }
            })
        });

        // Compute [omega_powers * \delta^0, omega_powers * \delta^1, ..., omega_powers * \delta^m]
        let mut deltaomega = vec![omega_powers; task.p.ncolumns()];
        timer!("keygen.rs deltaomega", {
            parallelize(&mut deltaomega, |o, start| {
                let mut cur = task.delta().pow_vartime(&[start as u64]);
                for omega_powers in o.iter_mut() {
                    for v in omega_powers {
                        *v *= &cur;
                    }
                    cur *= &task.delta();
                }
            });
        });

        // Computes the permutation polynomial based on the permutation
        // description in the assembly.
        let mut permutations = vec![task.domain.empty_lagrange(); task.p.ncolumns()];
        timer!("keygen.rs permutations", {
            parallelize(&mut permutations, |o, start| {
                for (x, permutation_poly) in o.iter_mut().enumerate() {
                    let i = start + x;
                    for (j, p) in permutation_poly.iter_mut().enumerate() {
                        let (permuted_i, permuted_j) = task.mapping[i][j];
                        *p = deltaomega[permuted_i][permuted_j];
                    }
                }
            });
        });

        // TIME! This is the rate-limiting step
        // Pre-compute commitments for the URS.
        let mut commitments = Vec::with_capacity(task.p.ncolumns());
        for permutation in &permutations {
            // Compute commitment to permutation polynomial
            commitments.push(
                task.params
                    .commit_lagrange(permutation, Blind::default())
                    .to_affine(),
            );
        }

        res.write_all(to_bytes(commitments).as_slice()).await?;
        // res.write_u8(WorkerStatus::Ok as u8).await?;
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
    let w = WorkerKZG::new(worker_id);
    w.start().await.unwrap();
}
