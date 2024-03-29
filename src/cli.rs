use std::fs::metadata;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, ensure, Context};
use clap::{crate_authors, crate_description, crate_name, crate_version, Parser};
use tokio::sync::oneshot;
use tracing::{debug, info};

use crate::{generate_magic_string, recv_file, recv_msg, send_file, send_msg, AResult};

#[derive(Parser, Debug)]
#[clap(name = crate_name!(), version = crate_version!(), author = crate_authors!(), about = crate_description!())]
pub enum App {
    Send {
        file_path: String,
        magic_string: Option<String>,
    },
    Recv {
        magic_string: String,
        save_dir: Option<PathBuf>,
    },
}

impl App {
    pub async fn run(self) -> AResult<()> {
        debug!("{self:?}");
        match self {
            Self::Send {
                magic_string: magic_string_opt,
                file_path,
            } => {
                let magic_string = match magic_string_opt {
                    Some(s) => s,
                    None => generate_magic_string()?,
                };

                println!("MAGIC: {magic_string}");

                info!("MAGIC: {magic_string}");

                let (tx, rx) = oneshot::channel();

                let file_size = metadata(&file_path)?.len();

                let port = send_file(&file_path, file_size, tx).await?;

                send_msg(
                    &magic_string,
                    port,
                    [
                        ("name".into(), file_path),
                        ("size".into(), file_size.to_string()),
                    ]
                    .into(),
                )?;

                rx.await?;
            }
            Self::Recv {
                magic_string,
                save_dir,
            } => {
                let (addrs, port, data) = recv_msg(&magic_string)?;
                let name = Path::new(
                    data.get_property_val_str("name")
                        .context("`name` key must be present")?,
                )
                .file_name()
                .and_then(|x| x.to_str())
                .ok_or_else(|| anyhow!("Error while read filename"))?;
                let path = save_dir.clone().unwrap_or_else(PathBuf::new).join(name);
                ensure!(path.exists(), "The file already exists");
                for addr in &addrs {
                    debug!("Trying {addr}");
                    if recv_file(
                        addr,
                        port,
                        &path,
                        data.get_property_val_str("size")
                            .context("`size` key must be present")?
                            .parse()?,
                    )
                    .await
                    .is_ok()
                    {
                        debug!("File is received. Breaking loop");
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
