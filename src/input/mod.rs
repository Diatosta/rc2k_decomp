//! Keyboard, controllers and force feedback, as the game sees them: the table of held keys and
//! the one pending key the menus read, the list of game controllers and their readings, and the
//! force-feedback effects run from what the car is doing. The input thread (`input_thread`) turns
//! every 20 ms; the devices themselves come from SDL through `platform::input`.

mod acquire_device;
pub(crate) mod append_key_name;
mod axis_found;
mod begin_effects;
mod begin_force;
mod clear_device_readings;
mod device_command;
mod device_effects;
pub(crate) mod device_ids;
mod devices;
mod di_harness;
pub(crate) mod di_objects;
mod effect_changes;
mod effect_found;
mod effect_objects;
mod effects;
mod end_effects;
mod end_force;
mod guids;
pub(crate) mod input_thread;
mod input_tick;
mod joystick_found;
pub(crate) mod key_to_char;
pub(crate) mod keys;
mod layouts;
mod list_joysticks;
mod make_effect;
pub(crate) mod menu_pad;
pub(crate) mod osk_draw;
pub(crate) mod owner;
mod poll_device;
pub(crate) mod profiles;
mod read_device;
mod release_device;
pub(crate) mod request_device_release;
pub(crate) mod request_device_setup;
mod set_effect;
mod set_up_device;
mod start_device_effects;
mod start_effects;
pub(crate) mod step_effects_clock;
mod stop_device_effects;
mod stop_effects;
pub(crate) mod store;
pub(crate) mod take_key;
pub(crate) mod typed_text;
pub(crate) mod types;
mod update_car_effects;
mod update_effects;
pub(crate) mod yes_no_for_key;
pub(crate) mod yes_no_key;
