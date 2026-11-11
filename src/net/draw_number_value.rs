//! `net::draw_number_value` (0x416B8E): a number setting's value: for the item being edited
//! the number's text as typed (`path_a`); else the number written into the game's text
//! buffer (`strings::format_u32`, or string 0x16E for 0); then on into
//! `net::draw_setting_value`.
//!
//! Takes the number, item, y.
//! Caller: `net::settings_frame` (the most players).

use crate::errors::Refused;
use crate::net::draw_setting_value::draw_setting_value;
use crate::net::lobby::{push_id, store_text};
use crate::net::owner::net;
use rc2k_core::net::screens::{self, Number};

/// The number `number` for item `item` at `y`, as the text being typed,
/// "nothing" or its digits (the last two built in the game's text buffer); returns the next
/// y (`draw_setting_value`'s).
pub fn draw_number_value(
    install: &'static crate::install::owner::Install,
    number: u32,
    item: u32,
    y: u32,
) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::screens` (`number_value`, `Number`, `NOTHING`).
    let shown = screens::number_value(item == net().edit().edited.get(), number);
    let text = if shown == Number::Typed {
        crate::place::BytePlace::from(&install.paths.path_a).text("a setting's text")?
    } else {
        let mut built = Vec::new();
        if shown == Number::Nothing {
            push_id(&mut built, screens::NOTHING)?;
        } else {
            built.extend_from_slice(&crate::strings::format_u32::format_u32(number, 0));
        }
        let mut stored = store_text(built)?;
        stored.pop();
        stored
    };
    draw_setting_value(&text, item, y)
}
