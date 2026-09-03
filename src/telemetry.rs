//! Simulated downlink frames. The real thing arrives over UDP at 50 Hz.

#[derive(Clone, Debug, Default)]
pub struct Frame {
    pub t: i64,
    /// Engine chamber pressure in bar (zero before ignition).
    pub chamber_pressure: f64,
    /// Oxidizer tank ullage pressure in bar.
    pub lox_pressure: f64,
    /// Main bus voltage in volts.
    pub battery_v: f64,
    /// GPS altitude above the ellipsoid in metres.
    pub altitude_m: f64,
}

/// A plausible frame for time `t` in the count.
pub fn sample(t: i64) -> Frame {
    let ignition = t >= -3;
    Frame {
        t,
        chamber_pressure: if ignition { 88.5 + (t as f64) * 0.4 } else { 0.0 },
        lox_pressure: 4.0 + ((t as f64) / 600.0).sin() * 0.2,
        battery_v: 28.1 - (t.unsigned_abs() as f64) * 0.0002,
        altitude_m: 0.0,
    }
}

/// Channel names in downlink order; see docs/notes/Telemetry.md.
pub const CHANNELS: [&str; 12] = [
    "pc", "lox_p", "rp1_p", "imu_ax", "imu_ay", "imu_az", "gps_alt", "gps_vel", "bat_v",
    "fts_arm", "valve_state", "temp_eng",
];
