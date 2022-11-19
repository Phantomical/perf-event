use nix::unistd::{ForkResult, Pid};
use perf_event::events::Software;
use perf_event::samples::RecordEvent;
use perf_event::Builder;

#[test]
fn record_self_fork() {
    let mut sampler = Builder::new()
        .kind(Software::DUMMY)
        .observe_self()
        .task(true)
        .build_sampler(4096)
        .expect("Failed to build sampler");

    sampler.enable().expect("Failed to enable sampler");

    let pid = match unsafe { nix::unistd::fork() }.expect("Failed to fork process") {
        ForkResult::Parent { child } => child.as_raw(),
        ForkResult::Child => {
            // Use _exit since at_exit functions are probably not async-signal-safe
            unsafe { libc::_exit(0) }
        }
    };

    sampler.disable().expect("Failed to disable sampler");

    let record = sampler
        .next_blocking(None)
        .expect("Sampler did not record any events");
    let record = match record.event {
        RecordEvent::Fork(record) => record,
        _ => panic!("expected an EXIT record, got {:?} instead", record.ty),
    };

    eprintln!("record: {:#?}", record);

    assert_eq!(record.pid, pid as _);
    assert_eq!(record.ppid, nix::unistd::getpid().as_raw() as _);
    assert_eq!(record.tid, pid as _);
    assert_eq!(record.ptid, nix::unistd::gettid().as_raw() as _);

    // Make sure to clean up the child process so it doesn't stick around as a
    // zombie. Probably not necessary for tests but it's good practice to do it
    // anyways.
    nix::sys::wait::waitpid(Pid::from_raw(pid), None)
        .expect("Failed to wait for child process to complete");
}
