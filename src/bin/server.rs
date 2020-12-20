//! rodent-redis-server
//! 默认主机: 127.0.0.1
//! 默认端口: 6380

use structopt::StructOpt;
use async_std::io;
use rodent_redis::server::{Config, Server};

#[async_std::main]
async fn main() -> io::Result<()> {
    let config = Config::from_args();
    Server::run(config).await
}