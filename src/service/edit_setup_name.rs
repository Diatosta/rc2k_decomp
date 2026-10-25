//! `service::edit_setup_name` (0x46F1A9): one step of typing the name of a setup slot on the
//! "save setup" screen; Enter saves the setup.
//!
//! In:  nothing
//! Out: every register preserved (carry is set on Escape; the caller does not read it)
//! The name is the highlighted slot's entry in the slot list, with the text entry's cursor
//! (string 4) appended while typing. One pending key is taken:
//!   none          the cursor is written again;
//!   1 (Escape)    the name from before comes back, the entry ends;
//!   0x1C (Enter)  an empty name is replaced by the one from before; the entry ends, the
//!                 name is cut at its length and `write_setup_slot` saves the slot (whether
//!                 that worked is not looked at);
//!   0x6E          the name is cleared;
//!   0x0E          the last character goes (`backspace_setup_name`);
//!   any other     `key_to_char`; a character is appended unless the length is already above
//!                 the maximum (`dword_755571`), and taken off again if the name then
//!                 measures 0xEC pixels or more in the font at 0x51E824.
//! Called from the screen loop `sub_46EB7A` while `dword_7556A8` is 1.
//! Refuses, before the key is taken: a slot past 99, a length outside 0 .. 0x1F, no car
//! object, no keyboard layout.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::input::key_to_char::{has_layout, key_to_char};
use crate::input::take_key::take_key;
use crate::menu::add_text_length::add_text_length;
use crate::menu::clear_text::clear_text;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::restore_text::restore_text;
use crate::menu::text_entry::set_length;
use crate::service::backspace_setup_name::{backspace_setup_name, entry};
use crate::service::load_setup_slot::highlighted_slot;
use crate::service::state::current_player;
use crate::service::write_setup_slot::write_setup_slot;
use crate::strings::append_string_by_id::push_string;
use crate::text::draw_text::Canvas;
use crate::text::measure_text::measure_text_at;

// ---- game data ----

const CURSOR_STRING: u32 = 4;
const MEASURING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::MEASURE;

// ---- Rust version ----

/// Writes the cursor and a NUL after the name's `len` characters.
unsafe fn put_cursor(name: *mut u8) {
    unsafe {
        let mut tail = Vec::new();
        push_string(&mut tail, CURSOR_STRING);
        tail.push(0);
        let len = LOOP_FLAGS.text_length.get() as i32;
        std::ptr::copy_nonoverlapping(tail.as_ptr(), name.offset(len as isize), tail.len());
    }
}

unsafe fn end_name(name: *mut u8) {
    unsafe {
        let len = LOOP_FLAGS.text_length.get() as i32;
        name.offset(len as isize).write(0);
    }
}

pub unsafe fn edit_setup_name(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Everything that could be refused, before the key is taken.
    let (name, _) = entry()?;
    let slot = highlighted_slot();
    if current_player().is_none() {
        return Err(BadInput("no current player object").into());
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
                OPTION_WORK.typing.set(0u32);
                return Ok(());
            }
            Key::Enter => {
                if (LOOP_FLAGS.text_length.get() as i32) == 0 {
                    restore_text(name);
                    add_text_length(name)?;
                }
                OPTION_WORK.typing.set(0u32);
                end_name(name);
                write_setup_slot(slot)?;
                end_name(name);
                OPTION_WORK.typing.set(0u32);
                return Ok(());
            }
            Key::Clear => clear_text(name),
            Key::Backspace => backspace_setup_name()?,
            Key::Other(key) => {
                let c = key_to_char(key as u32)? as u8;
                let len = LOOP_FLAGS.text_length.get() as i32;
                if entry::appends(c, len, LOOP_FLAGS.text_max.get() as i32) {
                    name.offset(len as isize).write(c);
                    set_length(len.wrapping_add(1));
                    // x is whatever ecx held: the length before this character.
                    let measured = measure_text_at(MEASURING_FONT.as_ptr(), name, len, canvas)?;
                    if entry::too_wide(measured.width as i32) {
                        backspace_setup_name()?;
                    }
                }
            }
        }
        put_cursor(name);
    }
    Ok(())
}
