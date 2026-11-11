//! A car block's grades put back (converted from `physics::reset_car_grades`
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

/// `physics::reset_car_grades`' writes into the block (byte offset, dword), in the
/// original's order: +0x29C .. +0x367 zeroed; +0x3F4 .. +0x404 `one`; +0x448 .. +0x458
/// `a` (the setting at `suspension_stiffness` as a percentage), +0x438 .. +0x444 and +0x45C `c`
/// (`ride_height`'s); +0x408 .. +0x434 and +0x64 .. +0x80 zeroed; +0x298 -1.
pub fn reset_writes(one: f32, a: f32, c: f32) -> Vec<(u32, u32)> {
    let mut w: Vec<(u32, u32)> = (0x29C..0x368).step_by(4).map(|o| (o, 0)).collect();
    let one = loaded(one).to_bits();
    w.extend([0x3F4, 0x3F8, 0x3FC, 0x400, 0x404].map(|o| (o, one)));
    w.extend([0x458, 0x448, 0x44C, 0x450, 0x454].map(|o| (o, a.to_bits())));
    w.extend([0x45C, 0x438, 0x43C, 0x440, 0x444].map(|o| (o, c.to_bits())));
    w.extend(
        (0x408..0x438)
            .step_by(4)
            .chain((0x64..0x84).step_by(4))
            .map(|o| (o, 0)),
    );
    w.push((0x298, u32::MAX));
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
        for (o, v) in reset_writes(1.0, 50.0, 25.0) {
            b[o as usize / 4] = v;
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
