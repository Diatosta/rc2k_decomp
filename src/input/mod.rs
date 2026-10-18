//! Keyboard input.

use crate::framework::GameHook;

pub mod append_key_name;
pub mod axes;
pub mod axis_moved;
pub mod binding_place;
pub mod buttons_to_keys;
pub mod clear_device_readings;
pub mod copy_keys;
pub mod device_command;
pub mod device_effects;
pub mod device_state;
pub mod devices;
pub mod end_frame;
pub mod find_pressed_keys;
pub mod first_device_state;
pub mod hat_direction;
pub mod hat_reading;
pub mod input_thread;
pub mod input_tick;
pub mod key_to_char;
pub mod keys;
pub mod poll_device;
pub mod read_device;
pub mod release_device;
pub mod repeat_keys;
pub mod screenshot_if_key;
pub mod set_up_device;
pub mod step_effects_clock;
pub mod take_key;
pub mod take_mouse;
pub mod take_mouse_clicks;
pub mod unpack_mouse;
pub mod update;
pub mod yes_no_for_key;
pub mod yes_no_key;

pub static HOOKS: &[&dyn GameHook] = &[
    &take_key::HOOK,
    &key_to_char::HOOK,
    &append_key_name::HOOK,
    &axis_moved::HOOK,
    &device_state::HOOK,
    &first_device_state::HOOK,
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
    &poll_device::HOOK,
    &device_effects::HOOK,
    &release_device::HOOK,
    &set_up_device::HOOK,
    &device_command::HOOK,
    &input_tick::HOOK,
    &input_thread::HOOK,
];
