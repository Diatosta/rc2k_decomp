//! `menu::first_page_stage`: the first stage on the last page (`champ_view.rows.first_row`)
//! for `champ_view.rows.count` stages, ten a page: 1 up to ten, else (n / 10) * 10 + 1, less
//! ten when n is a multiple of ten; n 0 there: the fatal error 0x80701010 (cannot happen past the
//! first test).

/// The first stage of the last page, stored in `ChampView.first_row` (1 for a single page).
use crate::menu::champ_store::ChampView;
pub fn first_page_stage(champ: &ChampView) {
    let n: u32 = champ.rows.count.get();
    let first = rc2k_core::menu::event::last_page(n).map_or(1, |last| last.first);
    champ.rows.first_row.set(first);
}
