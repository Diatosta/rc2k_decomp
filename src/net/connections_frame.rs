//! `net::connections_frame` (0x4175C7): the frame of `net::choose_connection`: on the locked
//! surface the background, the title (string 0x14E, `stru_51E9A4` at 0x316, 0x6F), and from
//! y 0x9B sixteen of the connections' names from `g_listChoiceBefore` (the list's top) in
//! `font`, the highlighted one in `unk_51E644` in the blink's lit half
//! (`net::connection_entry`, `text::draw_text` at 0x190, each the font's line height lower);
//! unlocked and shown.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; its caller, `screens::run_frame`,
//!      passes the frame's carry on, which no list screen reads)
//! Called by `screens::run_frame` for `net::choose_connection` (`routes.rs`).
//! Called by `screens::run_frame` for `net::choose_connection`.

use crate::errors::Refused;
use crate::install::vars::INSTALLER_PAGES;
use crate::net::connecting::STATUS_FONT;
use crate::net::lobby::{FONT, copy_picture, draw_string, lock_surface};
use crate::net::store::NET;
use rc2k_core::net::screens;

const LIT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BUTTON_LIT;

/// A list screen's frame: the title, then up to sixteen names from the list's top, each the
/// name (`name`) of an item below the count, the highlighted one blinking. Shared with
/// `net::sessions_frame`.
pub fn list_frame(
    title: u32,
    count: &crate::session::Dword,
    name: fn(u32) -> Result<[u8; 0x40], Refused>,
) -> Result<(), Refused> {
    if !lock_surface() {
        return Ok(());
    }
    copy_picture()?;
    draw_string(title, STATUS_FONT, 0x316, 0x6F)?;
    let mut y: u32 = 0x6F + 0x2C;
    // Converted: `rc2k_core::net::screens` (`LIST_ROWS`, `listed`, `blink_lit`).
    for row in 0..screens::LIST_ROWS {
        let Some(item) = screens::listed(row, INSTALLER_PAGES.choice_before.get(), count.get())
        else {
            continue;
        };
        let font = if item == INSTALLER_PAGES.choice.get()
            && screens::blink_lit(
                INSTALLER_PAGES.frame_clock.get(),
                INSTALLER_PAGES.move_clock.get(),
            ) {
            LIT
        } else {
            FONT
        };
        let mut text = crate::net::tables::name_text(&name(item)?)?;
        text.push(0);
        let at = y;
        y = y.wrapping_add(font.object().line_height.get());
        crate::text::draw_text::draw_bytes_at(
            font.as_ptr(),
            &text,
            0x190,
            at as i32,
            &mut crate::text::draw_text::GameCanvas,
        )?;
    }
    crate::platform::unlock_surface();
    crate::platform::show_frame_now();
    Ok(())
}

/// The Rust version.
pub fn connections_frame() -> Result<(), Refused> {
    list_frame(screens::CONNECTIONS_TITLE, &NET.connections, |n| {
        crate::net::tables::with_connection(n, |c| c.name)
    })
}
