use clap::Parser;
use rope::cli::App;
use rope::AResult;

#[tokio::main]
async fn main() -> AResult<()> {
    // tracing_subscriber::fmt::init();
    App::parse().run().await
}
