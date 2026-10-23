//! The input update (`input::update`, 0x420115, IDA 2026-10-16): once a frame in the
//! race and its menus, the mouse and the 0x80 key states turned into what was pressed.
//! `g_keysDown` (0x4F6ADC, by scan code; its first byte the pending key of
//! `input::take_key`) is what the window's messages keep; `g_keysNow` (0x4F6B5C) its copy
//! for the frame, with the mouse's buttons as keys 0x73 and 0x74 and two pairs of keys
//! made one; `g_keysBefore` (0x4F6BDC) last frame's; `g_keysPressed` (0x4F6CDC) those
//! that went down this frame, or auto-repeat (`g_keyRepeat`, 0x4F6D5C: a key held 0x41
//! frames is pressed again each frame).

use crate::framework::{Regs, game_addr, game_read, game_write, rva};

pub const MOUSE_WORD_RVA: usize = rva(0x4F6DF0); // g_mouseWord: buttons, x << 8, y << 20
pub const MOUSE_RVA: usize = rva(0x5C6924); // g_mouseX, g_mouseY
pub const LAST_MOUSE_RVA: usize = rva(0x5C692C); // g_lastMouseX, g_lastMouseY
pub const CLICKS_RVA: usize = rva(0x5C6934); // g_leftClick, g_rightClick
pub const BUTTONS_RVA: usize = rva(0x5C693C); // g_mouseButtons, g_leftButton, g_rightButton
pub const DOWN_RVA: usize = rva(0x4F6ADC); // g_keysDown
pub const NOW_RVA: usize = rva(0x4F6B5C); // g_keysNow
pub const BEFORE_RVA: usize = rva(0x4F6BDC); // g_keysBefore
pub const PRESSED_RVA: usize = rva(0x4F6CDC); // g_keysPressed
pub const REPEAT_RVA: usize = rva(0x4F6D5C); // g_keyRepeat
const KEYS: u32 = rc2k_core::input::frame::KEYS as u32;
const SCREENSHOT_KEY: u32 = rc2k_core::input::frame::SCREENSHOT_KEY as u32; // byte_4F6BAE
pub const SCREENSHOTS_RVA: usize = rva(0x4F52B4); // g_screenshotsOn
pub const FRAME_CLEARED_RVA: [usize; 2] = [rva(0x6138F4), rva(0x6138F8)];

// Converted: `rc2k_core::input::frame`. Here the tables are loaded from the game and only those
// a function writes are stored back (`g_keysDown` never: the window's messages write it).

use rc2k_core::input::frame::KeyTables;

unsafe fn byte(table: usize, n: u32) -> u8 {
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

/// `input::unpack_mouse` (0x4081FC): `g_mouseWord` as ebx the buttons (low byte), ecx the
/// x (bits 8 .. 19), edx the y (bits 20 .. 31), both signed.
pub unsafe fn unpack_mouse(entry: &Regs) -> Regs {
    let (buttons, x, y) =
        rc2k_core::input::frame::unpack_mouse(unsafe { game_read(MOUSE_WORD_RVA) });
    Regs {
        ebx: buttons,
        ecx: x as u32,
        edx: y as u32,
        ..*entry
    }
}

/// `input::take_mouse` (0x42019C): `unpack_mouse` into `g_mouseX`, `g_mouseY`,
/// `g_mouseButtons`, `g_leftButton` (bit 0), `g_rightButton` (bit 1). Leaves the
/// registers as `unpack_mouse`.
pub unsafe fn take_mouse(entry: &Regs) -> Regs {
    let g = unsafe { unpack_mouse(entry) };
    unsafe {
        game_write(MOUSE_RVA, [g.ecx, g.edx]);
        game_write(BUTTONS_RVA, rc2k_core::input::frame::buttons(g.ebx));
    }
    g
}

/// `input::copy_keys` (0x4201EC): `g_keysDown` to `g_keysNow`. Leaves esi and edi past
/// them, ecx 0.
pub unsafe fn copy_keys(entry: &Regs) -> Regs {
    unsafe {
        let mut t = tables();
        t.copy_keys(&game_read(DOWN_RVA));
        game_write(NOW_RVA, t.now);
    }
    Regs {
        esi: game_addr(NOW_RVA) as u32,
        edi: game_addr(BEFORE_RVA) as u32,
        ecx: 0,
        ..*entry
    }
}

/// `input::buttons_to_keys` (0x420200): `g_leftButton` and `g_rightButton` (their low
/// bytes) as keys 0x73 and 0x74. Leaves eax `g_rightButton`.
pub unsafe fn buttons_to_keys(entry: &Regs) -> Regs {
    unsafe {
        let [_, left, right]: [u32; 3] = game_read(BUTTONS_RVA);
        let mut t = tables();
        t.buttons_to_keys(left, right);
        game_write(NOW_RVA, t.now);
        Regs {
            eax: right,
            ..*entry
        }
    }
}

/// `input::find_pressed_keys` (0x420215): `g_keysPressed` the keys down now and not
/// before (their low bits only, a dword at a time), `g_keysBefore` = `g_keysNow`. Leaves
/// ebx 0x80, ecx 0, eax and edx the last dword's pressed and now.
pub unsafe fn find_pressed_keys(entry: &Regs) -> Regs {
    let mut t = tables();
    let (eax, edx) = t.find_pressed();
    unsafe {
        game_write(BEFORE_RVA, t.before);
        game_write(PRESSED_RVA, t.pressed);
    }
    Regs {
        eax,
        edx,
        ebx: KEYS,
        ecx: 0,
        ..*entry
    }
}

/// `input::repeat_keys` (0x420247): for keys 1 .. 0x7F, the count of frames held (0 when
/// up); at 0x41 the key is pressed again and the count goes back to 0x40. Leaves ebx
/// 0x80.
pub unsafe fn repeat_keys(entry: &Regs) -> Regs {
    let mut t = tables();
    t.repeat();
    unsafe {
        game_write(REPEAT_RVA, t.repeat);
        game_write(PRESSED_RVA, t.pressed);
    }
    Regs {
        ebx: KEYS,
        ..*entry
    }
}

/// `input::take_mouse_clicks` (0x42028C): `g_leftClick` and `g_rightClick` from keys
/// 0x73 and 0x74 pressed (low bits). Leaves eax the right.
pub unsafe fn take_mouse_clicks(entry: &Regs) -> Regs {
    let clicks = tables().clicks();
    unsafe { game_write(CLICKS_RVA, clicks) };
    Regs {
        eax: clicks[1],
        ..*entry
    }
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
/// `find_pressed_keys`, `repeat_keys`, `take_mouse_clicks`, `end_frame`. Leaves eax the
/// right click, ebx 0x80, ecx 0, edx the last key dword, esi `g_keysNow`, edi
/// `g_keysBefore`.
pub unsafe fn update(entry: &Regs, shot: Screenshot) -> Regs {
    unsafe {
        let mouse: [u32; 2] = game_read(MOUSE_RVA);
        game_write(LAST_MOUSE_RVA, mouse);
        game_write(CLICKS_RVA, [0u32; 2]);
        game_write(PRESSED_RVA, [0u8; KEYS as usize]);
        let mut g = take_mouse(entry);
        g = copy_keys(&g);
        let mut t = tables();
        t.join_pairs();
        game_write(NOW_RVA, t.now);
        g = buttons_to_keys(&g);
        g = find_pressed_keys(&g);
        g = repeat_keys(&g);
        g = take_mouse_clicks(&g);
        end_frame(&mut || screenshot_if_key(&mut *shot));
        g
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
