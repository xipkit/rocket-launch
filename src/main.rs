//! Kestrel launch sequencer: walks the terminal count, honours holds, and
//! hands off to the flight computer at T-0.

mod compress;
mod sequencer;
mod telemetry;

use sequencer::{Phase, Sequencer};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let manifest = args
        .iter()
        .position(|a| a == "--mission")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "mission.toml".to_string());

    let mut seq = Sequencer::from_manifest(&manifest)?;
    println!("{} on {} · window {}", seq.mission(), seq.pad(), seq.window());

    while seq.phase() != Phase::Liftoff {
        let frame = telemetry::sample(seq.t());
        match seq.step(&frame) {
            Ok(Phase::Hold { reason, .. }) => println!("HOLD  T{:+} {reason}", seq.t()),
            Ok(phase) => println!("{:<6} T{:+}", phase.label(), seq.t()),
            Err(abort) => {
                eprintln!("ABORT T{:+} {abort}", seq.t());
                return Err(abort.into());
            }
        }
        if !dry_run {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    println!("LIFTOFF");
    Ok(())
}
