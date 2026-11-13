//! Eight fields of car block 0 (`drive/lead_car_values.rs`) kept aside in `lead_kept`; the effects
//! read `lead_kept` (`effects::stage_record`).

use crate::drive::lead_car_values::pairs;
use crate::physics::car_store::Car;
use crate::race::owner_frame::Frame;

pub fn keep_lead_car_values(lead: &Car, frame: &Frame) {
    for (field, place) in pairs(lead, frame) {
        place.set(field.get());
    }
}
