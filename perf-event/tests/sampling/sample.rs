use perf_event::events::Breakpoint;
use perf_event::samples::{RecordEvent, SampleType};
use perf_event::Builder;

// Need a function that will not be optimized away or inlined by the compiler.
#[inline(never)]
fn use_data(data: &[u8]) {
    for byte in data {
        // Use a volatile read here to ensure that the resulting program
        // actually does the read from data and it doesn't get optimized away.
        unsafe { std::ptr::read_volatile(byte) };
    }
}

#[test]
fn record_own_breakpoints() {
    let vec = vec![0u8; 16];

    let mut sampler = Builder::new()
        .kind(Breakpoint::read_write(vec.as_ptr() as usize as _, 1))
        .sample(SampleType::IP)
        .sample(SampleType::ADDR)
        .sample(SampleType::TID)
        .sample(SampleType::PERIOD)
        .sample_period(1)
        .build_sampler(4096)
        .expect("Failed to build sampler");

    sampler.enable().expect("Failed to enable sampler");

    use_data(&vec);

    sampler.disable().expect("Failed to disable sampler");

    let record = sampler
        .next_record()
        .expect("Sampler did not record any events");
    let record = match record.event {
        RecordEvent::Sample(event) => event,
        _ => panic!("expected a SAMPLE record, got {:?} instead", record.ty),
    };

    eprintln!("record: {:#?}", record);

    assert_eq!(record.addr, Some(vec.as_ptr() as usize as _));
    assert_eq!(record.tid, Some(nix::unistd::gettid().as_raw() as _));
    assert_eq!(record.pid, Some(nix::unistd::getpid().as_raw() as _));
    assert_eq!(record.period, Some(1));
    assert_eq!(record.time, None); // not specified via sample
}
