//! `menu::stage_range_caption`: "first-last" into the caption text: the first
//! (`champ_view.rows.first_row`) and it plus 9 (a reading: the stages shown on a page).
//!
//! The numbers go through `strings::append_u32`; the minus and the NUL are written in place.
use crate::menu::champ_store::ChampView;

pub fn stage_range_caption(
    champ: &ChampView,
    text_buffers: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), crate::errors::BadInput> {
    let first: u32 = champ.rows.first_row.get();
    let mut text = rc2k_core::menu::event::stage_range(first);
    text.push(0);
    text_buffers.caption.write_at(0, &text, "the caption")
}
