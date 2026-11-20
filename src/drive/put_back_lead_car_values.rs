//! The eight values `drive::keep_lead_car_values` kept, back into car block 0
//! (`drive/lead_car_values.rs`).

use crate::drive::lead_car_values::pairs;
use crate::physics::car_store::Car;
use crate::race::owner_frame::Frame;

pub(super) fn put_back_lead_car_values(lead: &Car, frame: &Frame) {
    for (field, place) in pairs(lead, frame) {
        field.set(place.get());
    }
}
