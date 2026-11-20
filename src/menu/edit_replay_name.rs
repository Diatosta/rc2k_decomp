//! `menu::edit_replay_name`: one step of typing the name of a replay.
//!
//! The name is the highlighted entry of the replay list. One pending key is taken: none writes
//! the cursor again; Escape (1) restores the name from before, ends the entry and sets
//! `captions.left_by_key` and returns true; Enter (0x1C) replaces an empty name by the one from
//! before, cuts the name at its length and ends the entry; 0x6E clears the name; 0x0E takes the
//! last character off; any other key goes through `key_to_char` and is appended unless the
//! length is above the maximum, and taken off again if the name measures 0xEC pixels or more.
//! After anything but Escape and Enter the cursor (string 4) is written after the name if
//! `draw_work.cursor_on` is set, then a NUL; it returns true exactly when `cursor_on` is 0.
//!
//! The clear key, the backspace key and the "too wide" case act on the replay's name (the
//! original acted on the saved games' list). Called from `menu::save_replay_loop` while `entry.typing` is
//! set. Refuses, before the key is taken: a name past the list, a length outside 0 .. 0x1F,
//! no keyboard layout.

use crate::errors::{BadInput, Refused};
use crate::input::take_key::take_key;
use crate::menu::add_text_length::add_text_length;
use crate::menu::backspace_savegame_name::backspace_name;
use crate::menu::clear_text::clear_text;
use crate::menu::name_lists::{NAME_LEN, replay_name};
use crate::menu::owner::Menus;
use crate::menu::restore_text::restore_text;
use crate::menu::text_entry::{length, set_length};
use crate::race::owner::Race;
use crate::strings::append_string_by_id::push_string;
use crate::text::draw_text::Canvas;
use crate::text::measure_text::measure_text_at;

const CURSOR_STRING: u32 = 4;
const MEASURING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::MEASURE;

fn end_name(menus: &Menus, name: crate::memory::place::BytePlace) -> Result<(), BadInput> {
    name.add(length(menus) as u32)
        .write(&[0], "a replay's name")
}

/// Returns true after Escape, or when the cursor is off (see the module doc).
pub(super) fn edit_replay_name(
    input: &crate::input::owner::Input,
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    race: &'static Race,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    // Everything that could be refused, before the key is taken.
    let name = replay_name(menus, race)?;
    if !(0..NAME_LEN as i32).contains(&length(menus)) {
        return Err(BadInput("text length outside the name").into());
    }

    use rc2k_core::menu::text_entry::{self as entry, Key};
    let typed = crate::input::typed_text::take();
    // A character: appended unless the name is full, and taken off again when too wide.
    let mut append = |c: u8| -> Result<(), Refused> {
        let len = length(menus);
        if entry::appends(c, len, menus.screens.loop_flags.text_max.get() as i32) {
            name.add(len as u32).write(&[c], "a replay's name")?;
            set_length(menus, len.wrapping_add(1));
            let measured =
                measure_text_at(MEASURING_FONT, &name.bytes("a replay's name")?, len, canvas)?;
            if entry::too_wide(measured.width as i32) {
                backspace_name(menus, name)?;
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
            menus.screens.entry.typing.set(false);
            menus.tables.captions.left_by_key.set(true);
            return Ok(true);
        }
        Key::Enter => {
            if length(menus) == 0 {
                restore_text(menus, name)?;
                add_text_length(menus, name)?;
            }
            menus.screens.entry.typing.set(false);
            end_name(menus, name)?;
            return Ok(false);
        }
        Key::Clear => clear_text(menus, name)?,
        Key::Backspace => backspace_name(menus, name)?,
        // Printable keys come as typed text, not as keys.
        Key::Other(_) => {}
    }
    let cursor_on = menus.screens.draw_work.cursor_on.is_set();
    let mut tail = Vec::new();
    if cursor_on {
        push_string(so, &mut tail, CURSOR_STRING)?;
    }
    tail.push(0);
    name.add(length(menus) as u32)
        .write(&tail, "a replay's name")?;
    Ok(!cursor_on)
}
