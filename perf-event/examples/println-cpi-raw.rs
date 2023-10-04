use perf_event::events::{Raw, Software};
use perf_event::{Builder, ReadFormat};

fn main() -> std::io::Result<()> {
    let insns_retired: Raw = Raw::new(0x08);
    let cpu_cycles: Raw = Raw::new(0x11);

    let mut group = Builder::new(Software::DUMMY)
        .read_format(
            ReadFormat::GROUP
                | ReadFormat::TOTAL_TIME_ENABLED
                | ReadFormat::TOTAL_TIME_RUNNING
                | ReadFormat::ID,
        )
        .any_pid()
        .one_cpu(0)
        .build_group()?;

    let raw_insns_retired = Builder::new(insns_retired)
        .include_kernel()
        .any_pid()
        .one_cpu(0)
        .build_with_group(&mut group)?;

    let raw_cpu_cycles = Builder::new(cpu_cycles)
        .include_kernel()
        .any_pid()
        .one_cpu(0)
        .build_with_group(&mut group)?;

    let vec = (0..=51).collect::<Vec<_>>();

    group.enable()?;
    println!("{:?}", vec);
    group.disable()?;

    let counts = group.read()?;
    println!(
        "cycles / instructions: {} / {} ({:.2} cpi)",
        counts[&raw_cpu_cycles],
        counts[&raw_insns_retired],
        (counts[&raw_cpu_cycles] as f64 / counts[&raw_insns_retired] as f64)
    );

    Ok(())
}
