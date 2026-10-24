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
//! 2^32 times (refused). Called by `screens::run_frame` for `net::settings_loop`. Its calls
//! through `part`. Shadow mode runs the original alone (it draws). Self-test: a copy whose
//! calls go to stand-ins, against the Rust version: the lock failing, widths of 1, 0x20 and
//! 0x45.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::net::draw_centred_item::draw_centred_item;
use crate::net::draw_number_value::draw_number_value;
use crate::net::draw_setting_value::draw_setting_value;
use crate::net::lobby::{
    HOST_ADDRESS, MACHINE_NAME, MOST_PLAYERS, PALETTE, SAVED_NAME, a, blit, copy_picture, g,
    lock_surface,
};
use crate::net::setting_label::setting_label;
use crate::text::draw_text::GameCanvas;
use crate::text::font::Font;
use rc2k_core::net::screens;

/// The Rust version (the original keeps every register). Called through
/// `screens::run_frame` by `net::settings_loop`.
pub fn settings_frame() -> Result<(), Refused> {
    if !lock_surface() {
        return Ok(());
    }
    copy_picture()?;
    let mut y = 0x92;
    for (item, (label, value, number)) in [
        (0x163, a(SAVED_NAME), false),
        (0x164, a(MACHINE_NAME), false),
        (0x165, g(MOST_PLAYERS), true),
        (0x166, a(HOST_ADDRESS), false),
    ]
    .into_iter()
    .enumerate()
    {
        let item = item as u32;
        y = setting_label(label, item, y)?;
        y = if number {
            draw_number_value(value, item, y)?
        } else {
            draw_setting_value(value, item, y)?
        };
    }
    // The last item's string measured (the x the original hands it is `draw_text`'s x of the
    // value above, 0x190).
    let measured = unsafe {
        crate::text::measure_string_by_id::measure_string_by_id_at(
            0x16F,
            a(PALETTE) as *mut Font,
            0x190,
            &mut GameCanvas,
        )
    }?;
    let width = measured.width;
    // Converted: `rc2k_core::net::screens::last_item` (and the frame's sprites).
    let (tiles, text_x) = screens::last_item(width)?;
    // Each blit gets the palette the one before leaves (the first, the measuring's lines).
    let mut palette = measured.lines;
    let mut at = screens::LAST_ITEM_RIGHT;
    let mut piece = |glyph: u32, step: u32| -> Result<(), Refused> {
        at = at.wrapping_sub(step);
        palette = blit(glyph, at, 0x21D, palette)?;
        Ok(())
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
    draw_centred_item(0x16F, 4, text_x, 0x21D)?;
    crate::platform::unlock_surface();
    crate::platform::show_frame_now();
    Ok(())
}
