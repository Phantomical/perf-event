use perf_event::data::Record;
use perf_event::events::Tracepoint;
use perf_event::{Builder, CpuPid};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut counter = Builder::new(Tracepoint::with_name("net/net_dev_xmit")?)
        .targeting(CpuPid::AnyProcessOneCpu { cpu: 0 })
        .build()?
        .sampled(8192)?;

    counter.enable()?;

    while let Some(record) = counter.next_blocking(None) {
        println!("received event");
        if let Ok(Record::Sample(sample)) = record.parse_record() {
            dbg!(sample);
        }
    }

    Ok(())
}
