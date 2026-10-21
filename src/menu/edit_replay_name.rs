//! `menu::edit_replay_name` (0x46F4E7): one step of typing the name of a replay.
//!
//! In:  nothing
//! Out: carry (the caller skips its end-of-entry checks on it); every register preserved
//! The name is the highlighted entry of the replay list. One pending key is taken:
//!   none          the cursor is written again;
//!   1 (Escape)    the name from before comes back, the entry ends, `dword_758806` = 1;
//!                 carry set;
//!   0x1C (Enter)  an empty name is replaced by the one from before; the entry ends and the
//!                 name is cut at its length; carry clear;
//!   0x6E          clears a name;
//!   0x0E          takes the last character off a name;
//!   any other     `key_to_char`; a character is appended unless the length is already above
//!                 the maximum, and taken off again if the name then measures 0xEC pixels
//!                 or more.
//! After anything but Escape and Enter the cursor (string 4) is written after the name if
//! `dword_7559A8` is 1, then a NUL; carry is then set exactly when `dword_7559A8` is 0.
//! A slip in the original, reproduced: the clear key, the backspace key and the "too wide"
//! case act on the highlighted name of the SAVED GAME list, not the replay list, while the
//! length they change is the replay entry's.
//! Called from the screen loop `sub_47F77B` while `dword_7556A8` is 1.
//!
//! Screen: the owner said on 2026-10-10 that the replay naming screen comes up after a stage
//! is completed; this hook had 3,889 shadow calls in the session where it was used.
//! Refuses, before the key is taken: a name past either list, a length outside 0 .. 0x1F, no
//! keyboard layout. Manual patch with a private copy (0x22E bytes, 13 calls relocated).

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::blit_glyph::{record_calls, recording_works};
use crate::input::key_to_char::{
    ALT_GR_RVA, ALT_RVA, DEAD_KEY_RVA, LEFT_SHIFT_RVA, RIGHT_SHIFT_RVA, has_layout, key_to_char,
    regions as key_regions,
};
use crate::input::take_key::{PENDING_KEY_RVA, pending_key, take_key};
use crate::menu::add_text_length::add_text_length;
use crate::menu::backspace_savegame_name::backspace_savegame_name;
use crate::menu::clear_text::clear_text;
use crate::menu::name_lists::{
    NAME_LEN, REPLAY_PAGE_RVA, SAVEGAME_PAGE_RVA, replay_name, savegame_name,
};
use crate::menu::restore_text::restore_text;
use crate::menu::text_entry::{
    ENTERING_RVA, LENGTH_RVA, MAX_LENGTH_RVA, TEXT_LEN, backup, backup_region, length, set_length,
};
use crate::screens::harness::Left;
use crate::screens::{font_at, font_is_loaded};
use crate::service::harness::{
    Region, at, capture, check_state_with, dispatch_keyed_state, restore,
};
use crate::service::state::MENU_ITEM_RVA;
use crate::strings::append_string_by_id::push_string;
use crate::strings::string_table::{blob, is_table_id};
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas};
use crate::text::measure_text::measure_text_at;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::edit_replay_name", rva(0x46F4E7), thunk, &ORIGINAL)
    .private_copy(0x22E)
    .relocate_calls(&[
        0x01, 0x30, 0xA0, 0xA5, 0xE3, 0x120, 0x12E, 0x138, 0x191, 0x19E, 0x1D4, 0x222, 0x227,
    ])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const CANCELLED_RVA: usize = rva(0x758806); // dword_758806
const CURSOR_ON_RVA: usize = rva(0x7559A8); // dword_7559A8: 1 in the visible half of the cursor's blink

const KEY_ESCAPE: u8 = 1;
const KEY_ENTER: u8 = 0x1C;
const KEY_CLEAR: u8 = 0x6E;
const KEY_BACKSPACE: u8 = 0x0E;
const CURSOR_STRING: u32 = 4;
const MEASURING_FONT: usize = 0x51E824; // dword_51E824

// ---- Rust version ----

unsafe fn end_name(name: *mut u8) {
    unsafe { name.offset(length() as isize).write(0) };
}

