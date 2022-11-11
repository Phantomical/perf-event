use perf_event::events::Software;
use perf_event::samples::RecordEvent;
use perf_event::Builder;

fn generate_mmap_record(pagesize: usize) {
    let mmap = memmap2::MmapOptions::new()
        .len(pagesize)
        .map_anon()
        .expect("Failed to create anonymous memory map");

    // This should cause the sampler to record a MMAP event
    let mmap = mmap
        .make_exec()
        .expect("Failed to transition memory mapping to be executable");

    drop(mmap);
}

#[test]
fn record_too_many_mmap_events() {
    // Use the pagesize as the mmap size so that the recorded length in the
    // record and the length we allocate are the same.
    let pagesize = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    assert!(pagesize >= 0);
    let pagesize = pagesize as usize;

    let mut sampler = Builder::new()
        .kind(Software::DUMMY)
        .mmap(true)
        .build_sampler(pagesize)
        .expect("Failed to build sampler");

    sampler.enable().expect("Faield to enable sampler");

    for _ in 0..2 * pagesize {
        generate_mmap_record(pagesize);
    }

    let mut count = 0;
    let record = loop {
        let record = match sampler.next_record() {
            Some(record) => record,
            None => {
                // The kernel doesn't generate a LOST record until it adds the
                // next record to the ring buffer. So we need to generate a new
                // MMAP event so that the kernel can emit the LOST record.
                generate_mmap_record(pagesize);
                continue;
            }
        };

        match record.event {
            RecordEvent::Mmap(_) => count += 1,
            RecordEvent::Lost(lost) => break lost,
            _ => panic!(
                "expected a MMAP or LOST record, got {:?} instead",
                record.ty
            ),
        }
    };

    sampler.disable().expect("Failed to disable sampler");

    eprintln!("record: {:?}", record);

    assert_eq!(record.lost as usize, 2 * pagesize - count);
}
