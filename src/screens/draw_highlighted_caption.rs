//! `screens::draw_highlighted_caption` (0x4695C6): the name of the highlighted entry of a
//! menu whose entries are turning objects, on a button at the bottom of the screen.
//!
//! the carousel step state says which menu (0 .. 6; anything else: nothing is drawn) and so which list of
//! string numbers; the highlighted item picks the string, which is put at the caption buffer, where
//! string 0x1004 is read from. The button is as wide as the caption measured with the font at
//! 0x51E824 plus 12, centred on x = 0x197 at y = 0x1FE: sprite 0xF, pieces of 32 of sprite 0xB (the
//! last one drawn over the one before, so that the width comes out), sprite 0x10; their palette is
//! what the measuring leaves in ebx, the caption's number of lines. A width of 0 is fatal error
//! 0x80701010. Then the caption at (0x195, 0x1FB) in the font blended between those at 0x51E964 and
//! 0x51E844 by the time since the last key. Leaves the button's work variables (0x7559AC ..
//! 0x7559C4), its two ends in 0x755A20 .. 0x755A2C, and `g_markerX` / `g_markerY`. In 12 lists (the
//! main menu's among them: its menu is number 0). Refuses an item of 0 or above 32, a caption of
//! 0x100 bytes or more, and a caption narrower than 20 (the original then draws 2^32 middle
//! pieces). Screen: not confirmed; the name is a reading of the code. The menus and draw lists run
//! its `entry` (`menu::table_calls`).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::text::draw_string_by_id::string_text;
use crate::text::draw_text::Canvas;

// ---- game data ----
/// Where each menu's list of string numbers starts in `CAPTIONS.caption_lists`. The item is
/// not checked against a list's length, as in the original (the last list's 32 items still
/// end inside the table).
const LISTS: [usize; 7] = [0, 6, 29, 36, 40, 44, 52];
const CAPTION_LEN: usize = 0x100;
const ITEMS: u32 = 32;

pub fn draw_highlighted_caption(
    menus: &Menus,
    measured_at: u32,
    canvas: &mut dyn Canvas,
) -> Result<Option<bool>, Refused> {
    let menu: u32 = menus.screens.state.carousel.menu.get();
    let Some(&list) = LISTS.get(menu as usize) else {
        return Ok(None);
    };
    let item: u32 = menus.screens.loop_flags.item.get();
    if !(1..=ITEMS).contains(&item) {
        return Err(BadInput("a highlighted item outside the lists of captions").into());
    }
    let id: u32 = menus.tables.captions.caption_lists[list + item as usize - 1].get();
    let caption = string_text(id)?;
    if caption.len() > CAPTION_LEN {
        return Err(BadInput("a caption longer than its place").into());
    }
    let o = crate::owners::boundary();
    o.race
        .text_buffers()
        .caption
        .write_at(0, &caption, "the caption")?;
    // The logic is in `rc2k_core::screens::highlighted_caption`.
    let since_key = o
        .system
        .session
        .clocks
        .frame
        .get()
        .wrapping_sub(menus.play.champ_view.last_key_time.get());
    let too_wide = rc2k_core::screens::highlighted_caption::highlighted_caption(
        &mut crate::screens::game_screen::GameScreen::<
            rc2k_core::screens::highlighted_caption::Font,
            rc2k_core::screens::highlighted_caption::Work,
        >::new(canvas),
        measured_at as i32,
        since_key,
    )?;
    Ok(Some(too_wide))
}
