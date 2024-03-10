use anyhow::Result as AResult;
use clap::Parser;
use rope::cli::App;

#[tokio::main]
async fn main() -> AResult<()> {
    // tracing_subscriber::fmt::init();
    App::parse().run().await
}
