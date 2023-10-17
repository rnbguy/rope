use std::fmt::Write;

use if_addrs::IfAddr;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

pub fn my_ext_interfaces() -> Vec<IfAddr> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter(|i| i.is_loopback())
        .map(|i| i.addr)
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
