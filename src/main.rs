use rand::{distributions::Alphanumeric, Rng};
use std::{convert::TryInto, fs::File, net::TcpListener, net::TcpStream};
use tokio_stream::StreamExt;

use clap::{crate_authors, crate_description, crate_version, Clap};
use serde::{Deserialize, Serialize};

use zenoh::{net, Value, Zenoh};

use zenoh_protocol::link::Locator;

use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
    name: String,
    size: u64,
    addrs: Vec<String>,
}

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    #[clap(short, long)]
    alias: Option<String>,
    #[clap(short, long)]
    path: Option<String>,
}

async fn find_locator(peerid: &str) -> Option<Vec<SocketAddr>> {
    let mut stream =
        zenoh::net::scout(zenoh::net::whatami::PEER, zenoh::net::config::default()).await;
    let mut v: Option<Vec<_>> = None;
    while let Some(hello) = stream.next().await {
        if hello.pid.unwrap().to_string() == peerid {
            v = hello.locators.map(|mut x| {
                x.drain(..)
                    .map(|x| match x {
                        Locator::Tcp(x) | Locator::Udp(x) => x,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>()
            });
            break;
        }
    }
    v
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
    let session_id = zenoh.session().id().await;

    let mut my_stream = workspace
        .subscribe(&(session_id.as_str()).try_into()?)
        .await?;

    let peer = if let Some(alias) = opts.alias {
        workspace
            .put(&(alias.as_str()).try_into()?, (session_id.as_str()).into())
            .await?;
        match my_stream.next().await {
            Some(change) => match change.value {
                Some(Value::StringUTF8(v)) => v,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    } else {
        let peer = {
            let mut flag_stream = workspace.subscribe(&(alias.as_str()).try_into()?).await?;
            match flag_stream.next().await {
                Some(change) => match change.value {
                    Some(Value::StringUTF8(v)) => v,
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        };
        workspace
            .put(&(peer.as_str()).try_into()?, (session_id.as_str()).into())
            .await?;
        peer
    };

    if let Some(path) = opts.path {
        let listener = TcpListener::bind("0.0.0.0:0")?;
        let port = listener.local_addr()?.port();

        let name = std::path::Path::new(&path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .into();
        let mut source = File::open(&path)?;
        let size = source.metadata()?.len();

        let addrs = find_locator(&session_id)
            .await
            .unwrap()
            .drain(..)
            .map(|x| match x {
                SocketAddr::V4(mut v4) => {
                    v4.set_port(port);
                    v4.to_string()
                }
                SocketAddr::V6(mut v6) => {
                    v6.set_port(port);
                    v6.to_string()
                }
            })
            .collect::<Vec<_>>();

        let finfo = FileInfo { name, size, addrs };

        log::debug!("file info {:?}", finfo);

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
        log::debug!("received file info {:?}", finfo);
        for addr in finfo.addrs {
            log::debug!("trying {:?}", addr);
            if let Ok(mut source) = TcpStream::connect(addr) {
                let target = File::create(&finfo.name)?;
                let pb = indicatif::ProgressBar::new(finfo.size);
                std::io::copy(&mut source, &mut pb.wrap_write(target))?;
                break;
            }
        }
    }

    Ok(())
}
