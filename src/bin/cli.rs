//! rodent-redis-cli支持redis协议
//! 使用说明：
//! 默认连接主机: 127.0.0.1
//! 默认连接端口: 6380
//! 使用例子：
//! 默认参数
//! 
//! ```shell
//! rodent-redis-cli 
//! ```
//! 指定host和port连接redis
//! 
//! ```shell
//! rodent-redis-cli --host 127.0.0.1 --port 6379
//! ```
//! 

use rodent_redis::client::{Client, Options};
use structopt::StructOpt;

fn main() {
    // 命令行参数保存在Option
    let options = Options::from_args();
    let client = Client::new(options);
    // 运行客户端
    async_std::task::block_on(async {
        client.run().await.unwrap_or_else(|error|{println!("{}", error)});
    });
}
