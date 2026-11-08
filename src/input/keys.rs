//! The input update (`input::update`, 0x420115, IDA 2026-10-16): once a frame in the
//! race and its menus, the mouse and the 0x80 key states turned into what was pressed.
//! `g_keysDown` (0x4F6ADC, by scan code; its first byte the pending key of
//! `input::take_key`) is what the window's messages keep; `g_keysNow` (0x4F6B5C) its copy
//! for the frame, with the mouse's buttons as keys 0x73 and 0x74 and two pairs of keys
//! made one; `g_keysBefore` (0x4F6BDC) last frame's; `g_keysPressed` (0x4F6CDC) those
//! that went down this frame, or auto-repeat (`g_keyRepeat`, 0x4F6D5C: a key held 0x41
//! frames is pressed again each frame).

use crate::input::owner::{Input, input};
use crate::input::store::Devices;
use crate::input::store::KEYS as KEYS_LEN;
use crate::race::owner::race;
use rc2k_core::fatal::Context;

const SCREENSHOT_KEY: u32 = rc2k_core::input::frame::SCREENSHOT_KEY as u32; // byte_4F6BAE

// Converted: `rc2k_core::input::frame`. Here the tables are loaded from the game and only those
// a function writes are stored back (`g_keysDown` never: the window's messages write it).

use rc2k_core::input::frame::KeyTables;

fn tables(devices: &Devices) -> KeyTables {
    KeyTables {
        now: devices.keys_now.read(),
        before: devices.keys_before.read(),
        pressed: devices.keys_pressed.read(),
        repeat: devices.key_repeat.read(),
    }
}

/// `input::unpack_mouse` (0x4081FC): `g_mouseWord` as the buttons (low byte), the x
/// (bits 8 .. 19) and the y (bits 20 .. 31), both signed (the original's ebx, ecx, edx).
pub fn unpack_mouse(devices: &Devices) -> (u32, i32, i32) {
    rc2k_core::input::frame::unpack_mouse(devices.mouse_word.get())
}

/// `input::take_mouse` (0x42019C): `unpack_mouse` into `g_mouseX`, `g_mouseY`,
/// `g_mouseButtons`, `g_leftButton` (bit 0), `g_rightButton` (bit 1); what `unpack_mouse`
/// gives.
pub fn take_mouse(input: &Input) -> (u32, i32, i32) {
    let (buttons, x, y) = unpack_mouse(&input.devices);
    input.mouse.x.set(x as u32);
    input.mouse.y.set(y as u32);
    let [all, left, right] = rc2k_core::input::frame::buttons(buttons);
    input.mouse.buttons.set(all);
    input.mouse.left_button.set(left);
    input.mouse.right_button.set(right);
    (buttons, x, y)
}

/// `input::copy_keys` (0x4201EC): `g_keysDown` to `g_keysNow` (the original leaves esi and
/// edi past them, ecx 0).
pub fn copy_keys(devices: &Devices) {
    let mut t = tables(devices);
    t.copy_keys(&devices.keys_down.read());
    devices.keys_now.write(&t.now);
}

/// `input::buttons_to_keys` (0x420200): `g_leftButton` and `g_rightButton` (their low
/// bytes) as keys 0x73 and 0x74; `g_rightButton` (the original's eax).
pub fn buttons_to_keys(input: &Input) -> u32 {
    let (left, right) = (
        input.mouse.left_button.get(),
        input.mouse.right_button.get(),
    );
    let mut t = tables(&input.devices);
    t.buttons_to_keys(left, right);
    input.devices.keys_now.write(&t.now);
    right
}

/// `input::find_pressed_keys` (0x420215): `g_keysPressed` the keys down now and not
/// before (their low bits only, a dword at a time), `g_keysBefore` = `g_keysNow`; the last
/// dword's pressed and now (the original's eax, edx; ebx 0x80, ecx 0).
pub fn find_pressed_keys(devices: &Devices) -> (u32, u32) {
    let mut t = tables(devices);
    let found = t.find_pressed();
    devices.keys_before.write(&t.before);
    devices.keys_pressed.write(&t.pressed);
    found
}

