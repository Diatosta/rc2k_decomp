//! `menu::stage_range_caption` (0x4A1FBB): "first-last" into `g_captionText`: the first
//! (`dword_7556B8`) and it plus 9 (a reading: the stages shown on a page).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event pages' code (`sub_4A1F7A`, ..). `strings::append_u32`'s Rust version; the
//! minus and the NUL written in place. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

/// The Rust version (see the header): the registers it leaves.
use crate::menu::owner::Play;
pub fn stage_range_caption(play: &Play) -> Result<(), crate::errors::BadInput> {
    let first: u32 = play.champ_view.first_row.get();
    // Converted: `rc2k_core::menu::event::stage_range`.
    let mut text = rc2k_core::menu::event::stage_range(first);
    text.push(0);
    crate::pointers::GamePtr::to(&crate::screens::text_buffers_store::TEXT_BUFFERS.caption)
        .write_bytes(&text, "the caption")
}
