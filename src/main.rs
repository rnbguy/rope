use clap::{crate_authors, crate_description, crate_version};

use std::convert::TryInto;

use tokio_stream::StreamExt;

use clap::Clap;

use rand::{distributions::Alphanumeric, Rng};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    #[clap(short, long)]
    peer: Option<String>,
    #[clap(short, long)]
    filepath: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let opts: Opts = Opts::parse();

    let config = zenoh::net::config::default();
    let zenoh = zenoh::Zenoh::new(config).await?;
    let alias: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(4)
        .map(char::from)
        .collect();

    println!("alias {}", alias);

    let workspace = zenoh.workspace(Some("/rope".try_into()?)).await?;

    let mut my_stream = workspace.subscribe(&(alias.as_str()).try_into()?).await?;

    let peer = if let Some(peer) = opts.peer {
        workspace
            .put(&(peer.as_str()).try_into()?, (alias.as_str()).into())
            .await?;
        peer
    } else {
        match my_stream.next().await {
            Some(change) => match change.value {
                Some(zenoh::Value::StringUTF8(v)) => v,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    };

    println!("{} -> {}", alias, peer);

    Ok(())
}
