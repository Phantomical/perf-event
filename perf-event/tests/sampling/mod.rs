use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::Duration;

use nix::fcntl;
use nix::unistd::{ForkResult, Pid};
use perf_event::events::Software;
use perf_event::Builder;

mod comm;
mod exit;
mod fork;
mod lost;
mod mmap;
mod sample;

#[derive(Copy, Clone, Eq, PartialEq)]
struct Hex<T>(T);

impl<T: fmt::UpperHex> fmt::Display for Hex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::UpperHex> fmt::Debug for Hex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("0x")?;
        self.0.fmt(f)
    }
}

#[test]
fn next_blocking_does_not_hang_if_child_exits() {
    let (tx, rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn(move || {
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

        // This should eventually exit once the child process has completed and
        // we've gone through all the records.
        while let Some(_) = sampler.next_blocking(None) {}

        // Make sure to clean up the child process so it doesn't stick around as a
        // zombie. Probably not necessary for tests but it's good practice to do it
        // anyways.
        nix::sys::wait::waitpid(Pid::from_raw(pid), None)
            .expect("Failed to wait for child process to complete");

        tx.send(()).expect("Failed to send record on channel");
    });

    if let Err(e) = rx.recv_timeout(Duration::from_secs(60)) {
        if handle.is_finished() {
            if let Err(e) = handle.join() {
                std::panic::resume_unwind(e);
            }
        }

        panic!("next_blocking test case failed to exit in time: {}", e);
    }

    if let Err(e) = handle.join() {
        std::panic::resume_unwind(e);
    }
}
