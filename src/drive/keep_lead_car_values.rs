//! Eight fields of car block 0 (`drive/lead_car_values.rs`) kept aside in `lead_kept`; the effects
//! read `lead_kept` (`effects::stage_record`).

use crate::drive::lead_car_values::read;
use crate::physics::car_store::Car;
use crate::race::owner_frame::Frame;

pub(super) fn keep_lead_car_values(lead: &Car, frame: &Frame) {
    let kept = read(lead);
    frame.frame_work.borrow_mut().lead_kept = kept;
}
