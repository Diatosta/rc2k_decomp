//! `drive::keep_lead_car_values` (0x4564F5): eight fields of car block 0 (0x70FA70 ..,
//! `drive/lead_car_values.rs`) into 0x64165C .. 0x641678 (`push`/`pop`). The marks read
//! `lead_kept` (`effects::stage_record`).
//!
//! Caller: `drive::share_all_cars` (0x456484), first.

use crate::drive::lead_car_values::pairs;
use crate::race::owner_frame::Frame;

pub fn keep_lead_car_values(frame: &Frame) {
    for (field, place) in pairs(frame) {
        place.set(field.get());
    }
}
