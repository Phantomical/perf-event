use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::process::Command;

use nix::fcntl;
use nix::unistd::{ForkResult, Pid};
use perf_event::events::Software;
use perf_event::samples::RecordEvent;
use perf_event::Builder;

#[test]
fn record_exit_usr_bin_true() {
    let builder = Builder::new()
        .kind(Software::DUMMY)
        .enable_on_exec(true)
        .task(true);

    let (mut remote, mut local) = nix::unistd::pipe()
        .map(|(rx, tx)| unsafe { (File::from_raw_fd(rx), File::from_raw_fd(tx)) })
        .expect("Failed to create a pipe");

    // Ensure that the remote pipe gets closed when the forked process calls exec.
    fcntl::fcntl(remote.as_raw_fd(), fcntl::F_SETFL(fcntl::OFlag::O_CLOEXEC))
        .expect("Failed to set CLOEXEC flag on remote end of pipe");

    let mut command = Command::new("true");
    unsafe {
        command.pre_exec(move || {
            let mut buf = [0];
            remote.read_exact(&mut buf)?;
            Ok(())
        })
    };

    let pid = match unsafe { nix::unistd::fork() }.expect("Failed to fork process") {
        ForkResult::Parent { child } => child.as_raw(),
        ForkResult::Child => {
            let error = command.exec();
            eprintln!("an error occurred while attempting to exec the child program: {error}");

            // Use _exit since at_exit functions are probably not async-signal-safe
            unsafe { libc::_exit(127) }
        }
    };

    let mut sampler = builder
        .observe_pid(pid)
        .build_sampler(4096)
        .expect("Failed to build sampler");

    local
        .write_all(&[0])
        .expect("Failed to write to child process pipe");

    let record = sampler
        .next_blocking(None)
        .expect("Sampler did not record any events");
    let record = match record.event {
        RecordEvent::Exit(mmap) => mmap,
        _ => panic!("expected an EXIT record, got {:?} instead", record.ty),
    };

    eprintln!("record: {:#?}", record);

    assert_eq!(record.pid, pid as _);
    assert_eq!(record.ppid, nix::unistd::getpid().as_raw() as _);
    // /usr/bin/true spawns no threads so tid should be the same as pid
    assert_eq!(record.tid, pid as _);
    // Don't test record.ptid since it doesn't necessarily match the current
    // thread ID. It currently seems to match the main thread ID but this
    // could presumably change in the future.

    // Make sure to clean up the child process so it doesn't stick around as a
    // zombie. Probably not necessary for tests but it's good practice to do it
    // anyways.
    nix::sys::wait::waitpid(Pid::from_raw(pid), None)
        .expect("Failed to wait for child process to complete");
}
