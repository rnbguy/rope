pub mod cli;
pub mod utils;

use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;
use std::path::PathBuf;

use anyhow::anyhow;

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo, TxtProperties};
use names::Generator;
use tokio::fs::File;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

pub use anyhow::Result as AResult;
use tracing::debug;

use crate::utils::get_progressbar;

const SERVICE_TYPE: &str = "_rope._tcp.local.";

fn generate_magic_string() -> String {
    let mut generator = Generator::default();
    generator.next().unwrap()
}

fn send_msg(magic_string: &str, port: u16, data: HashMap<String, String>) -> AResult<()> {
    let mdns = ServiceDaemon::new()?;
    let my_addrs: Vec<Ipv4Addr> = crate::utils::my_ipv4_interfaces()
        .iter()
        .map(|i| i.ip)
        .collect();

    debug!("Collected addresses: {my_addrs:?}");

    let service_info = ServiceInfo::new(
        SERVICE_TYPE,
        magic_string,
        &format!("{magic_string}.local."),
        &my_addrs[..],
        port,
        Some(data),
    )?;

    mdns.register(service_info)?;

    debug!("Service registered: {magic_string}.{SERVICE_TYPE}");

    Ok(())
}

fn recv_msg(magic_string: &str) -> AResult<(HashSet<Ipv4Addr>, u16, TxtProperties)> {
    let mdns = ServiceDaemon::new()?;

    let receiver = mdns.browse(SERVICE_TYPE)?;

    let expected_fullname = format!("{magic_string}.{SERVICE_TYPE}");

    loop {
        if let ServiceEvent::ServiceResolved(info) = receiver.recv()? {
            if info.get_fullname() == expected_fullname {
                debug!("Matched service: {info:?}");
                return Ok((
                    info.get_addresses().clone(),
                    info.get_port(),
                    info.get_properties().clone(),
                ));
            }
        }
    }
}

async fn send_file(file_path: &str, size: u64, tx: oneshot::Sender<()>) -> AResult<u16> {
    let listener = TcpListener::bind("0:0").await?;

    let addr = listener.local_addr()?;

    debug!("Listening at {addr:?}");

    let file_path_owned = file_path.to_owned();

    tokio::spawn(async move {
        let (mut socket, _b) = listener.accept().await?;

        debug!("Peer is connected. Sending file: {file_path_owned}");

        let pb = get_progressbar(size);

        let f = File::open(file_path_owned).await?;

        tokio::io::copy(&mut pb.wrap_async_read(f), &mut socket).await?;

        debug!("Done. Sending signal via channel");

        tx.send(())
            .map_err(|_| anyhow!("Couldn't sent signal via channel"))
    });

    Ok(addr.port())
}

async fn recv_file(ip: &Ipv4Addr, port: u16, path: &PathBuf, size: u64) -> AResult<()> {
    let addr = format!("{ip}:{port}");
    let mut stream = TcpStream::connect(addr).await?;

    debug!("Peer is connected. Receiving file: {path:?}");

    let pb = get_progressbar(size);

    let f = File::create(path).await?;

    tokio::io::copy(&mut stream, &mut pb.wrap_async_write(f)).await?;

    debug!("Done");

    Ok(())
}
