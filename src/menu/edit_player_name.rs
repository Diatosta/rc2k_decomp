//! `menu::edit_player_name`: one step of typing a player's name on the players' details pages.
//!
//! The name is the current player's (`Player::name`, `menu/player_name.rs`). One pending key is
//! taken: none: nothing; 1 (Escape): the name kept at the start comes back, the entry ends;
//! 0x1C (Enter): an empty name is replaced by the one kept, the entry ends; 0x6E: the name is
//! cleared (`menu::clear_text`); 0x0E: the last character goes; any other `key_to_char`: a
//! character is appended unless the length is already above the maximum (`loop_flags.text_max`,
//! signed), and taken off again if the name then measures 0xEC pixels or more in the font
//! (measured from the x that is the length before the character). No cursor is written: the
//! page draws it (`screens::draw_name_cursor`). Called from the four `player*_details_loop`s
//! while `entry.typing` is set. Refuses, before the key is taken: no current player, a
//! player without a name, a length outside 0 .. 0x1F, no keyboard layout.

use crate::errors::{BadInput, Refused};
use crate::input::key_to_char::{has_layout, key_to_char};
use crate::input::take_key::take_key;
use crate::menu::clear_text::clear_text;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::player_name::{backspace_name, bring_back_name, name_place};
use crate::menu::text_entry::{TEXT_LEN, set_length};
use crate::text::draw_text::Canvas;
use crate::text::measure_text::measure_text_at;

const MEASURING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::MEASURE;

pub fn edit_player_name(
    input: &crate::input::owner::Input,
    menus: &'static Menus,
    screens: &Screens,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // Everything that could be refused, before the key is taken.
    let at = name_place(menus)?;
    let name = at.place(menus);
    let len = { screens.loop_flags.text_length.get() } as i32;
    if !(0..TEXT_LEN as i32).contains(&len) {
        return Err(BadInput("an entry length outside 0 .. 0x1F").into());
    }
    if !has_layout() {
        return Err(BadInput("no keyboard layout").into());
    }

    let key = take_key(input);
    use rc2k_core::menu::text_entry::{self as entry, Key};
    match entry::key(key) {
        Key::None => {}
        Key::Escape => {
            bring_back_name(menus, name)?;
            screens.entry.typing.set(false);
        }
        Key::Enter => {
            if len == 0 {
                bring_back_name(menus, name)?;
            }
            screens.entry.typing.set(false);
        }
        Key::Clear => clear_text(menus, name)?,
        Key::Backspace => backspace_name(menus, name)?,
        Key::Other(key) => {
            let c = key_to_char(input, key as u32)? as u8;
            if entry::appends(c, len, screens.loop_flags.text_max.get() as i32) {
                name.add(len as u32).write(&[c], "a player's name")?;
                set_length(menus, len.wrapping_add(1));
                let measured = measure_text_at(MEASURING_FONT, &at.tail(menus), len, canvas)?;
                if entry::too_wide(measured.width as i32) {
                    backspace_name(menus, name)?;
                }
            }
        }
    }
    Ok(())
}
