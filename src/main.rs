use clap::Parser;
use kmm::{sing_app::SingApp, Cli};

#[tokio::main]
async fn main() {
    let _app = SingApp::run_current().unwrap();
    let cli = Cli::parse();
    cli.run()
}
