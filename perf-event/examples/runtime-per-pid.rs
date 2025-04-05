//! This example showcases how to use `PERF_SAMPLE_READ` along with a counter
//! group in order to calculate counter values for all processes on a machine.
//!
//! It is fairly straightforward to do and if you're building some sort of
//! monitoring agent then it can be useful to be able to read a perf counter for
//! every process on the current machine. Doing it the naive way, with a perf
//! counter per process would be both inefficient and wrong.

use std::collections::BTreeMap;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::Context;
use perf_event::data::Record;
use perf_event::events::Software;
use perf_event::{Builder, ReadFormat, SampleFlag};
use perf_event_data::SwitchCpuWide;
use perf_event_open_sys::bindings;

const KB: usize = 1024;
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

fn main() -> anyhow::Result<()> {
    // We want to exit on SIGINT so make sure to set up the handler right away.
    ctrlc::set_handler(|| SHUTDOWN.store(true, Ordering::Relaxed))
        .context("failed to set ctrlc handler")?;

    let cpus = std::thread::available_parallelism()
        .context("failed to get the number of CPUs on the current host")?
        .get();

    println!("our pid: {}", std::process::id());
    println!("hit Ctrl^C when ready to see pid runtimes");

    let counts = std::thread::scope(|scope| {
        let mut handles = Vec::new();

        for cpu in 0..cpus {
            let handle = scope.spawn(move || -> anyhow::Result<_> {
                // Here we build a counter group per CPU in order to read the counter values.
                //
                // The big idea here is:
                // 1. We configure the counter group to emit a sample on every context switch.
                // 2. We configure perf_event to read all the counters in the group at that time
                //    and to include that in the sample.
                // 3. We diff the recorded counter values against the previous ones and
                //    attribute the difference to the process that just switched out.
                //
                // The important parts when setting up the counters for this are:
                // - each counter group records all processes on a single core
                // - include SampleFlag::READ in the sample flags
                // - set read_format to GROUP | ID
                // - and set sample_period to 1
                //
                // Beyond that, you just need to make sure the sample also includes enough
                // information to associate the resulting counter values back to however you are
                // segmenting things. In our case that's the PID, but maybe you'll want the TID
                // or the cgroup id.

                let mut leader = Builder::new(Software::CONTEXT_SWITCHES)
                    .one_cpu(cpu)
                    .any_pid()
                    .include_kernel()
                    .include_hv()
                    .sample(SampleFlag::READ | SampleFlag::TID | SampleFlag::TIME)
                    .sample_period(1)
                    .sample_id_all(true)
                    .read_format(ReadFormat::GROUP | ReadFormat::ID)
                    .wakeup_watermark(16 * KB)
                    .build()
                    .context("failed to build context switch counter")?;

                let mut clock = Builder::new(Software::CPU_CLOCK)
                    .one_cpu(cpu)
                    .any_pid()
                    .include_kernel()
                    .include_hv()
                    .build_with_group(&mut leader)
                    .context("failed to build clock counter")?;

                let mut leader = leader
                    .sampled(32 * KB)
                    .context("failed to mmap the counter sample buffer")?;

                // So this is a bit of a hack. Linux doesn't really provide a way to get the
                // process that is currently running on a given CPU core. This makes it hard to
                // get the right process to associate the first span with.
                //
                // However, if we set our thread affinity so that we can only run on the core we
                // are monitoring then it becomes trivial to figure out since we'll be the ones
                // currently running on the CPU core. It's also generally much easier to see
                // info about the current process, which again makes our lives easier.
                //
                // This isn't strictly necessary when sampling on task switches, since they
                // happen pretty frequently and you're not losing much by waiting for the first
                // one to occur. For less frequent events, such as cgroup switches, you will
                // find that this matters much more.
                set_thread_affinity(cpu).context("failed to set the thread affinity")?;

                // Make sure to read the counters before enabling the counters so we get their
                // default values. (These should pretty much always be 0).
                let mut prev = clock.read()?;

                leader
                    .enable_group()
                    .context("failed to enable the counter group")?;

                let mut counts: BTreeMap<u32, u64> = BTreeMap::new();

                let mut process_record = |record: perf_event::Record| {
                    let parsed = match record.parse_record() {
                        Ok(parsed) => parsed,
                        Err(e) => {
                            eprintln!("warning: parsing error while parsing perf record: {e}");
                            return;
                        }
                    };

                    let sample_id = record.parse_sample_id().unwrap_or_default();
                    let time = sample_id.time().unwrap();
                    let time = chrono::DateTime::from_timestamp_nanos(time as _)
                        .with_timezone(&chrono::Local);
                    let ftime = time.format("%s%.9f");

                    let sample = match parsed {
                        Record::Sample(sample) => sample,
                        Record::LostSamples(record) => {
                            eprintln!("warning: cpu {cpu}: lost {} samples", record.lost);
                            return;
                        }
                        Record::SwitchCpuWide(switch) => {
                            let (dir, pid, tid) = match switch {
                                SwitchCpuWide::In { pid, tid } => ("in", pid, tid),
                                SwitchCpuWide::Out { pid, tid } => ("out", pid, tid),
                            };

                            let preempt = (record.misc()
                                & (bindings::PERF_RECORD_MISC_SWITCH_OUT_PREEMPT as u16))
                                != 0;
                            let preempt = match preempt {
                                true => "[preempt]",
                                false => "",
                            };

                            println!("{ftime} switch: {dir: <3} {pid:<7} {tid:<7} {preempt}");
                            return;
                        }
                        _ => return,
                    };

                    // First, update the entry for the process that was switched out.
                    let pid = sample.pid().unwrap();
                    let values = sample.values().unwrap();
                    let clock = values.get_by_id(clock.id()).unwrap().value();
                    *counts.entry(pid).or_default() += clock - prev;

                    // Then, save the values for the next process switch.
                    prev = clock;
                };

                // Until we get a signal we just process records as normal.
                while !SHUTDOWN.load(Ordering::Relaxed) {
                    let record = match leader.next_blocking(Some(Duration::from_millis(100))) {
                        Some(record) => record,
                        None => continue,
                    };

                    process_record(record);
                }

                // Disable the group so that we don't keep getting new records
                leader
                    .disable_group()
                    .context("failed to disable the counter group")?;

                // Now we work our way through the remaining records.
                while let Some(record) = leader.next_record() {
                    process_record(record);
                }

                // Finally, record an entry for the process that was running when we disable the
                // group.
                let clock = clock.read()?;
                *counts.entry(std::process::id()).or_default() += clock - prev;

                Ok(counts)
            });

            handles.push(handle);
        }

        let mut counts: BTreeMap<u32, u64> = BTreeMap::new();
        for handle in handles {
            let cpu_counts = match handle.join() {
                Ok(result) => result?,
                Err(payload) => std::panic::resume_unwind(payload),
            };

            for (pid, count) in cpu_counts {
                *counts.entry(pid).or_default() += count;
            }
        }

        anyhow::Ok(counts)
    })?;

    // Compute the number of digits we'll need to show all the PID values.
    let pid_digits = counts
        .keys()
        .copied()
        .map(|pid| pid.checked_ilog10().unwrap_or(0) as usize + 1)
        .max()
        .unwrap_or(1)
        .max(3);

    let mut counts = counts.into_iter().collect::<Vec<_>>();
    counts.sort_by_key(|&(_, count)| count);

    println!("\n{:<pid_digits$}  total runtime", "pid");
    for (pid, count) in counts {
        let duration = Duration::from_nanos(count);

        println!("{pid: >pid_digits$}  {}", FormattedDuration(duration));
    }

    Ok(())
}

