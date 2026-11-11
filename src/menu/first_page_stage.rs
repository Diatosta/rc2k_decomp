//! `menu::first_page_stage` (0x4A2440): the first stage on the last page (`champ_view.first_row`)
//! for `champ_view.count` stages, ten a page: 1 up to ten, else (n / 10) * 10 + 1, less ten when
//! n is a multiple of ten; n 0 there: the fatal error 0x80701010 (cannot happen past the
//! first test).
//!
//! Callers: the event pages' code.

/// The first stage of the last page, stored in `ChampView.first_row` (1 for a single page). (What
/// the original also leaves in eax, edx and ebp, nothing reads.)
use crate::menu::owner::Play;
pub fn first_page_stage(play: &Play) {
    let n: u32 = play.champ_view.count.get();
    // The logic is in `rc2k_core::menu::event::last_page` (its division cannot meet 0).
    let first = rc2k_core::menu::event::last_page(n).map_or(1, |last| last.first);
    play.champ_view.first_row.set(first);
}
