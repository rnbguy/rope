use core::fmt::Write;

use anyhow::Context;
use if_addrs::{IfAddr, Interface};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use crate::AResult;

pub fn my_ext_interfaces() -> Vec<IfAddr> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter(Interface::is_loopback)
        .map(|i| i.addr)
        .collect()
}

pub fn get_progressbar(size: u64) -> AResult<ProgressBar> {
    let pb = ProgressBar::new(size);
    pb.set_style(ProgressStyle::with_template("{spinner:.yellow} [{elapsed_precise}] [{wide_bar:.green/red}] {bytes}/{total_bytes} ({eta})")
        .context("Failed to set progress bar style")?
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("█▓▒░─"));
    Ok(pb)
}
