use crate::db::Db;
use async_std::io::{self, BufReader};
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use structopt::StructOpt;
use crate::resp::Resp;

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
    async fn run(&self, stream: TcpStream) -> anyhow::Result<()> {
        let mut reader = BufReader::new(&stream);
        let resp = Resp::parse(&mut reader).await?;
        println!("{:?}", resp);
        match resp {
            Resp::Array(vec) => {
                if let Resp::Bulk(v) = &vec[0] {
                    println!("{:?}", String::from_utf8(v.to_vec()));
                }
               
            },
            _ => (),
        }
        Ok(())
    }
}