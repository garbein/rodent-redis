use structopt::StructOpt;
use rodent_redis::client::{Client, Options};

#[async_std::main]
async fn main() {
    let options = Options::from_args();
    let client = Client::new();
    client.run(options).await.unwrap();
}