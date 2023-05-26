use std::process::ExitCode;
use std::time::Duration;

use perf_event::events::x86::{Msr, MsrId};
use perf_event::Builder;

fn run() -> std::io::Result<()> {
    let tsc_event = Msr::new(MsrId::TSC)?;
    let aperf_event = Msr::new(MsrId::APERF)?;
    let mperf_event = Msr::new(MsrId::MPERF)?;

    let mut tsc = Builder::new(tsc_event)
        .one_cpu(0)
        .any_pid()
        .exclude_hv(false)
        .exclude_kernel(false)
        .build()?;
    let mut aperf = Builder::new(aperf_event)
        .one_cpu(0)
        .any_pid()
        .exclude_hv(false)
        .exclude_kernel(false)
        .build_with_group(&mut tsc)?;
    let mut mperf = Builder::new(mperf_event)
        .one_cpu(0)
        .any_pid()
        .exclude_hv(false)
        .exclude_kernel(false)
        .build_with_group(&mut tsc)?;

    tsc.enable_group()?;
    std::thread::sleep(Duration::from_secs(1));
    tsc.disable_group()?;

    let tsc_val: u64 = tsc.read()?;
    let ghz = tsc_val as f64 / (1000000000.0);
    let aperf_val = aperf.read()?;
    let mperf_val = mperf.read()?;
    let ratio = aperf_val as f64 / mperf_val as f64;
    let run_freq = ghz * ratio;

    println!(
        "{tsc_val} ref cycles passed in one second (~{ghz} GHz)\n\
        APERF: {aperf_val}\n\
        MPERF: {mperf_val}\n\
        Ratio: {ratio}\n\
        Running Frequency: {run_freq} GHz"
    );

    println!("{tsc_val} ref cycles passed in one second (~{ghz} GHz)\nAPERF: {aperf_val} MPERF: {mperf_val} Ratio: {ratio} Running frequency:{run_freq} GHz");
    Ok(())
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("{e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
