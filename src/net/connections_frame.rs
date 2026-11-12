//! `net::connections_frame` (0x4175C7): the frame of `net::choose_connection`: on the locked
//! surface the background, the title (string 0x14E, `typefaces::STAGE_TITLE` at 0x316, 0x6F), and from
//! y 0x9B sixteen of the connections' names from `g_listChoiceBefore` (the list's top) in
//! `font`, the highlighted one in `typefaces::BUTTON_LIT` in the blink's lit half
//! (`net::connection_entry`, `text::draw_text` at 0x190, each the font's line height lower);
//! unlocked and shown.
//!
//! Called by `screens::run_frame` for `net::choose_connection` (`routes.rs`).
//! Called by `screens::run_frame` for `net::choose_connection`.

use crate::errors::Refused;
use crate::install::owner::Install;
use crate::net::connecting::STATUS_FONT;
use crate::net::lobby::{FONT, copy_picture, draw_string, lock_surface};
use crate::net::owner::Net;
use rc2k_core::net::screens;

const LIT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BUTTON_LIT;

/// A list screen's frame: the title, then up to sixteen names from the list's top, each the
/// name (`name`) of an item below the count, the highlighted one blinking. Shared with
/// `net::sessions_frame`.
pub fn list_frame(
    install: &Install,
    title: u32,
    count: &crate::session::Dword,
    name: impl Fn(u32) -> Result<[u8; 0x40], Refused>,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    if !lock_surface(gfx) {
        return Ok(());
    }
    copy_picture()?;
    draw_string(title, STATUS_FONT, 0x316, 0x6F, gfx)?;
    let mut y: u32 = 0x6F + 0x2C;
    // Converted: `rc2k_core::net::screens` (`LIST_ROWS`, `listed`, `blink_lit`).
    let pages = &install.pages;
    for row in 0..screens::LIST_ROWS {
        let Some(item) = screens::listed(row, pages.choice_before.get(), count.get()) else {
            continue;
        };
        let font = if item == pages.choice.get()
            && screens::blink_lit(pages.frame_clock.get(), pages.move_clock.get())
        {
            LIT
        } else {
            FONT
        };
        let mut text = crate::net::tables::name_text(&name(item)?)?;
        text.push(0);
        let at = y;
        y = y.wrapping_add(font.object().line_height.get());
        crate::text::draw_text::draw_text_at(
            font,
            &text,
            0x190,
            at as i32,
            &mut crate::text::draw_text::GameCanvas::new(gfx),
        )?;
    }
    crate::platform::unlock_surface(gfx);
    crate::platform::show_frame_now();
    Ok(())
}
pub fn connections_frame(
    net: &Net,
    install: &Install,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    list_frame(
        install,
        screens::CONNECTIONS_TITLE,
        &net.vars.connections,
        |n| crate::net::tables::with_connection(&net.vars, n, |c| c.name),
        gfx,
    )
}