/// Set the thread affinity for this thread to the requested core.
///
/// This will immediately migrate this thread to the requested core and will not
/// allow it to run on any other CPUs.
///
/// # Errors
/// This method errors if
/// - An error occurs when getting the number of available CPUs.
/// - The core requested does not actually exist on this system.
/// - The core requested is greater than 1024.
/// - An error occurs when trying to set the CPU affinity.
fn set_thread_affinity(core: usize) -> anyhow::Result<()> {
    let parallelism = std::thread::available_parallelism()
        .context("failed to get the available parallelism")?
        .get();

    if core >= parallelism {
        anyhow::bail!(
            "attempted to set thread affinity to a nonexistant core (core {} does not exist)",
            core
        );
    }

    // This example only works on machines with < 1024 cores.
    // In practice you can allocate a dynamically-sized cpuset if you need to, but
    // that seems overkill for an example.
    if core >= std::mem::size_of::<libc::cpu_set_t>() * (u8::BITS as usize) {
        anyhow::bail!("core value too large for libc cpuset");
    }

    unsafe {
        // SAFETY: all-zeros is a valid representation for cpu_set_t.
        let mut cpuset: libc::cpu_set_t = std::mem::zeroed();

        // SAFETY: We have validated that core fits within the cpuset above.
        libc::CPU_SET(core, &mut cpuset);

        // SAFETY: &cpuset is a valid pointer and we also pass in the correct size.
        let result = libc::sched_setaffinity(0, std::mem::size_of_val(&cpuset), &cpuset);
        if result < 0 {
            return Err(anyhow::Error::from(io::Error::last_os_error()));
        }
    }

    Ok(())
}

struct FormattedDuration(Duration);

impl std::fmt::Display for FormattedDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nanos = self.0.subsec_nanos() % 1000;
        let micros = self.0.subsec_micros() % 1000;
        let millis = self.0.subsec_millis();
        let secs = self.0.as_secs();

        match self.0 {
            d if d < Duration::from_micros(1) => write!(f, "{nanos}ns"),
            d if d < Duration::from_micros(10) => write!(f, "{micros}.{:0>2}us", nanos / 10),
            d if d < Duration::from_micros(100) => write!(f, "{micros}.{:0>1}us", nanos / 100),
            d if d < Duration::from_millis(1) => write!(f, "{micros}us"),
            d if d < Duration::from_millis(10) => write!(f, "{millis}.{:0>2}ms", micros / 10),
            d if d < Duration::from_millis(100) => write!(f, "{millis}.{:0>1}ms", micros / 100),
            d if d < Duration::from_secs(1) => write!(f, "{millis}ms"),
            d if d < Duration::from_secs(10) => write!(f, "{secs}.{:0>2}s", millis / 10),
            d if d < Duration::from_secs(60) => write!(f, "{secs}.{:0>1}s", millis / 100),
            _ => {
                let hours = secs / 3600;
                let minutes = (secs % 3600) / 60;
                let secs = secs % 60;

                if hours > 0 {
                    write!(f, "{hours}h ")?;
                }

                if minutes > 0 {
                    write!(f, "{minutes}m ")?;
                }

                write!(f, "{secs}s")
            }
        }
    }
}
