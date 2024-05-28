use std::thread::sleep;
use std::time::Duration;

use libc::pid_t;
use perf_event::events::Hardware;
use perf_event::Builder;

fn main() -> std::io::Result<()> {
    let pid: pid_t = std::env::args()
        .nth(1)
        .expect("Usage: insns-for-pid PID")
        .parse()
        .expect("Usage: insns-for-pid PID");

    let mut insns = Builder::new(Hardware::BRANCH_INSTRUCTIONS)
        .observe_pid(pid)
        .build()?;

    // Count instructions in PID for five seconds.
    insns.enable()?;
    sleep(Duration::from_secs(5));
    insns.disable()?;

    println!("instructions in last five seconds: {}", insns.read()?);

    Ok(())
}
