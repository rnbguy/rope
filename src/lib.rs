pub mod cli;
pub mod utils;

use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use names::Generator;
use tokio::fs::File;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

pub use anyhow::Result as AResult;

fn generate_magic_string() -> String {
    let mut generator = Generator::default();
    generator.next().unwrap()
}

async fn send_file(file_path: &str, tx: oneshot::Sender<()>) -> AResult<u16> {
    let listener = TcpListener::bind("0:0").await?;

    let addr = listener.local_addr()?;

    let file_path_owned = file_path.to_owned();

    tokio::spawn(async move {
        let (mut socket, _b) = listener.accept().await?;
        let mut f = File::open(file_path_owned).await?;
        tokio::io::copy(&mut f, &mut socket).await?;
        tx.send(()).unwrap();
        anyhow::Ok(())
    });

    Ok(addr.port())
}

async fn recv_file(ip: &Ipv4Addr, port: u16, name: &str) -> AResult<()> {
    let addr = format!("{ip}:{port}");
    let mut stream = TcpStream::connect(addr).await?;

    let mut f = File::create(name).await?;
    tokio::io::copy(&mut stream, &mut f).await?;

    Ok(())
}

const SERVICE_TYPE: &str = "_rope._tcp.local.";

async fn send_msg(
    magic_string: &str,
    port: u16,
    data: HashMap<String, String>,
    rx: oneshot::Receiver<()>,
) -> AResult<()> {
    let mdns = ServiceDaemon::new()?;
    let my_addrs: Vec<Ipv4Addr> = crate::utils::my_ipv4_interfaces()
        .iter()
        .map(|i| i.ip)
        .collect();
    let service_hostname = format!("{magic_string}.local.");

    let service_info = ServiceInfo::new(
        SERVICE_TYPE,
        magic_string,
        &service_hostname,
        &my_addrs[..],
        port,
        Some(data),
    )?;

    mdns.register(service_info)?;

    rx.await?;

    Ok(())
}

fn recv_msg(magic_string: &str) -> AResult<(HashSet<Ipv4Addr>, u16, HashMap<String, String>)> {
    let mdns = ServiceDaemon::new()?;

    let receiver = mdns.browse(SERVICE_TYPE)?;

    let expected_fullname = format!("{magic_string}.{SERVICE_TYPE}");

    while let Ok(event) = receiver.recv() {
        if let ServiceEvent::ServiceResolved(info) = event {
            if info.get_fullname() == expected_fullname {
                return Ok((
                    info.get_addresses().clone(),
                    info.get_port(),
                    info.get_properties().clone(),
                ));
            }
        }
    }

    unreachable!()
}
