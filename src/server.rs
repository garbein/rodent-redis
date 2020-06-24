use crate::db::Db;
use crate::resp::Resp;
use async_std::io::{self, BufReader};
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
        Server { db: Db::new() }
    }

    pub async fn run(config: Config) -> io::Result<()> {
        let addr = format!("{}:{}", config.host, config.port);
        let listener = TcpListener::bind(addr).await?;
        let server = Server::new();
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            let handler = Handler {
                db: server.db.clone(),
            };
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
    async fn run(&self, mut stream: TcpStream) -> anyhow::Result<()> {
        loop {
            let mut reader = BufReader::new(&stream);
            let resp = Resp::parse(&mut reader).await?;
            match resp {
                Resp::Array(bulks) => {
                    if let Resp::Bulk(vec) = &bulks[0] {
                        let cmd = &vec[..];
                        match cmd {
                            b"ping" => {
                                &stream.write_all(b"+PONG\r\n").await?;
                            },
                            b"set" => {
                                let k = match &bulks[1] {
                                    Resp::Bulk(t) => String::from_utf8(t.to_vec()).unwrap(),
                                    _ => panic!("set error"),
                                };
                                let v = match &bulks[2] {
                                    Resp::Bulk(t) => t,
                                    _ => panic!("set error"),
                                };
                                self.db.set(k, v.to_vec()).await;
                                &stream.write_all(b"+OK\r\n").await?;
                            },
                            b"get" => {
                                let k = match &bulks[1] {
                                    Resp::Bulk(t) => String::from_utf8(t.to_vec()).unwrap(),
                                    _ => panic!("get error"),
                                };
                                let v_r = self.db.get(k).await;
                                let r = match v_r {
                                    Some(v) => format!("${}\r\n{}\r\n", v.len(), String::from_utf8(v).unwrap()),
                                    None => "$-1\r\n".to_string(),
                                };
                                stream.write_all(r.as_bytes()).await?;
                            },
                            _ => {},
                        }
                    }
                }
                _ => (),
            }
            
        }
    }
}
