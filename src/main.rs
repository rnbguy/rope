use clap::Parser;
use rope::cli::App;
use rope::AResult;

#[tokio::main]
async fn main() -> AResult<()> {
    App::parse().run().await
}
