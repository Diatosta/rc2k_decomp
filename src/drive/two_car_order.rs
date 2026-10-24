//! `drive::two_car_order` (0x460CDA): the two cars' places in a two-car race (the car
//! blocks' +0xFA4, `g_carBlockStore`): -1 for both outside mode 0 (`dword_62389C`) or with
//! one car (`dword_63DB94` 1); left as they are while either has +0xA8 set; else 1 for the
//! one ahead (checkpoint +0x94, then +0x624, each by the sign of the `cmp`), 2 for the
//! other.
//!
//! In:  nothing
//! Out: ebx the last figure compared; the rest kept; no flags read
//!
//! Caller: `drive::replay_step`.

use crate::physics::flow::{g, p};
use rc2k_core::drive::places;

const B: usize = 0x70F3EC; // g_carBlockStore
const NEXT: usize = 0xFE0;

/// The Rust version.
// Converted: `rc2k_core::drive::places::two_car_order`.
pub fn two_car_order() {
    let figure = |car: usize, o: u32| g(B + car * NEXT + o as usize);
    // The last argument is only handed back as `last` (ebx), which no caller reads.
    let order = places::two_car_order(g(0x62389C), g(0x63DB94), figure, 0);
    if let Some([x, y]) = order.places {
        p(B + 0xFA4, x);
        p(B + NEXT + 0xFA4, y);
    }
}
