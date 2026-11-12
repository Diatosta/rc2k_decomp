//! `service::edit_setup_name` (0x46F1A9): one step of typing the name of a setup slot on the
//! "save setup" screen; Enter saves the setup.
//!
//! The name is the highlighted slot's entry in the slot list, with the text entry's cursor (string
//! 4) appended while typing. One pending key is taken: none the cursor is written again; 1 (Escape)
//! the name from before comes back, the entry ends; 0x1C (Enter) an empty name is replaced by the
//! one from before; the entry ends, the name is cut at its length and `write_setup_slot` saves the
//! slot (whether that worked is not looked at); 0x6E the name is cleared; 0x0E the last character
//! goes (`backspace_setup_name`); any other `key_to_char`; a character is appended unless the
//! length is already above the maximum (`text_max`), and taken off again if the name then
//! measures 0xEC pixels or more in the font at 0x51E824. Called from the screen loop
//! `menu::save_setup_loop` while `typing` is set. Refuses, before the key is taken: a slot past
//! 99, a length outside 0 .. 0x1F, no car object, no keyboard layout.

use crate::errors::{BadInput, Refused};
use crate::input::key_to_char::{has_layout, key_to_char};
use crate::input::take_key::take_key;
use crate::menu::add_text_length::add_text_length;
use crate::menu::clear_text::clear_text;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::restore_text::restore_text;
use crate::menu::text_entry::set_length;
use crate::service::backspace_setup_name::{backspace_setup_name, entry};
use crate::service::load_setup_slot::highlighted_slot;
use crate::service::owner::Service;
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
fn put_cursor(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    screens: &Screens,
    name: crate::place::BytePlace,
) -> Result<(), BadInput> {
    let mut tail = Vec::new();
    push_string(so, &mut tail, CURSOR_STRING)?;
    tail.push(0);
    let len = screens.loop_flags.text_length.get() as i32;
    name.add(len as u32).write(&tail, "the setup name")?;
    Ok(())
}

/// `c` at the name's character `at`.
fn put_char(name: crate::place::BytePlace, at: i32, c: u8) -> Result<(), BadInput> {
    name.add(at as u32).write(&[c], "the setup name")
}

/// The name cut at its length.
fn end_name(screens: &Screens, name: crate::place::BytePlace) -> Result<(), BadInput> {
    put_char(name, screens.loop_flags.text_length.get() as i32, 0)
}

pub fn edit_setup_name(
    input: &crate::input::owner::Input,
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    service: &'static Service,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    // Everything that could be refused, before the key is taken.
    let (name, _) = entry(menus, service)?;
    let slot = highlighted_slot(menus);
    if current_player(menus).is_none() {
        return Err(BadInput("no current player object").into());
    }
    if !has_layout() {
        return Err(BadInput("no keyboard layout").into());
    }

    let key = take_key(input);
    // Converted: `rc2k_core::menu::text_entry` (`key`, `appends`, `too_wide`).
    use rc2k_core::menu::text_entry::{self as entry, Key};
    match entry::key(key) {
        Key::None => {}
        Key::Escape => {
            restore_text(menus, name)?;
            screens.option_work.typing.set(false);
            return Ok(());
        }
        Key::Enter => {
            if (screens.loop_flags.text_length.get() as i32) == 0 {
                restore_text(menus, name)?;
                add_text_length(menus, name)?;
            }
            screens.option_work.typing.set(false);
            end_name(screens, name)?;
            write_setup_slot(menus, service, slot)?;
            end_name(screens, name)?;
            screens.option_work.typing.set(false);
            return Ok(());
        }
        Key::Clear => clear_text(menus, name)?,
        Key::Backspace => backspace_setup_name(menus, service)?,
        Key::Other(key) => {
            let c = key_to_char(input, key as u32)? as u8;
            let len = screens.loop_flags.text_length.get() as i32;
            if entry::appends(c, len, screens.loop_flags.text_max.get() as i32) {
                put_char(name, len, c)?;
                set_length(menus, len.wrapping_add(1));
                // x is the length before this character.
                let measured =
                    measure_text_at(MEASURING_FONT, &name.bytes("a setup's name")?, len, canvas)?;
                if entry::too_wide(measured.width as i32) {
                    backspace_setup_name(menus, service)?;
                }
            }
        }
    }
    put_cursor(so, screens, name)?;
    Ok(())
}
