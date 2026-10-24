//! The input update (`input::update`, 0x420115, IDA 2026-10-16): once a frame in the
//! race and its menus, the mouse and the 0x80 key states turned into what was pressed.
//! `g_keysDown` (0x4F6ADC, by scan code; its first byte the pending key of
//! `input::take_key`) is what the window's messages keep; `g_keysNow` (0x4F6B5C) its copy
//! for the frame, with the mouse's buttons as keys 0x73 and 0x74 and two pairs of keys
//! made one; `g_keysBefore` (0x4F6BDC) last frame's; `g_keysPressed` (0x4F6CDC) those
//! that went down this frame, or auto-repeat (`g_keyRepeat`, 0x4F6D5C: a key held 0x41
//! frames is pressed again each frame).

use crate::framework::{Regs, rva};
use crate::gfx::shots::Slot as ShotSlot;
use crate::input::store::DeviceSlot;
use crate::input::store::MouseSlot;
use crate::store::{game_read, game_write};

pub const MOUSE_WORD_RVA: DeviceSlot = DeviceSlot::at(0x4F6DF0); // g_mouseWord: buttons, x << 8, y << 20
pub const MOUSE_RVA: MouseSlot = MouseSlot::at(0x5C6924); // g_mouseX, g_mouseY
pub const LAST_MOUSE_RVA: MouseSlot = MouseSlot::at(0x5C692C); // g_lastMouseX, g_lastMouseY
pub const CLICKS_RVA: MouseSlot = MouseSlot::at(0x5C6934); // g_leftClick, g_rightClick
pub const BUTTONS_RVA: MouseSlot = MouseSlot::at(0x5C693C); // g_mouseButtons, g_leftButton, g_rightButton
pub const DOWN_RVA: DeviceSlot = DeviceSlot::at(0x4F6ADC); // g_keysDown
pub const NOW_RVA: DeviceSlot = DeviceSlot::at(0x4F6B5C); // g_keysNow
pub const BEFORE_RVA: DeviceSlot = DeviceSlot::at(0x4F6BDC); // g_keysBefore
pub const PRESSED_RVA: DeviceSlot = DeviceSlot::at(0x4F6CDC); // g_keysPressed
pub const REPEAT_RVA: DeviceSlot = DeviceSlot::at(0x4F6D5C); // g_keyRepeat
const KEYS: u32 = rc2k_core::input::frame::KEYS as u32;
const SCREENSHOT_KEY: u32 = rc2k_core::input::frame::SCREENSHOT_KEY as u32; // byte_4F6BAE
pub const SCREENSHOTS_RVA: ShotSlot = ShotSlot::at(0x4F52B4); // g_screenshotsOn
pub const FRAME_CLEARED_RVA: [usize; 2] = [rva(0x6138F4), rva(0x6138F8)];

// Converted: `rc2k_core::input::frame`. Here the tables are loaded from the game and only those
// a function writes are stored back (`g_keysDown` never: the window's messages write it).

use rc2k_core::input::frame::KeyTables;

unsafe fn byte<W: crate::store::Where + core::ops::Add<usize, Output = W>>(table: W, n: u32) -> u8 {
    unsafe { game_read(table + n as usize) }
}

fn tables() -> KeyTables {
    unsafe {
        KeyTables {
            now: game_read(NOW_RVA),
            before: game_read(BEFORE_RVA),
            pressed: game_read(PRESSED_RVA),
            repeat: game_read(REPEAT_RVA),
        }
    }
}

/// `input::unpack_mouse` (0x4081FC): `g_mouseWord` as the buttons (low byte), the x
/// (bits 8 .. 19) and the y (bits 20 .. 31), both signed (the original's ebx, ecx, edx).
pub fn unpack_mouse() -> (u32, i32, i32) {
    rc2k_core::input::frame::unpack_mouse(unsafe { game_read(MOUSE_WORD_RVA) })
}

/// `input::take_mouse` (0x42019C): `unpack_mouse` into `g_mouseX`, `g_mouseY`,
/// `g_mouseButtons`, `g_leftButton` (bit 0), `g_rightButton` (bit 1); what `unpack_mouse`
/// gives.
pub fn take_mouse() -> (u32, i32, i32) {
    let (buttons, x, y) = unpack_mouse();
    unsafe {
        game_write(MOUSE_RVA, [x as u32, y as u32]);
        game_write(BUTTONS_RVA, rc2k_core::input::frame::buttons(buttons));
    }
    (buttons, x, y)
}

/// `input::copy_keys` (0x4201EC): `g_keysDown` to `g_keysNow` (the original leaves esi and
/// edi past them, ecx 0).
pub fn copy_keys() {
    unsafe {
        let mut t = tables();
        t.copy_keys(&game_read(DOWN_RVA));
        game_write(NOW_RVA, t.now);
    }
}

