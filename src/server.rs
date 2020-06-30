use crate::commands::CommandInfo;
use crate::db::Db;
use crate::networking;
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
                let h = handler.run(stream).await;
                match h {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
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
        unsafe {
            networking::set_keepalive(&stream, 300)?;
        }

        loop {
            let mut reader = BufReader::new(&stream);
            let resp = Resp::parse(&mut reader).await?;
            let cmd;
            match CommandInfo::from_resp(resp) {
                Ok(t) => cmd = t,
                Err(e) => {
                    self.write_error(&mut stream, &e.to_string()).await?;
                    continue;
                }
            };
            let resp = self.db.execute(cmd).await;
            self.write_resp(&mut stream, resp).await?;
        }
    }

    async fn write_resp(&self, stream: &mut TcpStream, resp: Resp) -> io::Result<()> {
        let mut crlf = vec![b'\r', b'\n'];
        let mut buf;
        match resp {
            Resp::Simple(mut v) => {
                buf = vec![b'+'];
                buf.append(&mut v);
                buf.append(&mut crlf);
            }
            Resp::Error(mut v) => {
                buf = vec![b'-'];
                buf.append(&mut v);
                buf.append(&mut crlf);
            }
            Resp::Integer(v) => buf = format!(":{}\r\n", v).as_bytes().to_vec(),
            Resp::Bulk(mut v) => {
                buf = vec![b'$', v.len() as u8, b'\r', b'\n'];
                buf.append(&mut v);
                buf.append(&mut crlf);
            }
            _ => buf = "$-1\r\n".as_bytes().to_vec(),
        };
        stream.write_all(&buf).await
    }

    async fn write_error(&self, stream: &mut TcpStream, e: &str) -> io::Result<()> {
        stream.write_all(e.as_bytes()).await
    }
}
