use clap::Parser;
use kmm::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    args.run().await;
}
