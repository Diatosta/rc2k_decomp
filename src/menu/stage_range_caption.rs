//! `menu::stage_range_caption` (0x4A1FBB): "first-last" into `g_captionText`: the first
//! (`dword_7556B8`) and it plus 9 (a reading: the stages shown on a page).
//!
//! Callers: the event pages' code (`menu::show_class_stage_results`, ..). `strings::append_u32`'s
//! Rust version; the minus and the NUL written in place. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

/// The Rust version.
use crate::menu::owner::Play;
pub fn stage_range_caption(play: &Play) -> Result<(), crate::errors::BadInput> {
    let first: u32 = play.champ_view.first_row.get();
    // Converted: `rc2k_core::menu::event::stage_range`.
    let mut text = rc2k_core::menu::event::stage_range(first);
    text.push(0);
    crate::screens::text_buffers_store::TEXT_BUFFERS
        .caption
        .write_at(0, &text, "the caption")
}