/// `input::repeat_keys` (0x420247): for keys 1 .. 0x7F, the count of frames held (0 when
/// up); at 0x41 the key is pressed again and the count goes back to 0x40 (the original
/// leaves ebx 0x80).
pub fn repeat_keys(devices: &Devices) {
    let mut t = tables(devices);
    t.repeat();
    devices.key_repeat.write(&t.repeat);
    devices.keys_pressed.write(&t.pressed);
}

/// `input::take_mouse_clicks` (0x42028C): `g_leftClick` and `g_rightClick` from keys
/// 0x73 and 0x74 pressed (low bits); the right (the original's eax).
pub fn take_mouse_clicks(input: &Input) -> u32 {
    let clicks = tables(&input.devices).clicks();
    input.mouse.left_click.set(clicks[0]);
    input.mouse.right_click.set(clicks[1]);
    clicks[1]
}

/// `screenshot_if_key`'s call of `gfx::take_screenshot`.
pub type Screenshot<'a> = &'a mut dyn FnMut();

/// `gfx::screenshot_if_key` (0x43B573): `gfx::take_screenshot` while key 0x52 (Numpad 0) is down
/// (in `g_keysNow`). Every register preserved.
pub fn screenshot_if_key(devices: &Devices, shot: Screenshot) {
    // Converted: `rc2k_core::input::frame::screenshot_wanted`.
    if rc2k_core::input::frame::screenshot_wanted(devices.keys_now.get(SCREENSHOT_KEY as usize)) {
        shot();
    }
}

/// `input::end_frame` (0x43AA91): with `g_screenshotsOn`, `check` (`screenshot_if_key`);
/// then `dword_6138F4` and `dword_6138F8` 0. Every register preserved.
pub fn end_frame(check: Screenshot) {
    // Converted: `rc2k_core::input::frame::end_frame`.
    let left = rc2k_core::input::frame::end_frame(crate::gfx::shots::on(), check);
    // Not the original's 0s once the debug overlay's keys are used (`debug::overlay`).
    let kept = crate::debug::overlay::words();
    for ((cell, value), kept) in race()
        .shared
        .physics_tables
        .debug
        .iter()
        .zip(left)
        .zip(kept)
    {
        cell.set(value | kept);
    }
}

/// `input::update` (0x420115): the mouse kept as last frame's, the clicks and
/// `g_keysPressed` 0; `take_mouse`, `copy_keys`, each pair of keys (0x1D and 0x61, 0x2A
/// and 0x36) made one (both the `or` of the two), `buttons_to_keys`,
/// `find_pressed_keys`, `repeat_keys`, `take_mouse_clicks`, `end_frame`. The right click
/// and the last key dword (the original's eax and edx).
pub fn update_input(shot: Screenshot) -> (u32, u32) {
    let input = input();
    input.mouse.last_x.set(input.mouse.x.get());
    input.mouse.last_y.set(input.mouse.y.get());
    input.mouse.left_click.set(0);
    input.mouse.right_click.set(0);
    input.devices.keys_pressed.write(&[0u8; KEYS_LEN]);
    take_mouse(input);
    copy_keys(&input.devices);
    let mut t = tables(&input.devices);
    t.join_pairs();
    input.devices.keys_now.write(&t.now);
    buttons_to_keys(input);
    let (_, edx) = find_pressed_keys(&input.devices);
    repeat_keys(&input.devices);
    let right = take_mouse_clicks(input);
    // Not the original's: the debug overlay's keys, taken from the game (`debug::overlay`).
    crate::debug::overlay::take_keys();
    end_frame(&mut || screenshot_if_key(&input.devices, &mut *shot));
    (right, edx)
}

/// In the game: `gfx::take_screenshot`'s Rust version.
pub fn game_screenshot() {
    if let Err(e) = crate::gfx::take_screenshot::take_screenshot() {
        crate::fatal::raise::refused_in("gfx::take_screenshot", e, &Context::default());
    }
}
