//! The race's run (`sub_444CEE` and under it, part R): the frame of a stage being driven.

use crate::framework::GameHook;

pub mod animate_lit_faces;
pub mod animate_stage;
pub mod car;
pub mod car_near_checkpoint;
pub mod checkpoint_taken;
pub mod choose_shown_cars;
pub mod clear_start_state;
pub mod drive_computer_car;
pub mod keep_lead_car_values;
pub mod lead_car_values;
pub mod load_car;
pub mod next_camera_view;
pub mod note_gap_ahead;
pub mod note_gap_behind;
pub mod note_gaps;
pub mod put_back_lead_car_values;
pub mod race_mode_cars;
pub mod read_car_controls;
pub mod set_camera_view;
pub mod set_listener;
pub mod share_all_cars;
pub mod share_car_state;
pub mod start_blend;
pub mod start_car_on_line;
pub mod start_cars_behind;
pub mod store_car;

pub static HOOKS: &[&dyn GameHook] = &[
    &load_car::HOOK,
    &store_car::HOOK,
    &keep_lead_car_values::HOOK,
    &put_back_lead_car_values::HOOK,
    &share_car_state::HOOK,
    &set_listener::HOOK,
    &share_all_cars::HOOK,
    &choose_shown_cars::HOOK,
    &set_camera_view::HOOK,
    &next_camera_view::HOOK,
    &note_gap_behind::HOOK,
    &note_gap_ahead::HOOK,
    &clear_start_state::HOOK,
    &car_near_checkpoint::HOOK,
    &checkpoint_taken::HOOK,
    &start_blend::HOOK,
    &animate_lit_faces::HOOK,
    &animate_stage::HOOK,
    &drive_computer_car::HOOK,
    &read_car_controls::HOOK,
    &start_car_on_line::HOOK,
    &start_cars_behind::HOOK,
    &race_mode_cars::HOOK,
];
