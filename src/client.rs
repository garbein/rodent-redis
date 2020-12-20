use crate::resp::Resp;
use async_std::io::{self, BufReader};
use async_std::net::TcpStream;
use async_std::prelude::*;
use structopt::StructOpt;
use crate::networking;

/// 客户端配置
#[derive(StructOpt, Debug)]
#[structopt(name = "rodent-redis-cli")]
pub struct Options {
    // 默认连接主机127.0.0.1
    #[structopt(name = "hostname", long = "--host", default_value = "127.0.0.1")]
    host: String,
    // 默认连接端口6380
    #[structopt(name = "port", long = "--port", default_value = "6380")]
    port: String,
}

/// 客户端
pub struct Client {
    host: String,
    port: String,
}

impl Client {

    /// 创建client
    /// # 例子
    /// 
    /// ```Rust
    /// let options = Options::from_args();
    /// let client = Client::new(options);
    /// client.run().await;
    /// ```
    /// 
    pub fn new(options: Options) -> Self {
        Client {host: options.host, port: options.port}
    }

    /// 运行客户端
    pub async fn run(&self) -> anyhow::Result<()> {
        // 连接服务端,　stream代表此连接
        let mut stream = TcpStream::connect(format!("{}:{}", self.host, self.port)).await?;

        // 心跳检测
        unsafe {
            networking::set_keepalive(&stream, 15)?;
        }

        // 命令行对话窗提示 127.0.0.1:6380>
        let prompt = format!("{}> ", stream.peer_addr()?);
        let mut stdout = io::stdout();
        stdout.write_all(prompt.as_bytes()).await?;
        stdout.flush().await?;

        let stdin = io::stdin();
        let mut line = String::new();
        loop {
            // 等待用户输入，按行读
            stdin.read_line(&mut line).await?;
            line = line.trim().to_string();
            // 用户直接按回车继续等待输入
            if line.is_empty() {
                stdout.write_all(prompt.as_bytes()).await?;
                stdout.flush().await?;
                continue;
            }
            // 按空格分割输入字符
            let args = line.split_whitespace();
            let mut cmd = String::new();
            let mut n: u8 = 0;
            // 拼接redis协议
            for arg in args {
                n = n + 1;
                cmd = format!("{}${}\r\n{}\r\n", cmd, arg.len(), arg);
            }
            cmd = format!("*{}\r\n{}", n, &cmd);
            // 发送redis协议数据对服务端
            stream.write_all(cmd.as_bytes()).await?;

            // 获取并解析服务端响应，输出结果到终端
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
