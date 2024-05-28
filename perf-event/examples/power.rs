use std::io;
use std::time::Duration;

use perf_event::events::{Dynamic, Software};
use perf_event::{Builder, Counter, Group};
use perf_event_data::ReadFormat;

/// The power perf PMU allows us to read the power consumption of various
/// components of the system in Joules.
///
/// In this example, we use [`Dynamic`] to create counters for each of the
/// energy counters exported by the kernel and print the total number of joules
/// recorded by each across a second. This demonstrates:
/// - creating a counter from pmu and event names,
/// - reading the unit and scale values for each counter, and,
/// - scaling the raw value read by perf to match the unit.
///
/// If you want to see what is included in each of the counters the best place
/// to start is probably the source code for the power PMU kernel module:
/// https://github.com/torvalds/linux/blob/master/arch/x86/events/rapl.c
fn main() -> anyhow::Result<()> {
    let mut group = Builder::new(Software::DUMMY)
        .read_format(ReadFormat::GROUP | ReadFormat::TOTAL_TIME_RUNNING)
        .one_cpu(0)
        .any_pid()
        .exclude_hv(false)
        .exclude_kernel(false)
        .build_group()?;

    let mut cores = Event::new("energy-cores", &mut group)?;
    let mut pkg = Event::new("energy-pkg", &mut group)?;
    let mut psys = Event::new("energy-psys", &mut group)?;
    let mut gpu = Event::new("energy-gpu", &mut group)?;
    let mut ram = Event::new("energy-ram", &mut group)?;

    let duration = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(1.0);

    group.enable()?;
    std::thread::sleep(Duration::from_secs_f64(duration));
    group.disable()?;

    let read = group.read()?;
    let time_running = read.time_running().unwrap();

    println!("Measured power for {:.6}s", time_running.as_secs_f64());
    println!("Package: {:.3} {}", pkg.read()?, pkg.unit);
    println!("Psys:    {:.3} {}", psys.read()?, psys.unit);
    println!("Core:    {:.3} {}", cores.read()?, cores.unit);
    println!("GPU:     {:.3} {}", gpu.read()?, gpu.unit);
    println!("RAM:     {:.3} {}", ram.read()?, ram.unit);

    Ok(())
}

struct Event {
    counter: Counter,
    unit: String,
    scale: f64,
}

impl Event {
    fn new(name: &str, group: &mut Group) -> io::Result<Self> {
        let mut builder = Dynamic::builder("power")?;
        builder.event(name)?;

        Ok(Self {
            unit: builder.unit()?.expect("event had no unit"),
            scale: builder.scale()?.expect("event had no scale"),
            counter: Builder::new(builder.build()?)
                .one_cpu(0)
                .any_pid()
                .exclude_hv(false)
                .exclude_kernel(false)
                .build_with_group(group)?,
        })
    }

    fn read(&mut self) -> io::Result<f64> {
        Ok(self.counter.read()? as f64 * self.scale)
    }
}
