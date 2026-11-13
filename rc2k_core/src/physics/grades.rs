//! A car block's grades put back to their starting values. Floats: the figures are whole
//! numbers / 256 x 100, exact in an `f64` at any precision.

use super::x87::loaded;

/// A setting (a whole number) / `k256` x `k100`: the setup's percentage.
pub fn percentage(setting: i32, k256: f32, k100: f32) -> f32 {
    (setting as f64 / k256 as f64 * k100 as f64) as f32
}

/// A field of the car block that `reset_writes` sets; runs count from the first of each run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// The 51 damage gauges.
    Gauge(u32),
    /// Power factor, brake (2) and drive (2) efficiencies.
    Efficiency(u32),
    /// The mean stiffness.
    StiffnessMean,
    /// The stiffness, one per wheel.
    WheelStiffness(u32),
    /// The mean ride height.
    RideHeightMean,
    /// The ride height, one per wheel.
    WheelRideHeight(u32),
    /// Steering efficiency, electrical failures (7), handbrake, gear-change and neutral flags,
    /// lost gear bits.
    Failure(u32),
    /// The impact marks, one per octant.
    ImpactMark(u32),
    /// The bits of the parts present.
    PartsPresent,
}

/// The writes into the block (field, dword), in order: gauges zeroed; efficiencies `one`;
/// stiffness `a`; ride heights `c`; failures and impact marks zeroed; all parts present.
pub fn reset_writes(one: f32, a: f32, c: f32) -> Vec<(Field, u32)> {
    let mut w: Vec<(Field, u32)> = (0..51).map(|k| (Field::Gauge(k), 0)).collect();
    let one = loaded(one).to_bits();
    w.extend((0..5).map(|k| (Field::Efficiency(k), one)));
    w.push((Field::StiffnessMean, a.to_bits()));
    w.extend((0..4).map(|k| (Field::WheelStiffness(k), a.to_bits())));
    w.push((Field::RideHeightMean, c.to_bits()));
    w.extend((0..4).map(|k| (Field::WheelRideHeight(k), c.to_bits())));
    w.extend((0..12).map(|k| (Field::Failure(k), 0)));
    w.extend((0..8).map(|k| (Field::ImpactMark(k), 0)));
    w.push((Field::PartsPresent, u32::MAX));
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset() {
        assert_eq!(percentage(128, 256.0, 100.0), 50.0);
        assert_eq!(percentage(-20, 256.0, 100.0), -7.8125);
        let mut b = [0xAAAA_AAAAu32; 0x3F8];
        for (field, v) in reset_writes(1.0, 50.0, 25.0) {
            let offset = match field {
                Field::Gauge(k) => 0x29C + 4 * k,
                Field::Efficiency(k) => 0x3F4 + 4 * k,
                Field::StiffnessMean => 0x458,
                Field::WheelStiffness(k) => 0x448 + 4 * k,
                Field::RideHeightMean => 0x45C,
                Field::WheelRideHeight(k) => 0x438 + 4 * k,
                Field::Failure(k) => 0x408 + 4 * k,
                Field::ImpactMark(k) => 0x64 + 4 * k,
                Field::PartsPresent => 0x298,
            };
            b[offset as usize / 4] = v;
        }
        assert_eq!(b[0x29C / 4], 0);
        assert_eq!(b[0x364 / 4], 0);
        assert_eq!(b[0x368 / 4], 0xAAAA_AAAA);
        assert_eq!(b[0x3F4 / 4], 1.0f32.to_bits());
        assert_eq!(b[0x458 / 4], 50.0f32.to_bits());
        assert_eq!(b[0x45C / 4], 25.0f32.to_bits());
        assert_eq!(b[0x80 / 4], 0);
        assert_eq!(b[0x84 / 4], 0xAAAA_AAAA);
        assert_eq!(b[0x298 / 4], u32::MAX);
    }
}
