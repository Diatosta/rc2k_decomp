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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::input::key_to_char::{has_layout, key_to_char};
use crate::input::take_key::take_key;
use crate::menu::add_text_length::add_text_length;
use crate::menu::backspace_savegame_name::backspace_savegame_name;
use crate::menu::clear_text::clear_text;
use crate::menu::name_lists::{NAME_LEN, replay_name, savegame_name};
use crate::menu::restore_text::restore_text;
use crate::menu::text_entry::{ENTERING_RVA, MAX_LENGTH_RVA, length, set_length};
use crate::screens::font_at;
use crate::strings::append_string_by_id::push_string;
use crate::text::draw_text::Canvas;
use crate::text::measure_text::measure_text_at;

// ---- game data ----

const CANCELLED_RVA: usize = rva(0x758806); // dword_758806
const CURSOR_ON_RVA: usize = rva(0x7559A8); // dword_7559A8: 1 in the visible half of the cursor's blink

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
