//! Keyboard input.

use crate::framework::GameHook;

pub mod append_key_name;
pub mod buttons_to_keys;
pub mod copy_keys;
pub mod end_frame;
pub mod find_pressed_keys;
pub mod key_to_char;
pub mod keys;
pub mod repeat_keys;
pub mod screenshot_if_key;
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
];
