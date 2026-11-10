//! `drive::put_back_lead_car_values` (0x456556): the eight values `drive::keep_lead_car_values`
//! kept (0x64165C ..) back into car block 0 (0x70FA70 .., `drive/lead_car_values.rs`;
//! `push`/`pop`).
//!
//! Caller: `drive::share_all_cars` (0x456484), last.

use crate::drive::lead_car_values::pairs;
use crate::race::owner_frame::Frame;

pub fn put_back_lead_car_values(frame: &Frame) {
    for (field, place) in pairs(frame) {
        field.set(place.get());
    }
}
