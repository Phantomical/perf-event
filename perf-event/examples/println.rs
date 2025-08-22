use perf_event::Builder;
use perf_event::events::Hardware;

fn main() -> std::io::Result<()> {
    let mut counter = Builder::new(Hardware::INSTRUCTIONS).build()?;

    let vec = (0..=51).collect::<Vec<_>>();

    counter.enable()?;
    println!("{:?}", vec);
    counter.disable()?;

    println!("{} instructions retired", counter.read()?);

    Ok(())
}
