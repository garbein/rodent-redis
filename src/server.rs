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
                let h = handler.run(stream).await;
                match h {
                    Ok(_) => (),
                    Err(e) => {println!("{:?}", e);},
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

    unsafe fn setsockopt<T>(
        &self,
        fd: libc::c_int,
        opt: libc::c_int,
        val: libc::c_int,
        payload: T,
    ) -> io::Result<()>
    where
        T: Copy,
    {
        let payload = &payload as *const T as *const libc::c_void;
        let res = libc::setsockopt(
            fd,
            opt,
            val,
            payload,
            std::mem::size_of::<T>() as libc::socklen_t,
        );
        if res == -1 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    unsafe fn set_keepalive(&self, stream: &TcpStream, sec: i32) -> io::Result<()> {
        use std::os::unix::io::AsRawFd;
        let fd = stream.as_raw_fd();

        self.setsockopt(fd, libc::SOL_SOCKET, libc::SO_KEEPALIVE, 1)?;

        let mut val = sec;
        self.setsockopt(fd, libc::IPPROTO_TCP, libc::TCP_KEEPIDLE, val)?;

        val = sec / 3;
        self.setsockopt(fd, libc::IPPROTO_TCP, libc::TCP_KEEPINTVL, val)?;

        val = 3;
        self.setsockopt(fd, libc::IPPROTO_TCP, libc::TCP_KEEPCNT, val)?;

        Ok(())
    }

    async fn run(&self, mut stream: TcpStream) -> anyhow::Result<()> {

        unsafe {
            self.set_keepalive(&stream, 15)?;
        }

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
                            b"push" => {
                                let k = match &bulks[1] {
                                    Resp::Bulk(t) => String::from_utf8(t.to_vec()).unwrap(),
                                    _ => panic!("set error"),
                                };
                                let v = match &bulks[2] {
                                    Resp::Bulk(t) => t,
                                    _ => panic!("set error"),
                                };
                                self.db.push(k, v.to_vec()).await;
                                &stream.write_all(b"+OK\r\n").await?;
                            },
                            b"pop" => {
                                let k = match &bulks[1] {
                                    Resp::Bulk(t) => String::from_utf8(t.to_vec()).unwrap(),
                                    _ => panic!("get error"),
                                };
                                let v_r = self.db.pop(k).await;
                                let r = match v_r {
                                    Some(v) => format!("${}\r\n{}\r\n", v.len(), String::from_utf8(v).unwrap()),
                                    None => "$-1\r\n".to_string(),
                                };
                                stream.write_all(r.as_bytes()).await?;
                            },
                            _ => stream.write_all("-Error unknown command\r\n".as_bytes()).await?,
                        }
                    }
                }
                _ => stream.write_all("-Error protocol error\r\n".as_bytes()).await?,
            }
            
        }
    }
}
