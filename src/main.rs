use rand::{distributions::Alphanumeric, Rng};
use std::{convert::TryInto, fs::File, net::TcpListener, net::TcpStream};
use tokio_stream::StreamExt;

use clap::{crate_authors, crate_description, crate_version, Clap};
use serde::{Deserialize, Serialize};

use zenoh::{net, Value, Zenoh};

#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
    name: String,
    size: u64,
    addr: String,
}

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

    let config = net::config::default();
    let zenoh = Zenoh::new(config).await?;
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
                Some(Value::StringUTF8(v)) => v,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    };

    if let Some(filepath) = opts.filepath {
        let listener = TcpListener::bind("0.0.0.0:0")?;
        // TODO: resolve to interface address
        let addr = listener.local_addr()?.to_string();

        let name = std::path::Path::new(&filepath)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .into();
        let mut source = File::open(&filepath)?;
        let size = source.metadata()?.len();

        let finfo = FileInfo { name, size, addr };

        log::info!("file info {:?}", finfo);

        workspace
            .put(
                &(peer.as_str()).try_into()?,
                bincode::serialize(&finfo)?.into(),
            )
            .await?;

        for stream in listener.incoming() {
            match stream {
                Ok(target) => {
                    let pb = indicatif::ProgressBar::new(size);
                    std::io::copy(&mut source, &mut pb.wrap_write(target))?;
                    break;
                }
                _ => unreachable!(),
            }
        }
    } else {
        let finfo: FileInfo = match my_stream.next().await {
            Some(change) => match change.value {
                Some(Value::Raw(zi, buf)) if zi == 0 => {
                    bincode::deserialize(&buf.to_vec()).unwrap()
                }
                _ => {
                    println!("{:?}", change);
                    unreachable!()
                }
            },
            _ => unreachable!(),
        };
        let mut source = TcpStream::connect(finfo.addr)?;
        let target = File::create(finfo.name)?;
        let pb = indicatif::ProgressBar::new(finfo.size);
        std::io::copy(&mut source, &mut pb.wrap_write(target))?;
    }

    Ok(())
}
