use async_std::io;
use async_std::prelude::*;
use async_std::net::TcpStream;

#[async_std::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6380").await?;
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
        cmd = format!("*{}\r\n{}\0", n, &cmd);
        println!("{}", cmd);
        stream.write_all(cmd.as_bytes()).await?;
        let mut buf = Vec::new();
        let n = stream.read(&mut buf).await?;
        stdout.write_all(&buf[..n]).await?;
        stdout.write_all(prompt.as_bytes()).await?;
        stdout.flush().await?;
        line.clear();
    }
}