/// `input::buttons_to_keys` (0x420200): `g_leftButton` and `g_rightButton` (their low
/// bytes) as keys 0x73 and 0x74; `g_rightButton` (the original's eax).
pub fn buttons_to_keys() -> u32 {
    unsafe {
        let [_, left, right]: [u32; 3] = game_read(BUTTONS_RVA);
        let mut t = tables();
        t.buttons_to_keys(left, right);
        game_write(NOW_RVA, t.now);
        right
    }
}

/// `input::find_pressed_keys` (0x420215): `g_keysPressed` the keys down now and not
/// before (their low bits only, a dword at a time), `g_keysBefore` = `g_keysNow`; the last
/// dword's pressed and now (the original's eax, edx; ebx 0x80, ecx 0).
pub fn find_pressed_keys() -> (u32, u32) {
    let mut t = tables();
    let found = t.find_pressed();
    unsafe {
        game_write(BEFORE_RVA, t.before);
        game_write(PRESSED_RVA, t.pressed);
    }
    found
}

/// `input::repeat_keys` (0x420247): for keys 1 .. 0x7F, the count of frames held (0 when
/// up); at 0x41 the key is pressed again and the count goes back to 0x40 (the original
/// leaves ebx 0x80).
pub fn repeat_keys() {
    let mut t = tables();
    t.repeat();
    unsafe {
        game_write(REPEAT_RVA, t.repeat);
        game_write(PRESSED_RVA, t.pressed);
    }
}

/// `input::take_mouse_clicks` (0x42028C): `g_leftClick` and `g_rightClick` from keys
/// 0x73 and 0x74 pressed (low bits); the right (the original's eax).
pub fn take_mouse_clicks() -> u32 {
    let clicks = tables().clicks();
    unsafe { game_write(CLICKS_RVA, clicks) };
    clicks[1]
}

/// `screenshot_if_key`'s call of `gfx::take_screenshot`.
pub type Screenshot<'a> = &'a mut dyn FnMut();

/// `gfx::screenshot_if_key` (0x43B573): `gfx::take_screenshot` while key 0x52 (Numpad 0) is down
/// (in `g_keysNow`). Every register preserved.
pub unsafe fn screenshot_if_key(shot: Screenshot) {
    // Converted: `rc2k_core::input::frame::screenshot_wanted`.
    if rc2k_core::input::frame::screenshot_wanted(unsafe { byte(NOW_RVA, SCREENSHOT_KEY) }) {
        shot();
    }
}

/// `input::end_frame` (0x43AA91): with `g_screenshotsOn`, `check` (`screenshot_if_key`);
/// then `dword_6138F4` and `dword_6138F8` 0. Every register preserved.
pub unsafe fn end_frame(check: Screenshot) {
    // Converted: `rc2k_core::input::frame::end_frame`.
    unsafe {
        let left = rc2k_core::input::frame::end_frame(game_read(SCREENSHOTS_RVA), check);
        for (at, value) in FRAME_CLEARED_RVA.into_iter().zip(left) {
            game_write(at, value);
        }
    }
}

/// `input::update` (0x420115): the mouse kept as last frame's, the clicks and
/// `g_keysPressed` 0; `take_mouse`, `copy_keys`, each pair of keys (0x1D and 0x61, 0x2A
/// and 0x36) made one (both the `or` of the two), `buttons_to_keys`,
/// `find_pressed_keys`, `repeat_keys`, `take_mouse_clicks`, `end_frame`. The right click
/// and the last key dword (the original's eax and edx).
pub fn update_input(shot: Screenshot) -> (u32, u32) {
    unsafe {
        let mouse: [u32; 2] = game_read(MOUSE_RVA);
        game_write(LAST_MOUSE_RVA, mouse);
        game_write(CLICKS_RVA, [0u32; 2]);
        game_write(PRESSED_RVA, [0u8; KEYS as usize]);
        take_mouse();
        copy_keys();
        let mut t = tables();
        t.join_pairs();
        game_write(NOW_RVA, t.now);
        buttons_to_keys();
        let (_, edx) = find_pressed_keys();
        repeat_keys();
        let right = take_mouse_clicks();
        end_frame(&mut || screenshot_if_key(&mut *shot));
        (right, edx)
    }
}

/// In the game: `screenshot_if_key` with the game's screenshot.
pub fn game_check() {
    unsafe { screenshot_if_key(&mut game_screenshot) };
}

/// In the game: `gfx::take_screenshot`'s Rust version.
pub fn game_screenshot() {
    if let Err(e) = crate::gfx::take_screenshot::take_screenshot() {
        unsafe { crate::fatal::raise::refused_in("gfx::take_screenshot", e, &Regs::default()) };
    }
}

// ---- for the self-tests ----
