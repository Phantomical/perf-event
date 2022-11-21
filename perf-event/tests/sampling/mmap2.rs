use nix::unistd::SysconfVar;
use perf_event::events::Software;
use perf_event::samples::RecordEvent;
use perf_event::Builder;

#[test]
fn record_executable_mmap2() {
    // Use the pagesize as the mmap size so that the recorded length in the
    // record and the length we allocate are the same.
    let pagesize = nix::unistd::sysconf(SysconfVar::PAGE_SIZE)
        .expect("Unable to get page size")
        .expect("No page size returned") as usize;

    let mut sampler = Builder::new()
        .kind(Software::DUMMY)
        .mmap2(true)
        .build_sampler(4096)
        .expect("Failed to build sampler");

    sampler.enable().expect("Failed to enable sampler");

    let mmap = memmap2::MmapOptions::new()
        .len(pagesize)
        .map_anon()
        .expect("Failed to create anonymous memory map");

    // This should cause the sampler to record a MMAP event
    let mmap = mmap
        .make_exec()
        .expect("Failed to transition memory mapping to be executable");

    sampler.disable().expect("Failed to disable sampler");

    let record = sampler
        .next_record()
        .expect("Sampler did not record any events");
    let record = match record.event {
        RecordEvent::Mmap2(mmap) => mmap,
        _ => panic!("expected a MMAP2 record, got {:?} instead", record.ty),
    };

    eprintln!("record: {:#?}", record);

    assert_eq!(record.addr, mmap.as_ptr() as usize as _);
    assert_eq!(record.pgoff, mmap.as_ptr() as usize as _);

    assert_eq!(record.len, mmap.len() as _);
    assert_eq!(record.pid, nix::unistd::getpid().as_raw() as _);
    assert_eq!(record.tid, nix::unistd::gettid().as_raw() as _);

    // Anonymous mapping so no device numbers or inode
    assert_eq!(record.maj, 0);
    assert_eq!(record.min, 0);
    assert_eq!(record.ino, 0);
    assert_eq!(record.ino_generation, 0);

    // Permissions a R+X but only for the owning process
    assert_eq!(record.prot, 0o005);
    assert_eq!(
        record.flags,
        (libc::MAP_EXECUTABLE | libc::MAP_PRIVATE) as _
    );
}
