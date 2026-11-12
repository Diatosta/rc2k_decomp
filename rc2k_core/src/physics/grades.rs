//! A car block's grades put back (from `physics::reset_car_grades`
//! 0x440A22). The block is the work copy's layout (`g_car`, 0xFE0 bytes); what the fields
//! are is stream A's reading (the block's grades and values).
//!
//! Floats: the two figures are a whole number / 256 x 100: exact in an `f64` and on the
//! x87 at any precision, so the floats are the original's to the bit.

use super::x87::loaded;

/// A setting (a whole number) / `k256` x `k100` (`fild; fdiv; fmul; fstp`): the setup's
/// percentage, by the look of it.
pub fn percentage(setting: i32, k256: f32, k100: f32) -> f32 {
    (setting as f64 / k256 as f64 * k100 as f64) as f32
}

/// A field of the car block that `physics::reset_car_grades` writes. The runs are the block's
/// consecutive fields, counted from the first of each run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// The 51 damage gauges, `damage_failed` first and `tyre_wear` last.
    Gauge(u32),
    /// `engine_power_factor`, `brake_efficiency` (2), `drive_efficiency` (2).
    Efficiency(u32),
    /// `stiffness_mean`.
    StiffnessMean,
    /// `wheel_stiffness`, one per wheel.
    WheelStiffness(u32),
    /// `ride_height_mean`.
    RideHeightMean,
    /// `wheel_ride_height`, one per wheel.
    WheelRideHeight(u32),
    /// `steering_efficiency`, `electrical_failures` (7), `handbrake_failed`,
    /// `gear_change_blocked`, `forced_neutral`, `lost_gear_bits`.
    Failure(u32),
    /// `impact_octant_marks`, one per octant.
    ImpactMark(u32),
    /// `parts_present`.
    PartsPresent,
}

/// `physics::reset_car_grades`' writes into the block (field, dword), in the original's
/// order: the gauges zeroed; the efficiencies `one`; the stiffness fields `a` (the setting
/// at `suspension_stiffness` as a percentage), the ride heights `c` (`ride_height`'s); the
/// failures and the impact marks zeroed; the parts present all set.
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
