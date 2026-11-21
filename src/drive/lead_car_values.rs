//! The eight fields of car block 0 that `drive::share_all_cars` keeps around its work: read as
//! eight words and put back from them.
use crate::physics::car_store::Car;

/// The lead car's fields in the order `lead_kept` holds them: the camera's view, place and turns,
/// and the 2-or-3 mode flag. `lead` is the car this machine plays (`RaceShared::own_car`).
pub fn read(lead: &Car) -> [u32; 8] {
    let place = lead.camera_position.bits();
    [
        lead.camera_view.get(),
        place[0],
        place[1],
        place[2],
        lead.camera_pitch.get(),
        lead.camera_heading.get(),
        lead.camera_roll_turn.get(),
        lead.camera_mode_is_2_or_3.get(),
    ]
}

/// The fields put back from `values`, in the same order.
pub fn write(lead: &Car, values: [u32; 8]) {
    lead.camera_view.set(values[0]);
    lead.camera_position.set_bits_at(0, values[1]);
    lead.camera_position.set_bits_at(1, values[2]);
    lead.camera_position.set_bits_at(2, values[3]);
    lead.camera_pitch.set(values[4]);
    lead.camera_heading.set(values[5]);
    lead.camera_roll_turn.set(values[6]);
    lead.camera_mode_is_2_or_3.set(values[7]);
}