/// Returns the carry flag.
pub unsafe fn edit_replay_name(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Everything that could be refused, before the key is taken.
    let name = replay_name()?;
    let other_name = savegame_name()?;
    if !(0..NAME_LEN as i32).contains(&length()) {
        return Err(BadInput("text length outside the name").into());
    }
    if !has_layout() {
        return Err(BadInput("no keyboard layout").into());
    }

    unsafe {
        let key = take_key();
        // Converted: `rc2k_core::menu::text_entry` (`key`, `appends`, `too_wide`).
        use rc2k_core::menu::text_entry::{self as entry, Key};
        match entry::key(key) {
            Key::None => {}
            Key::Escape => {
                restore_text(name);
                game_write(ENTERING_RVA, 0u32);
                game_write(CANCELLED_RVA, 1u32);
                return Ok(true);
            }
            Key::Enter => {
                if length() == 0 {
                    restore_text(name);
                    add_text_length(name)?;
                }
                game_write(ENTERING_RVA, 0u32);
                end_name(name);
                return Ok(false);
            }
            // The saved game's name, not the replay's: the original's slip.
            Key::Clear => clear_text(other_name),
            Key::Backspace => backspace_savegame_name()?,
            Key::Other(key) => {
                let c = key_to_char(key as u32)? as u8;
                let len = length();
                if entry::appends(c, len, game_read(MAX_LENGTH_RVA)) {
                    name.offset(len as isize).write(c);
                    set_length(len.wrapping_add(1));
                    let measured = measure_text_at(font_at(MEASURING_FONT), name, len, canvas)?;
                    if entry::too_wide(measured.width as i32) {
                        backspace_savegame_name()?;
                    }
                }
            }
        }
        let cursor_on: u32 = game_read(CURSOR_ON_RVA);
        let mut tail = Vec::new();
        if cursor_on == 1 {
            push_string(&mut tail, CURSOR_STRING);
        }
        tail.push(0);
        std::ptr::copy_nonoverlapping(tail.as_ptr(), name.offset(length() as isize), tail.len());
        Ok(cursor_on == 0)
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![at(LENGTH_RVA), at(ENTERING_RVA), at(CANCELLED_RVA)];
    regions.extend(key_regions());
    if let Ok(name) = replay_name() {
        regions.push((name as u32, NAME_LEN + 8));
    }
    if let Ok(name) = savegame_name() {
        regions.push((name as u32, NAME_LEN));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let carry = if shadow() {
        unsafe { edit_replay_name(&mut RecordingCanvas::new(false)) }?
    } else {
        unsafe { edit_replay_name(&mut GameCanvas) }?
    };
    Ok(Left::carry(carry))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: the last name of both lists, which are put back ----

unsafe extern "C" fn original_without_drawing(r: *mut Regs) {
    record_calls(true, || unsafe { call_original(r) });
}

fn rust_without_drawing(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe {
        edit_replay_name(&mut RecordingCanvas::new(false))
    }?))
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off");
    }
    if blob().is_null()
        || !is_table_id(CURSOR_STRING)
        || !font_is_loaded(MEASURING_FONT)
        || !has_layout()
    {
        return t.skip("no strings, measuring font or keyboard layout");
    }
    let font = font_at(MEASURING_FONT);
    let saved_font = unsafe { font.read_unaligned() };
    let inputs = [
        at(REPLAY_PAGE_RVA),
        at(SAVEGAME_PAGE_RVA),
        at(MENU_ITEM_RVA),
        at(CURSOR_ON_RVA),
        backup_region(),
        (game_addr(PENDING_KEY_RVA) as u32, 1),
        (game_addr(LEFT_SHIFT_RVA) as u32, 1),
        (game_addr(RIGHT_SHIFT_RVA) as u32, 1),
        (game_addr(ALT_RVA) as u32, 1),
        (game_addr(ALT_GR_RVA) as u32, 1),
    ];
    let saved_inputs = capture(&inputs);
    // Item 20 of replay page 4 and of saved game page 8 are both entry 99.
    unsafe {
        game_write(REPLAY_PAGE_RVA, 4u32);
        game_write(SAVEGAME_PAGE_RVA, 8u32);
        game_write(MENU_ITEM_RVA, 20u32);
    }
    let (Ok(name), Ok(other_name)) = (replay_name(), savegame_name()) else {
        restore(&inputs, &saved_inputs);
        return t.skip("the test entries are not in the lists");
    };
    let regions = regions();
    let saved = capture(&regions);

    let wide = [b'W'; 29];
    let names: [(&[u8], i32); 5] = [
        (b"abc", 3),
        (b"", 0),
        (b"abcdefghijklmnopqrstuvwxyz1234", 30),
        (&wide, 29),
        (b"x", 1),
    ];
    for key in [
        KEY_ESCAPE,
        KEY_ENTER,
        KEY_CLEAR,
        KEY_BACKSPACE,
        0x1E,
        0x39,
        0x9E,
        0,
    ] {
        for (text, len) in names {
            for cursor_on in [0u32, 1, 2] {
                let prepare = || unsafe {
                    font.write_unaligned(saved_font);
                    name.write_bytes(0xAA, NAME_LEN + 8);
                    std::ptr::copy_nonoverlapping(text.as_ptr(), name, text.len());
                    name.add(text.len()).write(0);
                    other_name.write_bytes(b's', NAME_LEN);
                    set_length(len);
                    backup().write_bytes(0, TEXT_LEN);
                    std::ptr::copy_nonoverlapping(b"before".as_ptr(), backup(), 6);
                    game_write(ENTERING_RVA, 1u32);
                    game_write(CANCELLED_RVA, 0x5EEDu32);
                    game_write(CURSOR_ON_RVA, cursor_on);
                    game_write(DEAD_KEY_RVA, 0u8);
                    for flag in [LEFT_SHIFT_RVA, RIGHT_SHIFT_RVA, ALT_RVA, ALT_GR_RVA] {
                        game_write(flag, 0u8);
                    }
                    pending_key().store(key, Ordering::SeqCst);
                };
                check_state_with(
                    &mut t,
                    &format!(
                        "key={key:#x} name={:?} cursor_on={cursor_on}",
                        String::from_utf8_lossy(text)
                    ),
                    original_without_drawing,
                    Regs::SENTINEL,
                    &prepare,
                    &regions,
                    &rust_without_drawing,
                    &|| vec![pending_key().load(Ordering::SeqCst)],
                );
            }
        }
    }

    unsafe { font.write_unaligned(saved_font) };
    restore(&regions, &saved);
    restore(&inputs, &saved_inputs);
    t.finish()
}
