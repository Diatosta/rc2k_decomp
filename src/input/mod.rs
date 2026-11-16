//! Keyboard, controllers and force feedback, as the game sees them: the table of held keys and
//! the one pending key the menus read, the list of game controllers and their readings, and the
//! force-feedback effects run from what the car is doing. The input thread (`input_thread`) turns
//! every 20 ms; the devices themselves come from SDL through `platform::input`.

pub mod acquire_device;
pub mod append_key_name;
pub mod axes;
pub mod axis_found;
pub mod begin_effects;
pub mod begin_force;
pub mod clear_device_readings;
pub mod device_command;
pub mod device_effects;
pub mod devices;
pub mod di_harness;
pub mod di_objects;
pub mod effect_changes;
pub mod effect_found;
pub mod effect_objects;
pub mod effects;
pub mod end_effects;
pub mod end_force;
pub mod guids;
pub mod input_thread;
pub mod input_tick;
pub mod joystick_found;
pub mod key_to_char;
pub mod keys;
pub mod layouts;
pub mod list_joysticks;
pub mod make_effect;
pub mod menu_pad;
pub mod owner;
pub mod poll_device;
pub mod read_device;
pub mod release_device;
pub mod request_device_release;
pub mod request_device_setup;
pub mod set_effect;
pub mod set_up_device;
pub mod start_device_effects;
pub mod start_effects;
pub mod step_effects_clock;
pub mod stop_device_effects;
pub mod stop_effects;
pub mod store;
pub mod take_key;
pub mod types;
pub mod update_car_effects;
pub mod update_effects;
pub mod yes_no_for_key;
pub mod yes_no_key;
