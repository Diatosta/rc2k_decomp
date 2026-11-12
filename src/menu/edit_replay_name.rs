//! `menu::edit_replay_name` (0x46F4E7): one step of typing the name of a replay.
//!
//! The name is the highlighted entry of the replay list. One pending key is taken:
//! none          the cursor is written again;
//! 1 (Escape)    the name from before comes back, the entry ends, `captions.left_by_key` = 1;
//! carry set;
//! 0x1C (Enter)  an empty name is replaced by the one from before; the entry ends and the
//! name is cut at its length; carry clear;
//! 0x6E          clears a name;
//! 0x0E          takes the last character off a name;
//! any other     `key_to_char`; a character is appended unless the length is already above
//! the maximum, and taken off again if the name then measures 0xEC pixels
//! or more.
//! After anything but Escape and Enter the cursor (string 4) is written after the name if
//! `draw_work.cursor_on` is set, then a NUL; carry is then set exactly when `draw_work.cursor_on` is 0.
//! A slip in the original, reproduced: the clear key, the backspace key and the "too wide"
//! case act on the highlighted name of the SAVED GAME list, not the replay list, while the
//! length they change is the replay entry's.
//! Called from the screen loop `menu::save_replay_loop` while `option_work.typing` is set.
//!
//! Refuses, before the key is taken: a name past either list, a length outside 0 .. 0x1F, no
//! keyboard layout.

use crate::errors::{BadInput, Refused};
use crate::input::key_to_char::{has_layout, key_to_char};
use crate::input::take_key::take_key;
use crate::menu::add_text_length::add_text_length;
use crate::menu::backspace_savegame_name::backspace_savegame_name;
use crate::menu::clear_text::clear_text;
use crate::menu::name_lists::{NAME_LEN, replay_name, savegame_name};
use crate::menu::owner::Menus;
use crate::menu::restore_text::restore_text;
use crate::menu::text_entry::{length, set_length};
use crate::race::owner::Race;
use crate::saves::owner::Saves;
use crate::strings::append_string_by_id::push_string;
use crate::text::draw_text::Canvas;
use crate::text::measure_text::measure_text_at;

// ---- game data ----

const CURSOR_STRING: u32 = 4;
const MEASURING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::MEASURE;

fn end_name(menus: &Menus, name: crate::place::BytePlace) -> Result<(), BadInput> {
    name.add(length(menus) as u32)
        .write(&[0], "a replay's name")
}

/// Returns the carry flag.
pub fn edit_replay_name(
    input: &crate::input::owner::Input,
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    saves: &'static Saves,
    race: &'static Race,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    // Everything that could be refused, before the key is taken.
    let name = replay_name(menus, race)?;
    let other_name = savegame_name(menus, saves)?;
    if !(0..NAME_LEN as i32).contains(&length(menus)) {
        return Err(BadInput("text length outside the name").into());
    }
    if !has_layout() {
        return Err(BadInput("no keyboard layout").into());
    }

    let key = take_key(input);
    // The logic is in `rc2k_core::menu::text_entry` (`key`, `appends`, `too_wide`).
    use rc2k_core::menu::text_entry::{self as entry, Key};
    match entry::key(key) {
        Key::None => {}
        Key::Escape => {
            restore_text(menus, name)?;
            menus.screens.option_work.typing.set(false);
            menus.tables.captions.left_by_key.set(true);
            return Ok(true);
        }
        Key::Enter => {
            if length(menus) == 0 {
                restore_text(menus, name)?;
                add_text_length(menus, name)?;
            }
            menus.screens.option_work.typing.set(false);
            end_name(menus, name)?;
            return Ok(false);
        }
        // The saved game's name, not the replay's: the original's slip.
        Key::Clear => clear_text(menus, other_name)?,
        Key::Backspace => backspace_savegame_name(menus, saves)?,
        Key::Other(key) => {
            let c = key_to_char(input, key as u32)? as u8;
            let len = length(menus);
            if entry::appends(c, len, menus.screens.loop_flags.text_max.get() as i32) {
                name.add(len as u32).write(&[c], "a replay's name")?;
                set_length(menus, len.wrapping_add(1));
                let measured =
                    measure_text_at(MEASURING_FONT, &name.bytes("a replay's name")?, len, canvas)?;
                if entry::too_wide(measured.width as i32) {
                    backspace_savegame_name(menus, saves)?;
                }
            }
        }
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
