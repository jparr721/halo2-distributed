use futures::SinkExt;
use serde_derive::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};
struct Worker {
    listener: TcpListener,
}

#[derive(Serialize, Deserialize, Debug)]
struct Call {
    name: String,
    args: Vec<String>,
}

impl Call {
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
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
            let (socket, connection) = self.listener.accept().await?;
            println!("received connection from {:?}", connection);
            let mut lines = Framed::new(socket, LinesCodec::new());

            // Convert the framed stream into JSON call
            while let Some(result) = lines.next().await {
                match result {
                    Ok(line) => {
                        let call: Call = match serde_json::from_slice(line.as_bytes()) {
                            Ok(call) => call,
                            Err(e) => {
                                let message = format!("Error parsing fn call: {:?}", e.to_string());
                                println!("{}", message);
                                lines.send(message).await.unwrap();
                                continue;
                            }
                        };
                        println!("Received fn call: {:?}", call);
                        lines.send(call.serialize().unwrap()).await.unwrap();
                    }
                    Err(e) => println!("error decoding {:?}", e),
                }
            }
        }
    }

    fn handle_fn_call(&self, call: &Call) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Worker::new("127.0.0.1:8000").await?.run().await?;

    Ok(())
}
