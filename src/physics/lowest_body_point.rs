//! `physics::lowest_body_point` (0x45D6DB): with more than four probe points
//! (`probe_count`), the deepest of the body's (points 4 .., their depths
//! `probe_height`[n]`, from 0x640000 down by the sign of the difference) into
//! `lowest_depth` and its number into `lowest_point` (-1 for none); then `wheel_share` =
//! `lowest_depth` / 65536 (with four or fewer, of what it held).
//!
//! Callers: `physics::feel_ground`, `physics::find_clear_place` (twice), `physics::crash_contact`,
//! `physics::ground_contact`. Refuses (before anything) a count above 0x40 (the depths' room).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::race::numbers_store::Numbers;
use crate::session::Cell32;

/// On the variables' cells `v`.
pub fn lowest_at(numbers: &Numbers, v: &[&dyn Cell32]) -> Result<(), Refused> {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    let n = get(0);
    if n > 0x40 {
        return Err(BadInput("more than 0x40 probe points (the depths' room)").into());
    }
    if n > 4 {
        put(1, 0x64_0000);
        put(2, u32::MAX);
        for k in 4..n {
            let depth = get(4 + k as usize);
            if (depth.wrapping_sub(get(1)) as i32) < 0 {
                put(1, depth);
                put(2, k);
            }
        }
    }
    put(3, x87::int_over(get(1) as i32, &numbers.k65536).to_bits());
    Ok(())
}

/// The addresses: the count, the lowest, its number, the share, then the 0x40 depths.
fn game<'a>(car: &'a Car, contact: &'a ContactWork) -> Vec<&'a dyn Cell32> {
    let mut v = vec![
        &car.probe_count as &dyn Cell32,
        &contact.lowest_depth,
        &contact.lowest_point,
        &contact.wheel_share,
    ];
    v.extend(
        contact.probe_height[..0x40]
            .iter()
            .map(|c| c as &dyn Cell32),
    );
    v
}

/// The exact version, for `physics::feel_ground`, `ground_contact`, `find_clear_place`,
/// `crash_contact`.
pub fn lowest_body_point(
    car: &Car,
    numbers: &Numbers,
    contact: &ContactWork,
) -> Result<(), Refused> {
    lowest_at(numbers, &game(car, contact))
}
