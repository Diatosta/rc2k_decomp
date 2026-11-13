//! A number setting's value: the number's text as typed for the item being edited, else the
//! number as text (string 0x16E for 0), then drawn by `net::draw_setting_value`.

use crate::errors::Refused;
use crate::net::draw_setting_value::draw_setting_value;
use crate::net::lobby::{push_id, store_text};
use rc2k_core::net::screens::{self, Number};

/// The number `number` for item `item` at `y`, as the text being typed, "nothing" or its
/// digits (the last two built in the game's text buffer); returns the next y.
pub fn draw_number_value(
    o: crate::owners::Owners<'static>,
    number: u32,
    item: u32,
    y: u32,
) -> Result<u32, Refused> {
    let (net, install) = (o.net, o.install);
    // The logic is in `rc2k_core::net::screens` (`number_value`, `Number`, `NOTHING`).
    let shown = screens::number_value(item == net.edit().edited.get(), number);
    let text = if shown == Number::Typed {
        crate::place::BytePlace::from(&install.paths.path_a).text("a setting's text")?
    } else {
        let mut built = Vec::new();
        if shown == Number::Nothing {
            push_id(o, &mut built, screens::NOTHING)?;
        } else {
            built.extend_from_slice(&crate::strings::format_u32::format_u32(number, 0));
        }
        let mut stored = store_text(o.strings, built)?;
        stored.pop();
        stored
    };
    draw_setting_value(o, &text, item, y)
}
