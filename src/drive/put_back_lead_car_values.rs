//! `drive::put_back_lead_car_values` (0x456556): the eight values
//! `drive::keep_lead_car_values` kept (0x64165C ..) back into car block 0 (0x70FA70 ..,
//! `drive/lead_car_values.rs`; `push`/`pop`).
//!
//! In:  nothing
//! Out: every register kept; no flags read (`drive::share_all_cars` returns next)
//!
//! Caller: `drive::share_all_cars` (0x456484), last.

use crate::drive::lead_car_values::pairs;
use crate::framework::{game_read, rva};

/// The Rust version.
pub fn put_back_lead_car_values() {
    for (field, place) in pairs() {
        field.set(unsafe { game_read(rva(place)) });
    }
}
