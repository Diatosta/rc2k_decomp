//! `menu::edit_player_name` (0x467C63): one step of typing a player's name on the players'
//! details pages.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The name is the current player's (`Player::name`, `menu/player_name.rs`). One pending
//! key is taken:
//!   none          nothing;
//!   1 (Escape)    the name kept at the start comes back, the entry ends;
//!   0x1C (Enter)  an empty name is replaced by the one kept; the entry ends;
//!   0x6E          the name is cleared (`menu::clear_text`);
//!   0x0E          the last character goes;
//!   any other     `key_to_char`; a character is appended unless the length is already above
//!                 the maximum (`dword_755571`, signed), and taken off again if the name then
//!                 measures 0xEC pixels or more in the font at 0x51E824 (measured from the x
//!                 that is the length before the character, as ecx held it).
//! No cursor is written: the page draws it (`screens::draw_name_cursor`). Called from the
//! four `player*_details_loop`s while `dword_7556A8` is 1.
//! Refuses, before the key is taken: no current player, a player without a name, a length
//! outside 0 .. 0x1F, no keyboard layout.
//! Compared in shadow mode (the key replayed). Self-test: the first player with a name,
//! eight keys on five names.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::shadow;
use crate::framework::{Regs, game_read, game_write};
use crate::input::key_to_char::{has_layout, key_to_char};
use crate::input::take_key::take_key;
use crate::menu::clear_text::clear_text;
use crate::menu::player_name::{backspace_name, bring_back_name, name};
use crate::menu::text_entry::{ENTERING_RVA, LENGTH_RVA, MAX_LENGTH_RVA, TEXT_LEN, set_length};
use crate::screens::font_at;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas};
use crate::text::measure_text::measure_text_at;

// ---- game data ----

const MEASURING_FONT: usize = 0x51E824; // dword_51E824

// ---- Rust version ----

pub unsafe fn edit_player_name(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Everything that could be refused, before the key is taken.
    let name = name()?;
    let len: i32 = unsafe { game_read(LENGTH_RVA) };
    if !(0..TEXT_LEN as i32).contains(&len) {
        return Err(BadInput("an entry length outside 0 .. 0x1F").into());
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
                bring_back_name(name);
                game_write(ENTERING_RVA, 0u32);
            }
            Key::Enter => {
                if len == 0 {
                    bring_back_name(name);
                }
                game_write(ENTERING_RVA, 0u32);
            }
            Key::Clear => clear_text(name),
            Key::Backspace => backspace_name(name),
            Key::Other(key) => {
                let c = key_to_char(key as u32)? as u8;
                if entry::appends(c, len, game_read(MAX_LENGTH_RVA)) {
                    name.offset(len as isize).write(c);
                    set_length(len.wrapping_add(1));
                    let measured = measure_text_at(font_at(MEASURING_FONT), name, len, canvas)?;
                    if entry::too_wide(measured.width as i32) {
                        backspace_name(name);
                    }
                }
            }
        }
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    if shadow() {
        unsafe { edit_player_name(&mut RecordingCanvas::new(false)) }?;
    } else {
        unsafe { edit_player_name(&mut GameCanvas) }?;
    }
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
