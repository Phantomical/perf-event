use perf_event::events::Software;
use perf_event::samples::RecordEvent;
use perf_event::Builder;
use std::fs::File;
use std::io::Write;

#[test]
fn profile_bin_true() {
    let mut sampler = Builder::new()
        .kind(Software::DUMMY)
        .enable_on_exec(true)
        .task(true)
        .build_sampler(4096)
        .expect("Failed to build sampler");

    sampler.enable().expect("Failed to enable sampler");

    let mut file = File::create("/proc/self/comm").expect("Failed to open /proc/self/comm");
    file.write(b"test")
        .expect("Failed to write to /proc/self/comm");

    sampler.disable().expect("Failed to disable sampler");

    let record = sampler.next().expect("Sampler did not record any events");
    let record = match record.event {
        RecordEvent::Comm(mmap) => mmap,
        _ => panic!("expected a COMM record, got {:?} instead", record.ty),
    };

    eprintln!("record: {:#?}", record);

    assert_eq!(record.pid, nix::unistd::getpid().as_raw() as _);
    // Don't test record.tid since it doesn't match the current thread ID. It
    // seems to match the main thread ID but presumably this could change in
    // the future.
    assert_eq!(record.comm, "test");
}
