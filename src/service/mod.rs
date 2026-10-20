//! The service area's logic: what happens between the stages besides drawing.

use crate::framework::GameHook;

pub mod apply_repairs;
pub mod backspace_setup_name;
pub mod begin_edits;
pub mod begin_gear_ratio_edit;
pub mod begin_ride_height_edit;
pub mod begin_setup_name;
pub mod begin_suspension_stiffness_edit;
pub mod begin_tyre_edit;
pub mod choose_repair;
pub mod choose_repair_marked;
pub mod clear_repair_flags;
pub mod clear_repair_marks;
pub mod edit_brake_balance;
pub mod edit_gear_ratio;
pub mod edit_ride_height;
pub mod edit_setup_name;
pub mod edit_steering_sensitivity;
pub mod edit_suspension_stiffness;
pub mod edit_tyre;
pub mod handle_edit_keys;
pub mod harness;
pub mod load_background;
pub mod load_setup_slot;
pub mod load_setup_values;
pub mod next_setup_page;
pub mod pages;
pub mod read_setup_slot;
pub mod repair_body;
pub mod repair_brakes;
pub mod repair_car;
pub mod repair_gearbox;
pub mod repair_lights;
pub mod repair_parts;
pub mod repair_suspension;
pub mod reset_tyre_wear;
pub mod service_area_loop;
pub mod setup_values;
pub mod state;
pub mod step_brake_balance_down;
pub mod step_brake_balance_up;
pub mod step_gear_ratio_down;
pub mod step_gear_ratio_up;
pub mod step_ride_height_down;
pub mod step_ride_height_up;
pub mod step_steering_sensitivity_down;
pub mod step_steering_sensitivity_up;
pub mod step_suspension_stiffness_down;
pub mod step_suspension_stiffness_up;
pub mod step_tyre_down;
pub mod step_tyre_up;
pub mod toggle_gear_ratio_change;
pub mod toggle_repair_choice;
pub mod toggle_ride_height_change;
pub mod toggle_suspension_stiffness_change;
pub mod toggle_tyre_change;
pub mod try_toggle_repair;
pub mod update_repair_total;
pub mod write_setup_slot;

pub static HOOKS: &[&dyn GameHook] = &[
    &load_background::HOOK,
    &load_setup_values::HOOK,
    &choose_repair::HOOK,
    &choose_repair_marked::HOOK,
    &clear_repair_marks::HOOK,
    &step_brake_balance_down::HOOK,
    &step_brake_balance_up::HOOK,
    &step_gear_ratio_down::HOOK,
    &step_gear_ratio_up::HOOK,
    &step_ride_height_down::HOOK,
    &step_ride_height_up::HOOK,
    &step_steering_sensitivity_down::HOOK,
    &step_steering_sensitivity_up::HOOK,
    &step_suspension_stiffness_down::HOOK,
    &step_suspension_stiffness_up::HOOK,
    &step_tyre_down::HOOK,
    &step_tyre_up::HOOK,
    &toggle_gear_ratio_change::HOOK,
    &toggle_ride_height_change::HOOK,
    &toggle_suspension_stiffness_change::HOOK,
    &toggle_tyre_change::HOOK,
    &try_toggle_repair::HOOK,
    &update_repair_total::HOOK,
    &edit_gear_ratio::HOOK,
    &edit_tyre::HOOK,
    &edit_ride_height::HOOK,
    &edit_suspension_stiffness::HOOK,
    &edit_steering_sensitivity::HOOK,
    &edit_brake_balance::HOOK,
    &handle_edit_keys::HOOK,
    &clear_repair_flags::HOOK,
    &read_setup_slot::HOOK,
    &load_setup_slot::HOOK,
    &write_setup_slot::HOOK,
    &backspace_setup_name::HOOK,
    &edit_setup_name::HOOK,
    &begin_setup_name::HOOK,
    &next_setup_page::HOOK,
    &reset_tyre_wear::HOOK,
    &repair_body::HOOK,
    &repair_gearbox::HOOK,
    &repair_brakes::HOOK,
    &repair_lights::HOOK,
    &repair_suspension::HOOK,
    &repair_car::HOOK,
    &apply_repairs::HOOK,
    &service_area_loop::HOOK,
    &begin_gear_ratio_edit::HOOK,
    &begin_tyre_edit::HOOK,
    &begin_ride_height_edit::HOOK,
    &begin_suspension_stiffness_edit::HOOK,
    &toggle_repair_choice::HOOK,
];
