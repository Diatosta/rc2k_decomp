//! `drive::keep_lead_car_values` (0x4564F5): eight fields of car block 0 (0x70FA70 ..,
//! `drive/lead_car_values.rs`) into 0x64165C .. 0x641678 (`push`/`pop`). The marks read
//! `dword_64165C` (`marks::stage_record`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::share_all_cars` (0x456484), first.

use crate::drive::lead_car_values::pairs;

/// The Rust version.
pub fn keep_lead_car_values() {
    for (field, place) in pairs() {
        place.set(field.get());
    }
}
