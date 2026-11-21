//! The eight values `drive::keep_lead_car_values` kept, back into car block 0
//! (`drive/lead_car_values.rs`).

use crate::drive::lead_car_values::write;
use crate::physics::car_store::Car;
use crate::race::owner_frame::Frame;

pub(super) fn put_back_lead_car_values(lead: &Car, frame: &Frame) {
    let kept = frame.frame_work.borrow().lead_kept;
    write(lead, kept);
}
