//! `net::settings_frame` (0x4169E2): the frame of the network settings screen
//! (`net::settings_loop`): on the locked surface, the background, four settings from y 0x92
//! (`net::setting_label` and the value): the connection (string 0x163, `unk_52ACA4`), the
//! player's name (0x164, `g_machineName`), the most players (0x165, `dword_52ABD0`,
//! `net::draw_number_value`) and the host's address (0x166, `unk_52ABE4`); then the last
//! item's frame, right to left from 0x311 at y 0x21D (sprites 0x13, 0x12 once per 32 pixels
//! of string 0x16F's width, 0x11) and its string (`net::draw_centred_item`); unlocked, shown.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! The width's tiles are counted down with `dec` / `jnz`: a string of width 0 would go round
//! 2^32 times (refused). Called by `screens::run_frame` for `net::settings_loop`.

use crate::errors::Refused;
use crate::net::draw_centred_item::draw_centred_item;
use crate::net::draw_number_value::draw_number_value;
use crate::net::draw_setting_value::draw_setting_value;
use crate::net::lobby::{PALETTE, blit, copy_picture, lock_surface};
use crate::net::owner::net;
use crate::net::setting_label::setting_label;
use crate::text::draw_text::GameCanvas;
use rc2k_core::net::screens;

/// The Rust version (the original keeps every register). Called through
/// `screens::run_frame` by `net::settings_loop`.
pub fn settings_frame() -> Result<(), Refused> {
    let net = net();
    let vars = &net.vars;
    if !lock_surface() {
        return Ok(());
    }
    copy_picture()?;
    let mut y = 0x92;
    /// A setting's value: a text kept in the network's state, or a number.
    enum Value {
        Text(&'static crate::session::Bytes<64>),
        Number(u32),
    }
    for (item, (label, value)) in [
        (0x163, Value::Text(&vars.saved_name)),
        (0x164, Value::Text(&vars.machine_name)),
        (0x165, Value::Number(vars.most_players.get())),
        (0x166, Value::Text(&vars.host_address)),
    ]
    .into_iter()
    .enumerate()
    {
        let item = item as u32;
        y = setting_label(net, label, item, y)?;
        y = match value {
            Value::Number(n) => draw_number_value(&net.vars, n, item, y)?,
            Value::Text(text) => draw_setting_value(&net.vars, text, item, y)?,
        };
    }
    // The last item's string measured (the x the original hands it is `draw_text`'s x of the
    // value above, 0x190).
    let measured = {
        crate::text::measure_string_by_id::measure_string_by_id_at(
            0x16F,
            PALETTE.game_ptr(),
            0x190,
            &mut GameCanvas,
        )
    }?;
    let width = measured.width;
    // Converted: `rc2k_core::net::screens::last_item` (and the frame's sprites).
    let (tiles, text_x) = screens::last_item(width)?;
    let mut at = screens::LAST_ITEM_RIGHT;
    let mut piece = |glyph: u32, step: u32| -> Result<(), Refused> {
        at = at.wrapping_sub(step);
        blit(glyph, at, 0x21D)
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
    draw_centred_item(&net.vars, 0x16F, 4, text_x, 0x21D)?;
    crate::platform::unlock_surface();
    crate::platform::show_frame_now();
    Ok(())
}
