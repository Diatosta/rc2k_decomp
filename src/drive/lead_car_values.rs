//! The eight fields of car block 0 that `drive::share_all_cars` keeps around its work:
//! (block field, kept value) pairs.
use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;

/// The lead car's fields in the order `lead_kept` holds them: the camera's view, place and turns,
/// and the 2-or-3 mode flag. `lead` is the car this machine plays (`RaceShared::own_car`).
pub fn fields(lead: &Car) -> [&PlainDword; 8] {
    [
        &lead.camera_view,
        &lead.camera_position[0],
        &lead.camera_position[1],
        &lead.camera_position[2],
        &lead.camera_pitch,
        &lead.camera_heading,
        &lead.camera_roll_turn,
        &lead.camera_mode_is_2_or_3,
    ]
}
