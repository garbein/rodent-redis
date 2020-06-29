use rodent_redis::client::{Client, Options};
use structopt::StructOpt;

fn main() {
    std::env::set_var("ASYNC_STD_THREAD_COUNT", "1");
    let options = Options::from_args();
    let client = Client::new();
    async_std::task::block_on(async {
        client.run(options).await.unwrap();
    });
}
