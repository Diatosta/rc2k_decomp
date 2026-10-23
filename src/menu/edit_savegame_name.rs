//! `menu::edit_savegame_name` (0x46D685): one step of typing the name of a saved game; Enter
//! saves the game.
//!
//! In:  nothing
//! Out: every register preserved (carry is set on Escape; the caller does not read it)
//! The name is the highlighted entry of the saved game list. One pending key is taken:
//!   none          the cursor is written again;
//!   1 (Escape)    the name from before comes back, the entry ends;
//!   0x1C (Enter)  an empty name is replaced by the one from before; the entry ends, the
//!                 name is cut at its length and the game is saved in that slot
//!                 (`saves::write_savegame_slot`);
//!   0x6E          the name is cleared;
//!   0x0E          the last character goes (`backspace_savegame_name`);
//!   any other     `key_to_char`; a character is appended unless the length is already above
//!                 the maximum, and taken off again if the name then measures 0xEC pixels
//!                 or more.
//! After anything but Escape and Enter the cursor (string 4) is written after the name if
//! `dword_7559A8` is 1, then a NUL.
//! Called from the screen loop `sub_47F034` while `dword_7556A8` is 1.
//! Refuses, before the key is taken: a name past the list, a length outside 0 .. 0x1F, no
//! keyboard layout. Manual patch with a private copy (0x20A bytes, 14 calls relocated).
//! In shadow mode the game is saved once, by the original; the replayed Rust version leaves
//! the save out. The self-test runs a copy of the original whose call to the save function
//! goes to a stand-in that notes the slot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::input::key_to_char::{has_layout, key_to_char};
use crate::input::take_key::take_key;
use crate::menu::add_text_length::add_text_length;
use crate::menu::backspace_savegame_name::backspace_savegame_name;
use crate::menu::clear_text::clear_text;
use crate::menu::name_lists::{NAME_LEN, SAVEGAME_PAGE_RVA, savegame_name};
use crate::menu::restore_text::restore_text;
use crate::menu::text_entry::{ENTERING_RVA, MAX_LENGTH_RVA, length, set_length};
use crate::saves::write_savegame_slot::write_savegame_slot;
use crate::screens::font_at;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use crate::service::state::menu_item;
use crate::strings::append_string_by_id::push_string;
use crate::text::draw_text::{Canvas, GameCanvas};
use crate::text::measure_text::measure_text_at;

// ---- game data ----

const SAVE_GAME_RVA: usize = rva(0x4AB558); // saves::write_savegame_slot: ebx = slot
const CURSOR_ON_RVA: usize = rva(0x7559A8); // dword_7559A8: 1 in the visible half of the cursor's blink

const CURSOR_STRING: u32 = 4;
const MEASURING_FONT: usize = 0x51E824; // dword_51E824

// ---- Rust version ----

unsafe fn end_name(name: *mut u8) {
    unsafe { name.offset(length() as isize).write(0) };
}

/// The slot of the highlighted saved game: `page * 10 + item - 1`.
fn highlighted_slot() -> u32 {
    unsafe { game_read::<u32>(SAVEGAME_PAGE_RVA) }
        .wrapping_mul(10)
        .wrapping_add(menu_item())
        .wrapping_sub(1)
}

/// Saves the game in `slot`. Whether it worked is not looked at, as in the original. A slot
/// or a state the Rust writer refuses goes to the original's.
pub fn save_game(slot: u32) {
    if unsafe { write_savegame_slot(slot) }.is_err() {
        let mut regs = Regs {
            ebx: slot,
            ..Regs::default()
        };
        crate::calls::call(SAVE_GAME_RVA, &mut regs);
    }
}

/// `save` is called with the slot when Enter ends the entry.
pub unsafe fn edit_savegame_name(
    canvas: &mut dyn Canvas,
    save: &mut dyn FnMut(u32),
) -> Result<(), Refused> {
    // Everything that could be refused, before the key is taken.
    let name = savegame_name()?;
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
                return Ok(());
            }
            Key::Enter => {
                if length() == 0 {
                    restore_text(name);
                    add_text_length(name)?;
                }
                game_write(ENTERING_RVA, 0u32);
                end_name(name);
                save(highlighted_slot());
                end_name(name);
                game_write(ENTERING_RVA, 0u32);
                return Ok(());
            }
            Key::Clear => clear_text(name),
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
        let mut tail = Vec::new();
        if game_read::<u32>(CURSOR_ON_RVA) == 1 {
            push_string(&mut tail, CURSOR_STRING);
        }
        tail.push(0);
        std::ptr::copy_nonoverlapping(tail.as_ptr(), name.offset(length() as isize), tail.len());
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { edit_savegame_name(&mut GameCanvas, &mut save_game) }?;

    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// pusha; call take_key
