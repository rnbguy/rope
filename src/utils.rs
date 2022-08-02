use std::fmt::Write;

use if_addrs::{IfAddr, Ifv4Addr};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

pub fn my_ipv4_interfaces() -> Vec<Ifv4Addr> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|i| {
            if i.is_loopback() {
                None
            } else {
                match i.addr {
                    IfAddr::V4(ifv4) => Some(ifv4),
                    _ => None,
                }
            }
        })
        .collect()
}

pub fn get_progressbar(size: u64) -> ProgressBar {
    let pb = ProgressBar::new(size);
    pb.set_style(ProgressStyle::with_template("{spinner:.yellow} [{elapsed_precise}] [{wide_bar:.green/red}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("█▓▒░─"));

    pb
}
