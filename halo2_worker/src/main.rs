use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_util::codec::{BytesCodec, Decoder};

struct Worker {
    listener: TcpListener,
}

impl Worker {
    async fn new(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            listener: TcpListener::bind(&addr).await?,
        })
    }

    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Listening on: {}", self.listener.local_addr()?);
        loop {
            let (socket, _) = self.listener.accept().await?;
            tokio::spawn(async move {
                let mut framed = BytesCodec::new().framed(socket);

                while let Some(message) = framed.next().await {
                    println!("Received message: {:?}", message);
                }
            });
        }
    }
}

struct WorkOrder {
    // The name of the request
    name: String,

    // The arguments to the request
    args: Vec<u8>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Worker::new("127.0.0.1:8000").await?.run().await?;

    Ok(())
}
