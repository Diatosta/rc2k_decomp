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
use crate::net::connecting::STATUS_FONT;
use crate::net::lobby::{
    CHOICE, CHOICE_BEFORE, FONT, FRAME_CLOCK, MOVE_CLOCK, a, connection, copy_picture, draw_string,
    draw_text, g, lock_surface,
};
use crate::net::store::NET;
use rc2k_core::net::screens;

const LIT: usize = 0x51E644; // unk_51E644

/// A list screen's frame: the title, then up to sixteen names from the list's top, each the
/// record (`record`) of an item below the count, the highlighted one blinking. Shared with
/// `net::sessions_frame`.
pub fn list_frame(
    title: u32,
    count: &crate::session::Dword,
    record: fn(u32) -> Result<u32, Refused>,
) -> Result<(), Refused> {
    if !lock_surface() {
        return Ok(());
    }
    copy_picture()?;
    draw_string(title, STATUS_FONT, 0x316, 0x6F)?;
    let mut y: u32 = 0x6F + 0x2C;
    // Converted: `rc2k_core::net::screens` (`LIST_ROWS`, `listed`, `blink_lit`).
    for row in 0..screens::LIST_ROWS {
        let Some(item) = screens::listed(row, g(CHOICE_BEFORE), count.get()) else {
            continue;
        };
        let font = if item == g(CHOICE) && screens::blink_lit(g(FRAME_CLOCK), g(MOVE_CLOCK)) {
            a(LIT)
        } else {
            a(FONT)
        };
        let name = record(item)?;
        let at = y;
        y = y.wrapping_add(unsafe { ((font + 8) as *const u32).read_unaligned() });
        draw_text(font, name, 0x190, at)?;
    }
    crate::platform::unlock_surface();
    crate::platform::show_frame_now();
    Ok(())
}

/// The Rust version.
pub fn connections_frame() -> Result<(), Refused> {
    list_frame(screens::CONNECTIONS_TITLE, &NET.connections, connection)
}
