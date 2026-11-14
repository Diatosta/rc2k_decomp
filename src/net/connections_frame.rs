//! The frame of `net::choose_connection`: background, title and sixteen connection names from
//! the list's top, the highlighted one in the lit font during the blink's lit half.

use crate::errors::Refused;
use crate::net::connecting::STATUS_FONT;
use crate::net::lobby::{FONT, copy_picture, draw_string, lock_surface};
use rc2k_core::net::screens;

const LIT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BUTTON_LIT;

/// A list screen's frame: the title, then up to sixteen names from the list's top, each the
/// name (`name`) of an item below the count, the highlighted one blinking. Shared with
/// `net::sessions_frame`.
pub fn list_frame(
    o: crate::owners::Owners<'static>,
    title: u32,
    count: &crate::session::Dword,
    name: impl Fn(u32) -> Result<[u8; 0x40], Refused>,
) -> Result<(), Refused> {
    let (gfx, system) = (o.gfx, o.system);
    let list_state = &o.menus.list;
    if !lock_surface(gfx, system) {
        return Ok(());
    }
    copy_picture(gfx, system)?;
    draw_string(
        crate::strings::append_string_by_id::StringOwners::of(o),
        title,
        STATUS_FONT,
        0x316,
        0x6F,
        gfx,
    )?;
    let mut y: u32 = 0x6F + 0x2C;
    // The logic is in `rc2k_core::net::screens` (`LIST_ROWS`, `listed`, `blink_lit`).
    for row in 0..screens::LIST_ROWS {
        let Some(item) = screens::listed(row, list_state.choice_before.get(), count.get()) else {
            continue;
        };
        let font = if item == list_state.choice.get()
            && screens::blink_lit(list_state.frame_clock.get(), list_state.move_clock.get())
        {
            LIT
        } else {
            FONT
        };
        let mut text = crate::net::tables::name_text(&name(item)?)?;
        text.push(0);
        let at = y;
        y = y.wrapping_add(font.object(&gfx.fonts.typefaces).line_height.get());
        crate::text::draw_text::draw_text_at(
            font,
            &text,
            0x190,
            at as i32,
            &mut crate::text::draw_text::GameCanvas::new(gfx),
        )?;
    }
    crate::platform::unlock_surface(gfx);
    crate::platform::show_frame_now(o);
    Ok(())
}
pub fn connections_frame(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let net = o.net;
    list_frame(o, screens::CONNECTIONS_TITLE, net.link.connections(), |n| {
        crate::net::tables::with_connection(&net.link, n, |c| c.name)
    })
}
