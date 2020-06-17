use structopt::StructOpt;
use async_std::io;
use rodent_redis::server::{Config, Server};

#[async_std::main]
async fn main() -> io::Result<()> {
    let config = Config::from_args();
    Server::run(config).await
}