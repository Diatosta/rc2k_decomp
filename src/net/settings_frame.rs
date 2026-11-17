//! The frame of the network settings screen: background, six settings (connection, player's
//! name, most players, host's address, port, password) and the last item's framed string. A zero-width string is
//! refused.

use crate::errors::Refused;
use crate::net::draw_centred_item::draw_centred_item;
use crate::net::draw_number_value::draw_number_value;
use crate::net::draw_setting_value::draw_setting_value;
use crate::net::lobby::{PALETTE, blit, copy_picture, lock_surface};
use crate::net::setting_label::setting_label;
use crate::text::draw_text::GameCanvas;
use rc2k_core::net::screens;

/// Called through
/// `screens::run_frame` by `net::settings_loop`.
pub fn settings_frame(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let net = o.net;
    if !lock_surface(o.gfx, o.system) {
        return Ok(());
    }
    copy_picture(o.gfx, o.system)?;
    let mut y = 0x92;
    /// A setting's value: a text kept in the network's state, or a number.
    enum Value {
        Text(crate::memory::place::BytePlace),
        Number(u32),
        /// A text built for the screen (the password: its stars, or itself while typed).
        Shown(Vec<u8>),
    }
    let password = if net.settings().edit.edited.get() == rc2k_core::net::chat::PASSWORD {
        let text = net.settings().most_players_text.read();
        text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())].to_vec()
    } else {
        crate::net::password::masked()
    };
    for (item, (label, value)) in [
        (0x163, Value::Text((&net.settings().saved_name).into())),
        (0x164, Value::Text((&net.setup().machine_name).into())),
        (0x165, Value::Number(net.setup().most_players.get())),
        (0x166, Value::Text((&net.setup().host_address).into())),
        (
            rc2k_core::strings::own::id::NETWORK_PORT,
            Value::Number(rc2k_core::net::chat::shown_port(net.setup().port.get())),
        ),
        (
            rc2k_core::strings::own::id::NETWORK_PASSWORD,
            Value::Shown(password),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let item = item as u32;
        y = setting_label(o, net, label, item, y)?;
        y = match value {
            Value::Number(n) => draw_number_value(o, n, item, y)?,
            Value::Shown(text) => draw_setting_value(o, &text, item, y)?,
            Value::Text(text) => {
                let text = text.text("a setting's text")?;
                draw_setting_value(o, &text, item, y)?
            }
        };
    }
    // The last item's string measured from the x of the value above (0x190).
    let measured = {
        crate::text::measure_string_by_id::measure_string_by_id_at(
            crate::strings::append_string_by_id::StringOwners::of(o),
            0x16F,
            PALETTE,
            0x190,
            &mut GameCanvas::new(o.gfx),
        )
    }?;
    let width = measured.width;
    // The logic is in `rc2k_core::net::screens::last_item` (and the frame's sprites).
    let (tiles, text_x) = screens::last_item(width)?;
    let mut at = screens::LAST_ITEM_RIGHT;
    let mut piece = |glyph: u32, step: u32| -> Result<(), Refused> {
        at = at.wrapping_sub(step);
        blit(glyph, at, 0x21D, o.gfx, o.menus)
    };
    piece(screens::RIGHT_END, screens::END_WIDTH)?;
    // Counted down with `dec` / `jnz`: at least once.
    let mut left = tiles;
    loop {
        piece(screens::TILE, screens::TILE_WIDTH)?;
        left = left.wrapping_sub(1);
        if left == 0 {
            break;
        }
    }
    piece(screens::LEFT_END, screens::END_WIDTH)?;
    draw_centred_item(o, net, 0x16F, 6, text_x, 0x21D)?;
    crate::platform::unlock_surface(o.gfx);
    crate::platform::show_frame_now(o);
    Ok(())
}
