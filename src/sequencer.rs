//! Terminal count state machine.
//!
//! The count runs from T-600 to T-0 in one-second steps. Holds are entered
//! automatically when a redline trips and released by the operator.

use crate::telemetry::Frame;
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    PadIdle,
    PropellantLoad,
    TerminalCount,
    Hold { at: i64, reason: String },
    Ignition,
    Liftoff,
}

impl Phase {
    pub fn label(&self) -> &'static str {
        match self {
            Phase::PadIdle => "IDLE",
            Phase::PropellantLoad => "LOAD",
            Phase::TerminalCount => "COUNT",
            Phase::Hold { .. } => "HOLD",
            Phase::Ignition => "IGN",
            Phase::Liftoff => "LIFT",
        }
    }
}

#[derive(Debug)]
pub struct Abort {
    pub t: i64,
    pub reason: String,
}

impl fmt::Display for Abort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at T{:+}", self.reason, self.t)
    }
}

impl std::error::Error for Abort {}

#[derive(Deserialize)]
struct Manifest {
    mission: MissionSection,
    sequencer: SequencerSection,
}

#[derive(Deserialize)]
struct MissionSection {
    name: String,
    pad: String,
    #[serde(rename = "window-open")]
    window_open: String,
}

#[derive(Deserialize)]
struct SequencerSection {
    #[serde(rename = "terminal-count-s")]
    terminal_count_s: i64,
    #[serde(rename = "hold-points")]
    hold_points: Vec<i64>,
    #[serde(rename = "auto-abort")]
    auto_abort: bool,
}

pub struct Sequencer {
    manifest: Manifest,
    t: i64,
    phase: Phase,
    holds_taken: usize,
}

/// Redlines checked every second of the terminal count.
const MAX_CHAMBER_PRESSURE_BAR: f64 = 92.0;
const MIN_LOX_PRESSURE_BAR: f64 = 3.2;
const MIN_BATTERY_V: f64 = 27.5;

impl Sequencer {
    pub fn from_manifest(path: &str) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let manifest: Manifest = toml::from_str(&text)?;
        let t = -manifest.sequencer.terminal_count_s;
        Ok(Self {
            manifest,
            t,
            phase: Phase::PadIdle,
            holds_taken: 0,
        })
    }

    pub fn mission(&self) -> &str {
        &self.manifest.mission.name
    }

    pub fn pad(&self) -> &str {
        &self.manifest.mission.pad
    }

    pub fn window(&self) -> &str {
        &self.manifest.mission.window_open
    }

    pub fn t(&self) -> i64 {
        self.t
    }

    pub fn phase(&self) -> Phase {
        self.phase.clone()
    }

    /// Advance one second. Returns the new phase, or the abort that stopped
    /// the count.
    pub fn step(&mut self, frame: &Frame) -> Result<Phase, Abort> {
        if let Some(reason) = self.redline(frame) {
            if self.manifest.sequencer.auto_abort && self.t > -10 {
                return Err(Abort {
                    t: self.t,
                    reason,
                });
            }
            self.holds_taken += 1;
            self.phase = Phase::Hold {
                at: self.t,
                reason,
            };
            return Ok(self.phase.clone());
        }
        self.t += 1;
        self.phase = match self.t {
            t if t <= -300 => Phase::PropellantLoad,
            t if t < -3 => Phase::TerminalCount,
            t if t < 0 => Phase::Ignition,
            _ => Phase::Liftoff,
        };
        Ok(self.phase.clone())
    }

    fn redline(&self, frame: &Frame) -> Option<String> {
        if frame.chamber_pressure > MAX_CHAMBER_PRESSURE_BAR {
            return Some(format!("chamber pressure {:.1} bar", frame.chamber_pressure));
        }
        if frame.lox_pressure < MIN_LOX_PRESSURE_BAR && self.t > -120 {
            return Some(format!("LOX ullage {:.2} bar", frame.lox_pressure));
        }
        if frame.battery_v < MIN_BATTERY_V {
            return Some(format!("bus voltage {:.1} V", frame.battery_v));
        }
        None
    }

    pub fn holds_taken(&self) -> usize {
        self.holds_taken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nominal() -> Frame {
        Frame {
            t: -100,
            chamber_pressure: 0.0,
            lox_pressure: 4.1,
            battery_v: 28.2,
            altitude_m: 0.0,
        }
    }

    #[test]
    fn count_reaches_liftoff() {
        let mut seq = Sequencer::from_manifest("mission.toml").unwrap();
        let frame = nominal();
        while seq.phase() != Phase::Liftoff {
            seq.step(&frame).unwrap();
        }
        assert_eq!(seq.holds_taken(), 0);
    }
}
