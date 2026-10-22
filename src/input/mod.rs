//! Keyboard input.

use crate::framework::GameHook;

pub mod acquire_device;
pub mod append_key_name;
pub mod axes;
pub mod axis_found;
pub mod axis_moved;
pub mod begin_effects;
pub mod begin_force;
pub mod binding_place;
pub mod buttons_to_keys;
pub mod clear_device_readings;
pub mod copy_keys;
pub mod device_command;
pub mod device_effects;
pub mod device_state;
pub mod devices;
pub mod di_harness;
pub mod di_objects;
pub mod effect_0_change;
pub mod effect_1_change;
pub mod effect_2_change;
pub mod effect_3_change;
pub mod effect_4_change;
pub mod effect_5_change;
pub mod effect_changes;
pub mod effect_found;
pub mod effect_objects;
pub mod effects;
pub mod end_effects;
pub mod end_force;
pub mod end_frame;
pub mod find_pressed_keys;
pub mod first_device_state;
pub mod hat_direction;
pub mod hat_reading;
pub mod input_thread;
pub mod input_tick;
pub mod joystick_found;
pub mod key_to_char;
pub mod keys;
pub mod list_joysticks;
pub mod make_effect;
pub mod poll_device;
pub mod read_device;
pub mod release_device;
pub mod repeat_keys;
pub mod request_device_release;
pub mod request_device_setup;
pub mod screenshot_if_key;
pub mod set_effect;
pub mod set_up_device;
pub mod start_device_effects;
pub mod start_effects;
pub mod step_effects_clock;
pub mod stop_device_effects;
pub mod stop_effects;
pub mod take_key;
pub mod take_mouse;
pub mod take_mouse_clicks;
pub mod types;
pub mod unpack_mouse;
pub mod update;
pub mod update_car_effects;
pub mod update_effects;
pub mod yes_no_for_key;
pub mod yes_no_key;

pub static HOOKS: &[&dyn GameHook] = &[
    &acquire_device::HOOK,
    &axis_found::HOOK,
    &begin_force::HOOK,
    &effect_found::HOOK,
    &end_force::HOOK,
    &joystick_found::HOOK,
    &list_joysticks::HOOK,
    &make_effect::HOOK,
    &key_to_char::HOOK,
    &append_key_name::HOOK,
    &axis_moved::HOOK,
    &device_state::HOOK,
    &first_device_state::HOOK,
    &take_key::HOOK,
    &yes_no_key::HOOK,
    &yes_no_for_key::HOOK,
    &unpack_mouse::HOOK,
    &take_mouse::HOOK,
    &copy_keys::HOOK,
    &buttons_to_keys::HOOK,
    &find_pressed_keys::HOOK,
    &repeat_keys::HOOK,
    &take_mouse_clicks::HOOK,
    &screenshot_if_key::HOOK,
    &end_frame::HOOK,
    &update::HOOK,
    &step_effects_clock::HOOK,
    &clear_device_readings::HOOK,
    &binding_place::HOOK,
    &hat_reading::HOOK,
    &hat_direction::HOOK,
    &read_device::HOOK,
    &device_effects::HOOK,
    &release_device::HOOK,
    &set_up_device::HOOK,
    &device_command::HOOK,
    &input_tick::HOOK,
    &set_effect::HOOK,
    &start_device_effects::HOOK,
    &stop_device_effects::HOOK,
    &start_effects::HOOK,
    &stop_effects::HOOK,
    &begin_effects::HOOK,
    &end_effects::HOOK,
    &update_car_effects::HOOK,
    &update_effects::HOOK,
    &request_device_setup::HOOK,
    &request_device_release::HOOK,
    &effect_0_change::HOOK,
    &effect_1_change::HOOK,
    &effect_2_change::HOOK,
    &effect_3_change::HOOK,
    &effect_4_change::HOOK,
    &effect_5_change::HOOK,
];
