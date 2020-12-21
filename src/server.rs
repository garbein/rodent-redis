use crate::commands::CommandInfo;
use crate::db::Db;
use crate::networking;
use crate::resp::Resp;
use async_std::io::{self, BufReader};
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use structopt::StructOpt;

/// 服务端配置
#[derive(StructOpt, Debug)]
#[structopt(name = "rodent-redis-server")]
pub struct Config {
    #[structopt(name = "hostname", long = "--host", default_value = "127.0.0.1")]
    host: String,

    #[structopt(name = "port", long = "--port", default_value = "6380")]
    port: String,
}

/// 服务端
pub struct Server {
    /// 数据库
    db: Db,
    /// 主机
    host: String,
    /// 端口
    port: String,
}

impl Server {
    /// 创建服务端
    pub fn new(config: Config) -> Self {
        Server {
            db: Db::new(),
            host: config.host,
            port: config.port,
        }
    }

    /// 运行服务端
    pub async fn run(config: Config) -> io::Result<()> {
        // 创建服务端
        let server = Server::new(config);
        let addr = format!("{}:{}", server.host, server.port);
        // 监听主机和端口
        let listener = TcpListener::bind(addr).await?;
        // 监听请求到达
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            // 创建请求处理handler
            let handler = Handler {
                db: server.db.clone(),
            };
            // spawn一个task创处理请求
            task::spawn(async move {
                handler.run(stream).await.unwrap_or_else(|error| println!("{:?}", error));
            });
        }
        Ok(())
    }
}

/// 请求处理handler
struct Handler {
    db: Db,
}

impl Handler {

    /// 处理请求
    async fn run(&self, mut stream: TcpStream) -> anyhow::Result<()> {
        // 心跳检测
        unsafe {
            networking::set_keepalive(&stream, 300)?;
        }
        loop {
            let mut reader = BufReader::new(&stream);
            // 解析redis协议
            let resp = Resp::parse(&mut reader).await?;
            let cmd;
            match CommandInfo::from_resp(resp) {
                Ok(t) => cmd = t,
                Err(e) => {
                    self.write_error(&mut stream, e.to_string()).await?;
                    continue;
                }
            };
            // 执行redis命令
            let resp = self.db.execute(cmd).await;
            // 返回数据给客户端
            self.write_resp(&mut stream, resp).await?;
        }
    }

    /// 返回数据给客户端
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
                buf = format!("${}\r\n", v.len()).as_bytes().to_vec();
                buf.append(&mut v);
                buf.append(&mut crlf);
            }
            _ => buf = "$-1\r\n".as_bytes().to_vec(),
        };
        stream.write_all(&buf).await
    }

    /// 返回错误给客户端
    async fn write_error(&self, stream: &mut TcpStream, e: String) -> io::Result<()> {
        stream.write_all(format!("-{}\r\n", e).as_bytes()).await
    }
}
