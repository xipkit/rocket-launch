//! Delta encoding for archived downlink frames. Live frames stay raw.

/// Encode a channel as first sample plus deltas, quantised to `step`.
pub fn delta_encode(samples: &[f64], step: f64) -> Vec<i32> {
    let mut out = Vec::with_capacity(samples.len());
    let mut prev = 0.0;
    for (i, &s) in samples.iter().enumerate() {
        let q = (s / step).round();
        out.push(if i == 0 { q as i32 } else { (q - prev) as i32 });
        prev = q;
    }
    out
}

pub fn delta_decode(deltas: &[i32], step: f64) -> Vec<f64> {
    let mut acc = 0i64;
    deltas
        .iter()
        .map(|&d| {
            acc += d as i64;
            acc as f64 * step
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_within_a_step() {
        let samples = [4.01, 4.02, 4.05, 3.98];
        let step = 0.01;
        let decoded = delta_decode(&delta_encode(&samples, step), step);
        for (a, b) in samples.iter().zip(decoded) {
            assert!((a - b).abs() <= step / 2.0 + 1e-9);
        }
    }
}
