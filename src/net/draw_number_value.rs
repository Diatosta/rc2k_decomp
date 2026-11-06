//! `net::draw_number_value` (0x416B8E): a number setting's value: for the item being edited
//! the number's text as typed (`byte_95E8F8`); else the number written into the game's text
//! buffer (`strings::format_u32`, or string 0x16E for 0); then on into
//! `net::draw_setting_value`.
//!
//! In:  eax = the number, esi = the item, ebp = y
//! Out: as `net::draw_setting_value` leaves them; no `pusha`
//! Caller: `net::settings_frame` (the most players).

use crate::errors::Refused;
use crate::net::draw_setting_value::draw_setting_value;
use crate::net::lobby::Cursor;
use crate::net::store::NetVars;
use crate::strings::owner::strings;
use rc2k_core::net::screens::{self, Number};

/// The Rust version: the number `number` for item `item` at `y`, as the text being typed,
/// "nothing" or its digits; returns the next y (`draw_setting_value`'s).
pub fn draw_number_value(vars: &NetVars, number: u32, item: u32, y: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::screens` (`number_value`, `Number`, `NOTHING`).
    let shown = screens::number_value(item == vars.edited.get(), number);
    let text = if shown == Number::Typed {
        crate::place::Place::from(&crate::install::owner::install().paths.path_a)
    } else {
        let mut cursor = Cursor::text();
        if shown == Number::Nothing {
            cursor.string_id(screens::NOTHING);
        } else {
            cursor.bytes(&crate::strings::format_u32::format_u32(number, 0))?;
        }
        cursor.nul()?;
        crate::place::Place::from(&strings().text.buffer.text)
    };
    draw_setting_value(vars, text, item, y)
}
