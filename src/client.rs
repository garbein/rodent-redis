use crate::resp::Resp;
use async_std::io::{self, BufReader};
use async_std::net::TcpStream;
use async_std::prelude::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "rodent-redis-cli")]
pub struct Options {
    #[structopt(name = "hostname", long = "--host", default_value = "127.0.0.1")]
    host: String,

    #[structopt(name = "port", long = "--port", default_value = "6380")]
    port: String,
}

pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Client {}
    }

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

    pub async fn run(&self, options: Options) -> anyhow::Result<()> {
        let mut stream = TcpStream::connect(format!("{}:{}", options.host, options.port)).await?;

        unsafe {
            self.set_keepalive(&stream, 15)?;
        }

        let prompt = format!("{}> ", stream.peer_addr()?);
        let mut stdout = io::stdout();
        stdout.write_all(prompt.as_bytes()).await?;
        stdout.flush().await?;
        let stdin = io::stdin();
        let mut line = String::new();
        loop {
            stdin.read_line(&mut line).await?;
            line = line.trim().to_string();
            if line.is_empty() {
                stdout.write_all(prompt.as_bytes()).await?;
                stdout.flush().await?;
                continue;
            }
            let args = line.split_whitespace();
            let mut cmd = String::new();
            let mut n: u8 = 0;
            for arg in args {
                n = n + 1;
                cmd = format!("{}${}\r\n{}\r\n", cmd, arg.len(), arg);
            }
            cmd = format!("*{}\r\n{}", n, &cmd);
            stream.write_all(cmd.as_bytes()).await?;

            let mut reader = BufReader::new(&stream);
            let resp = Resp::parse(&mut reader).await?;
            match resp {
                Resp::Simple(v) => println!("{:?}", String::from_utf8(v.to_vec())?),
                Resp::Error(v) => println!("{:?}", String::from_utf8(v.to_vec())?),
                Resp::Integer(v) => println!("{:?}", v),
                Resp::Bulk(v) => {
                    stdout.write_all(&v).await?;
                    println!("");
                }
                Resp::Array(vec) => {
                    for bulk in vec {
                        if let Resp::Bulk(v) = bulk {
                            stdout.write_all(&v).await?;
                            println!("");
                        }
                    }
                }
                Resp::Null => println!("nil"),
            }
            stdout.write_all(prompt.as_bytes()).await?;
            stdout.flush().await?;
            line.clear();
        }
    }
}
