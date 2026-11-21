//! With more than four probe points (`probe_count`), the deepest of the body's (points 4 on, their
//! depths `probe_height[n]`, starting from 0x640000) goes into `lowest_depth` and its number into
//! `lowest_point` (-1 for none); then `wheel_share` = `lowest_depth` / 65536 (with four or fewer,
//! of what it held).
//!
//! Refuses (before anything) a count above 0x40 (the depths' room).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::car_store::Car;
use crate::physics::contact_store::{ContactWork, ProbeLists};
use crate::race::numbers_store::Numbers;

/// The lowest of the probe points past the first four (its depth and number), and the
/// wheels' share from its depth.
pub(super) fn lowest_body_point(
    car: &Car,
    numbers: &Numbers,
    contact: &ContactWork,
    probes: &ProbeLists,
) -> Result<(), Refused> {
    let n = car.probe_count.get();
    if n > 0x40 {
        return Err(BadInput("more than 0x40 probe points (the depths' room)").into());
    }
    let (lowest, number) = (&contact.lowest_depth, &contact.lowest_point);
    if n > 4 {
        lowest.set(0x64_0000);
        number.set(u32::MAX);
        for k in 4..n {
            let depth = probes.probe_height[k as usize];
            if (depth.wrapping_sub(lowest.get()) as i32) < 0 {
                lowest.set(depth);
                number.set(k);
            }
        }
    }
    contact
        .wheel_share
        .set(x87::int_over(lowest.get() as i32, &numbers.k65536));
    Ok(())
}
