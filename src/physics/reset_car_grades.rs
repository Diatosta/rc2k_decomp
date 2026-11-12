//! `physics::reset_car_grades` (0x440A22): resets a car block (0xFE0 bytes, the work copy's
//! layout): `damage_failed` .. +0x367 zeroed; `engine_power_factor` .. +0x404 = 1.0; the
//! wheel stiffness fields (+0x448 .. +0x458) = `suspension_stiffness` / 256 x 100 and the wheel
//! ride heights (`wheel_ride_height` .. +0x444, +0x45C) = `ride_height` / 256 x 100 (both read from
//! the work copy `g_car`, whatever block is reset: the setup's percentages, by the look of it);
//! `steering_efficiency` .. `lost_gear_bits` and +0x64 .. +0x80 zeroed; `parts_present` = -1.
//! Stream A's reading: the block's grades and values.
//!
//! Takes: the block
//!
//! Callers: `physics::reset_current_car` (the work car `g_car`), and unreferenced code that
//! runs it over every car before `race::grade_all_cars`.

use crate::race::numbers_store::Numbers;
use crate::session::PlainDword;
use rc2k_core::physics::grades::{percentage, reset_writes};

fn put(b: &dyn crate::cells::Store, offset: u32, v: u32) {
    crate::cells::store_value::<u32>(b, offset as usize, v)
        .unwrap_or_else(|e| crate::fatal::raise::end("physics::reset_car_grades", e));
}

/// The block at `b`, the two settings read at `s0c`, `s08` (the same at any precision).
// Converted: `rc2k_core::physics::grades::{percentage, reset_writes}`.
pub fn reset_grades_at(
    numbers: &Numbers,
    b: &dyn crate::cells::Store,
    s0c: &PlainDword,
    s08: &PlainDword,
) {
    let (k256, k100) = (numbers.k256.get(), numbers.hundred.get());
    let a = percentage(s0c.get() as i32, k256, k100);
    let c = percentage(s08.get() as i32, k256, k100);
    for (offset, v) in reset_writes(numbers.one.get(), a, c) {
        put(b, offset, v);
    }
}
