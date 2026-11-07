//! `menu::edit_savegame_name` (0x46D685): one step of typing the name of a saved game; Enter
//! saves the game.
//!
//! The name is the highlighted entry of the saved game list. One pending key is taken: none
//! the cursor is written again; 1 (Escape)    the name from before comes back, the entry ends; 0x1C
//! (Enter)  an empty name is replaced by the one from before; the entry ends, the name is cut at
//! its length and the game is saved in that slot (`saves::write_savegame_slot`); 0x6E          the
//! name is cleared; 0x0E          the last character goes (`backspace_savegame_name`); any other
//! `key_to_char`; a character is appended unless the length is already above the maximum, and taken
//! off again if the name then measures 0xEC pixels or more. After anything but Escape and Enter the
//! cursor (string 4) is written after the name if `dword_7559A8` is 1, then a NUL. Called from the
//! screen loop `menu::save_championship_loop` while `dword_7556A8` is 1. Refuses, before the key is
//! taken: a name past the list, a length outside 0 .. 0x1F, no keyboard layout.

use crate::errors::{BadInput, Refused};
use crate::input::key_to_char::{has_layout, key_to_char};
use crate::input::take_key::take_key;
use crate::menu::add_text_length::add_text_length;
use crate::menu::backspace_savegame_name::backspace_savegame_name;
use crate::menu::clear_text::clear_text;
use crate::menu::name_lists::{NAME_LEN, savegame_name};
use crate::menu::owner::Menus;
use crate::menu::owner::Tables;
use crate::menu::restore_text::restore_text;
use crate::menu::text_entry::{length, set_length};
use crate::saves::write_savegame_slot::write_savegame_slot;
use crate::service::state::menu_item;
use crate::strings::append_string_by_id::push_string;
use crate::text::draw_text::Canvas;
use crate::text::measure_text::measure_text_at;

// ---- game data ----

const CURSOR_STRING: u32 = 4;
const MEASURING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::MEASURE;

// ---- Rust version ----

fn end_name(name: crate::place::Place) -> Result<(), BadInput> {
    name.add(length() as u32).write(&[0], "a saved game's name")
}

/// The slot of the highlighted saved game: `page * 10 + item - 1`.
fn highlighted_slot(tables: &Tables) -> u32 {
    tables
        .pages
        .save_page
        .get()
        .wrapping_mul(10)
        .wrapping_add(menu_item())
        .wrapping_sub(1)
}

/// Saves the game in `slot`. Whether it worked is not looked at, as in the original. A slot
/// or a state the Rust writer refuses ends the program with the reason (before 2026-10-24
/// it called the original's writer by address, which the standalone does not have: it
/// ended as "not ported").
pub fn save_game(slot: u32) {
    if let Err(why) = write_savegame_slot(crate::menu::owner::menus(), slot) {
        crate::framework::log(&format!("saves::write_savegame_slot: slot {slot}"));
        crate::fatal::raise::end("saves::write_savegame_slot", why);
    }
}

/// `save` is called with the slot when Enter ends the entry.
pub fn edit_savegame_name(
    menus: &Menus,
    canvas: &mut dyn Canvas,
    save: &mut dyn FnMut(u32),
) -> Result<(), Refused> {
    let screens = &menus.screens;
    // Everything that could be refused, before the key is taken.
    let name = savegame_name()?;
    if !(0..NAME_LEN as i32).contains(&length()) {
        return Err(BadInput("text length outside the name").into());
    }
    if !has_layout() {
        return Err(BadInput("no keyboard layout").into());
    }

    let key = take_key();
    // Converted: `rc2k_core::menu::text_entry` (`key`, `appends`, `too_wide`).
    use rc2k_core::menu::text_entry::{self as entry, Key};
    match entry::key(key) {
        Key::None => {}
        Key::Escape => {
            restore_text(name)?;
            screens.option_work.typing.set(0u32);
            return Ok(());
        }
        Key::Enter => {
            if length() == 0 {
                restore_text(name)?;
                add_text_length(menus, name)?;
            }
            screens.option_work.typing.set(0u32);
            end_name(name)?;
            save(highlighted_slot(&menus.tables));
            end_name(name)?;
            screens.option_work.typing.set(0u32);
            return Ok(());
        }
        Key::Clear => clear_text(menus, name)?,
        Key::Backspace => backspace_savegame_name(menus)?,
        Key::Other(key) => {
            let c = key_to_char(key as u32)? as u8;
            let len = length();
            if entry::appends(c, len, screens.loop_flags.text_max.get() as i32) {
                name.add(len as u32).write(&[c], "a saved game's name")?;
                set_length(menus, len.wrapping_add(1));
                let measured = measure_text_at(
                    MEASURING_FONT,
                    &name.game_ptr().bytes("a saved game's name")?,
                    len,
                    canvas,
                )?;
                if entry::too_wide(measured.width as i32) {
                    backspace_savegame_name(menus)?;
                }
            }
        }
    }
    let mut tail = Vec::new();
    if screens.draw_work.cursor_on.get() == 1 {
        push_string(&mut tail, CURSOR_STRING)?;
    }
    tail.push(0);
    name.add(length() as u32)
        .write(&tail, "a saved game's name")?;
    Ok(())
}

// pusha; call take_key
