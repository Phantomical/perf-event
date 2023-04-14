use perf_event::events::{Cache, CacheOp, CacheResult, Hardware, WhichCache};
use perf_event::{Builder, Group};

fn main() -> std::io::Result<()> {
    const ACCESS: Cache = Cache {
        which: WhichCache::L1D,
        operation: CacheOp::READ,
        result: CacheResult::ACCESS,
    };
    const MISS: Cache = Cache {
        result: CacheResult::MISS,
        ..ACCESS
    };

    let mut group = Group::new()?;
    let access_counter = Builder::new(ACCESS).group(&mut group).build()?;
    let miss_counter = Builder::new(MISS).group(&mut group).build()?;
    let branches = Builder::new(Hardware::BRANCH_INSTRUCTIONS)
        .group(&mut group)
        .build()?;
    let missed_branches = Builder::new(Hardware::BRANCH_MISSES)
        .group(&mut group)
        .build()?;

    // Note that if you add more counters than you actually have hardware for,
    // the kernel will time-slice them, which means you may get no coverage for
    // short measurements. See the documentation.

    let vec = (0..=51).collect::<Vec<_>>();

    group.enable()?;
    println!("{:?}", vec);
    group.disable()?;

    let counts = group.read()?;
    println!(
        "L1D cache misses/references: {} / {} ({:.0}%)",
        counts[&miss_counter],
        counts[&access_counter],
        (counts[&miss_counter] as f64 / counts[&access_counter] as f64) * 100.0
    );

    println!(
        "branch prediction misses/total: {} / {} ({:.0}%)",
        counts[&missed_branches],
        counts[&branches],
        (counts[&missed_branches] as f64 / counts[&branches] as f64) * 100.0
    );

    // You can iterate over a `Counts` value:
    for (id, value) in &counts {
        println!("Counter id {} has value {}", id, value);
    }

    Ok(())
}
