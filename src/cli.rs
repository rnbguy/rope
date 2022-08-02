use crate::AResult;
use clap::Parser;
use clap::{crate_authors, crate_description, crate_name, crate_version};
use tokio::sync::oneshot;

use crate::{generate_magic_string, recv_file, recv_msg, send_file, send_msg};

#[derive(Parser, Debug)]
#[clap(name = crate_name!(), version = crate_version!(), author = crate_authors!(), about = crate_description!())]
pub enum App {
    Send {
        file_path: String,
        opt_magic_string: Option<String>,
    },
    Recv {
        magic_string: String,
    },
}

impl App {
    pub async fn run(&self) -> AResult<()> {
        match self {
            App::Send {
                opt_magic_string,
                file_path,
            } => {
                let magic_string = opt_magic_string
                    .to_owned()
                    .unwrap_or_else(generate_magic_string);
                println!("MAGIC -> {magic_string}");

                let (tx, rx) = oneshot::channel();
                let port = send_file(file_path, tx).await?;
                send_msg(
                    &magic_string,
                    port,
                    [("name".into(), file_path.into())].into(),
                    rx,
                )
                .await?;
            }
            App::Recv { magic_string } => {
                let (addrs, port, data) = recv_msg(magic_string)?;
                recv_file(addrs.iter().next().unwrap(), port, &data["name"]).await?;
            }
        }
        Ok(())
    }
}
