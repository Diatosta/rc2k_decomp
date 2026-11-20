//! `menu::edit_savegame_name`: one step of typing the name of a saved game; Enter
//! saves the game.
//!
//! The name is the highlighted entry of the saved game list. One pending key is taken:
//! - none: the cursor is written again;
//! - 1 (Escape): the name from before comes back, the entry ends;
//! - 0x1C (Enter): an empty name is replaced by the one from before; the entry ends, the name is
//!   cut at its length and the game is saved in that slot (`saves::write_savegame_slot`);
//! - 0x6E: the name is cleared;
//! - 0x0E: the last character goes (`backspace_savegame_name`);
//! - any other: `key_to_char`; a character is appended unless the length is already above the
//!   maximum, and taken off again if the name then measures 0xEC pixels or more.
//!
//! After anything but Escape and Enter the cursor (string 4) is written after the name if
//! `draw_work.cursor_on` is set, then a NUL. Called from the screen loop
//! `menu::save_championship_loop` while `entry.typing` is set. Refuses, before the key is
//! taken: a name past the list, a length outside 0 .. 0x1F, no keyboard layout.

use crate::errors::{BadInput, Refused};
use crate::input::take_key::take_key;
use crate::menu::add_text_length::add_text_length;
use crate::menu::backspace_savegame_name::backspace_savegame_name;
use crate::menu::clear_text::clear_text;
use crate::menu::loop_store::LoopFlags;
use crate::menu::name_lists::{NAME_LEN, savegame_name};
use crate::menu::owner::Menus;
use crate::menu::pages_store::MenuPages;
use crate::menu::restore_text::restore_text;
use crate::menu::text_entry::{length, set_length};
use crate::saves::owner::Saves;
use crate::saves::write_savegame_slot::write_savegame_slot;
use crate::strings::append_string_by_id::push_string;
use crate::text::draw_text::Canvas;
use crate::text::measure_text::measure_text_at;

const CURSOR_STRING: u32 = 4;
const MEASURING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::MEASURE;

fn end_name(menus: &Menus, name: crate::memory::place::BytePlace) -> Result<(), BadInput> {
    name.add(length(menus) as u32)
        .write(&[0], "a saved game's name")
}

/// The slot of the highlighted saved game: `page * 10 + item - 1`.
fn highlighted_slot(pages: &MenuPages, loop_flags: &LoopFlags) -> u32 {
    pages
        .save_page
        .get()
        .wrapping_mul(10)
        .wrapping_add(loop_flags.item.get())
        .wrapping_sub(1)
}

/// Saves the game in `slot`. When the file cannot be written the slot's name goes back to the
/// one from before, so the list does not show a game that was not saved. A slot or a state the
/// writer refuses ends the program with the reason.
pub(super) fn save_game(menus: &Menus, saves: &'static Saves, slot: u32) {
    match write_savegame_slot(menus, saves, slot) {
        Ok(true) => {}
        Ok(false) => {
            crate::program_log::log(&format!(
                "saves: the saved game in slot {slot} was not written"
            ));
            // The slot is below 100 here: `write_savegame_slot` took it.
            let _ = restore_text(
                menus,
                crate::saves::write_savegame_slot::slot_name(saves, slot),
            );
        }
        Err(why) => {
            crate::program_log::log(&format!("saves::write_savegame_slot: slot {slot}"));
            crate::fatal::raise::end("saves::write_savegame_slot", why);
        }
    }
}

/// `save` is called with the slot when Enter ends the entry.
pub fn edit_savegame_name(
    input: &crate::input::owner::Input,
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    saves: &'static Saves,
    canvas: &mut dyn Canvas,
    save: &mut dyn FnMut(u32),
) -> Result<(), Refused> {
    let screens = &menus.screens;
    // Everything that could be refused, before the key is taken.
    let name = savegame_name(menus, saves)?;
    if !(0..NAME_LEN as i32).contains(&length(menus)) {
        return Err(BadInput("text length outside the name").into());
    }

    use rc2k_core::menu::text_entry::{self as entry, Key};
    let typed = crate::input::typed_text::take();
    // A character: appended unless the name is full, and taken off again when too wide.
    let mut append = |c: u8| -> Result<(), Refused> {
        let len = length(menus);
        if entry::appends(c, len, screens.loop_flags.text_max.get() as i32) {
            name.add(len as u32).write(&[c], "a saved game's name")?;
            set_length(menus, len.wrapping_add(1));
            let measured = measure_text_at(
                MEASURING_FONT,
                &name.bytes("a saved game's name")?,
                len,
                canvas,
            )?;
            if entry::too_wide(measured.width as i32) {
                backspace_savegame_name(menus, saves)?;
            }
        }
        Ok(())
    };
    for &c in &typed.chars {
        append(c)?;
    }
    let key = take_key(input);
    match entry::key(key) {
        Key::None => {}
        Key::Escape => {
            restore_text(menus, name)?;
            screens.entry.typing.set(false);
            return Ok(());
        }
        Key::Enter => {
            if length(menus) == 0 {
                restore_text(menus, name)?;
                add_text_length(menus, name)?;
            }
            screens.entry.typing.set(false);
            end_name(menus, name)?;
            save(highlighted_slot(
                &menus.tables.pages,
                &menus.screens.loop_flags,
            ));
            return Ok(());
        }
        Key::Clear => clear_text(menus, name)?,
        Key::Backspace => backspace_savegame_name(menus, saves)?,
        // Printable keys come as typed text, not as keys.
        Key::Other(_) => {}
    }
    let mut tail = Vec::new();
    if screens.draw_work.cursor_on.is_set() {
        push_string(so, &mut tail, CURSOR_STRING)?;
    }
    tail.push(0);
    name.add(length(menus) as u32)
        .write(&tail, "a saved game's name")?;
    Ok(())
}

// pusha; call take_key
