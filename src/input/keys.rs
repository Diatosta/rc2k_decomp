//! The input update: once a frame in the race and its menus, the mouse and the 0x80 key states
//! turned into what was pressed. `keys_down` (by scan code; its first byte the pending key of
//! `input::take_key`) is what the window's messages keep; `keys_now` its copy for the frame,
//! with the mouse's buttons as keys 0x73 and 0x74 and two pairs of keys made one; `keys_before`
//! last frame's; `keys_pressed` those that went down this frame, or auto-repeat (`key_repeat`:
//! a key held 0x41 frames is pressed again each frame).

use crate::input::owner::Input;
use crate::input::store::Devices;
use crate::input::store::KEYS as KEYS_LEN;
use crate::owners::Owners;
use crate::race::owner::Race;

const SCREENSHOT_KEY: u32 = rc2k_core::input::frame::SCREENSHOT_KEY as u32; // Devices.keys_now

use rc2k_core::input::frame::KeyTables;

fn tables(devices: &Devices) -> KeyTables {
    KeyTables {
        now: devices.keys_now.read(),
        before: devices.keys_before.read(),
        pressed: devices.keys_pressed.read(),
        repeat: devices.key_repeat.read(),
    }
}

/// The mouse word as the buttons (low byte), the x (bits 8 .. 19) and the y (bits 20 .. 31),
/// both signed.
pub fn unpack_mouse(devices: &Devices) -> (u32, i32, i32) {
    rc2k_core::input::frame::unpack_mouse(devices.mouse_word.get())
}

/// `unpack_mouse` into the mouse's position and buttons (bit 0 left, bit 1 right); gives
/// what `unpack_mouse` gives.
fn take_mouse(input: &Input) -> (u32, i32, i32) {
    let (buttons, x, y) = unpack_mouse(&input.devices);
    input.mouse().x.set(x as u32);
    input.mouse().y.set(y as u32);
    let [all, left, right] = rc2k_core::input::frame::buttons(buttons);
    input.mouse().buttons.set(all);
    input.mouse().left_button.set(left);
    input.mouse().right_button.set(right);
    (buttons, x, y)
}

/// Copies `keys_down` to `keys_now`.
pub fn copy_keys(devices: &Devices) {
    let mut t = tables(devices);
    t.copy_keys(&devices.keys_down.read());
    devices.keys_now.write(&t.now);
}

/// The left and right buttons (their low bytes) as keys 0x73 and 0x74; gives the right one.
pub fn buttons_to_keys(input: &Input) -> u32 {
    let (left, right) = (
        input.mouse().left_button.get(),
        input.mouse().right_button.get(),
    );
    let mut t = tables(&input.devices);
    t.buttons_to_keys(left, right);
    input.devices.keys_now.write(&t.now);
    right
}

/// Sets `keys_pressed` to the keys down now and not before (their low bits only, a dword at
/// a time) and `keys_before` = `keys_now`; gives the last dword's pressed and now.
fn find_pressed_keys(devices: &Devices) -> (u32, u32) {
    let mut t = tables(devices);
    let found = t.find_pressed();
    devices.keys_before.write(&t.before);
    devices.keys_pressed.write(&t.pressed);
    found
}

/// For keys 1 .. 0x7F, the count of frames held (0 when
/// up); at 0x41 the key is pressed again and the count goes back to 0x40.
fn repeat_keys(devices: &Devices) {
    let mut t = tables(devices);
    t.repeat();
    devices.key_repeat.write(&t.repeat);
    devices.keys_pressed.write(&t.pressed);
}

/// The left and right clicks from keys 0x73 and 0x74 pressed (low bits); gives the right one.
fn take_mouse_clicks(input: &Input) -> u32 {
    let clicks = tables(&input.devices).clicks();
    input.mouse().left_click.set(clicks[0]);
    input.mouse().right_click.set(clicks[1]);
    clicks[1]
}

pub type Screenshot<'a> = &'a mut dyn FnMut();

/// Takes a screenshot while key 0x52 (Numpad 0) is down in `keys_now`.
fn screenshot_if_key(devices: &Devices, shot: Screenshot) {
    if rc2k_core::input::frame::screenshot_wanted(devices.keys_now.get(SCREENSHOT_KEY as usize)) {
        shot();
    }
}

/// With screenshots on, `check` (`screenshot_if_key`); then the two physics debug words 0.
pub fn end_frame(race: &Race, shots_on: u32, check: Screenshot) {
    let left = rc2k_core::input::frame::end_frame(shots_on, check);
    // Not cleared once the debug overlay's keys are used (`debug::overlay`).
    let kept = crate::debug::overlay::words();
    for ((cell, value), kept) in race
        .shared
        .physics_tables
        .get()
        .debug
        .iter()
        .zip(left)
        .zip(kept)
    {
        cell.set(value | kept);
    }
}

/// The mouse kept as last frame's, the clicks and `keys_pressed`
/// 0; `take_mouse`, `copy_keys`, each pair of keys (0x1D and 0x61, 0x2A
/// and 0x36) made one (both the `or` of the two), `buttons_to_keys`,
/// `find_pressed_keys`, `repeat_keys`, `take_mouse_clicks`, `end_frame`. The right click
/// and the last key dword are returned.
pub fn update_input(o: Owners<'static>) -> (u32, u32) {
    update_input_in(o.input, o.race, o.gfx.shots.on.get(), &mut || {
        game_screenshot(o.gfx, o.system);
    })
}

fn update_input_in(input: &Input, race: &Race, shots_on: u32, shot: Screenshot) -> (u32, u32) {
    input.mouse().last_x.set(input.mouse().x.get());
    input.mouse().last_y.set(input.mouse().y.get());
    input.mouse().left_click.set(0);
    input.mouse().right_click.set(0);
    input.devices.keys_pressed.write(&[0u8; KEYS_LEN]);
    take_mouse(input);
    copy_keys(&input.devices);
    let mut t = tables(&input.devices);
    t.join_pairs();
    input.devices.keys_now.write(&t.now);
    buttons_to_keys(input);
    let (_, pressed) = find_pressed_keys(&input.devices);
    repeat_keys(&input.devices);
    let right = take_mouse_clicks(input);
    // The debug overlay's keys, taken from the game (`debug::overlay`).
    crate::debug::overlay::take_keys(&input.devices);
    end_frame(race, shots_on, &mut || {
        screenshot_if_key(&input.devices, &mut *shot);
    });
    (right, pressed)
}

fn game_screenshot(gfx: &crate::gfx::owner::Gfx, system: &crate::system::owner::System) {
    if let Err(e) = crate::gfx::take_screenshot::take_screenshot(gfx, system) {
        crate::fatal::raise::refused("gfx::take_screenshot", e);
    }
}
