use crate::db::Db;
use async_std::io::{self, Read, BufReader};
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "rodent-redis-server")]
pub struct Config {
    #[structopt(name = "hostname", long = "--host", default_value = "127.0.0.1")]
    host: String,

    #[structopt(name = "port", long = "--port", default_value = "6380")]
    port: String,
}

pub struct Server {
    db: Db,
}

impl Server {
    pub fn new() -> Self {
        Server { db: Db { id: 0u8 } }
    }

    pub async fn run(config: Config) -> io::Result<()> {
        let addr = format!("{}:{}", config.host, config.port);
        let listener = TcpListener::bind(addr).await?;
        let server = Server::new();
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            let handler = Handler { db: server.db.clone() };
            task::spawn(async move {
                handler.run(stream).await.unwrap();
            });
        }
        Ok(())
    }
}

struct Handler {
    db: Db,
}

impl Handler {
    async fn run(&self, mut stream: TcpStream) -> io::Result<()> {
        stream.bytes
        let reader = BufReader::new(&stream);
        let mut lines = reader.lines();
        while let Some(line) = lines.next().await {
            let line = line?;
            if line.starts_with("*") {

            }
            println!("{}", line);
        }
        stream.write_all("+".as_bytes()).await?;
        Ok(())
    }
}
