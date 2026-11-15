//! The name of the highlighted entry of a menu whose entries are turning objects, on a
//! button at the bottom of the screen. The carousel step state says which menu (0 to 6; anything
//! else draws nothing) and so which string list. The button is as wide as the caption measured
//! plus 12, centred, built of the end sprites and 32-pixel middle pieces (the last drawn over the
//! one before so the width comes out); the caption is drawn in a font blended by the time since
//! the last key. A width of 0 is fatal error 0x80701010.
//!
//! Refuses an item of 0 or above 32, a caption of 0x100 bytes or more and one narrower than 20.
//! The name is a reading of the code; the screen is not confirmed.

use crate::errors::{BadInput, Refused};
use crate::owners::Owners;
use crate::screens::text_buffers_store::text_buffers;
use crate::text::draw_string_by_id::string_text;
use crate::text::draw_text::Canvas;

// ---- game data ----
/// Where each menu's list of string numbers starts in `CAPTIONS.caption_lists`. The item is not
/// checked against a list's length (the last list's 32 items still end inside the table).
const LISTS: [usize; 7] = [0, 6, 29, 36, 40, 44, 52];
const CAPTION_LEN: usize = 0x100;
const ITEMS: u32 = 32;

pub fn draw_highlighted_caption(
    o: Owners<'static>,
    measured_at: u32,
    canvas: &mut dyn Canvas,
) -> Result<Option<bool>, Refused> {
    let menus = o.menus;
    let menu: u32 = menus.screens.carousel.menu.get();
    let Some(&list) = LISTS.get(menu as usize) else {
        return Ok(None);
    };
    let item: u32 = menus.screens.loop_flags.item.get();
    if !(1..=ITEMS).contains(&item) {
        return Err(BadInput("a highlighted item outside the lists of captions").into());
    }
    let id: u32 = menus.tables.captions.caption_lists[list + item as usize - 1].get();
    let caption = string_text(crate::strings::append_string_by_id::StringOwners::of(o), id)?;
    if caption.len() > CAPTION_LEN {
        return Err(BadInput("a caption longer than its place").into());
    }
    text_buffers()
        .caption
        .write_at(0, &caption, "the caption")?;
    // The logic is in `rc2k_core::screens::highlighted_caption`.
    let since_key = o
        .system
        .clocks()
        .frame
        .get()
        .wrapping_sub(menus.play.champ_view.key_clock.last_key_time.get());
    let too_wide = rc2k_core::screens::highlighted_caption::highlighted_caption(
        &mut crate::screens::game_screen::GameScreen::<
            rc2k_core::screens::highlighted_caption::Font,
            rc2k_core::screens::highlighted_caption::Work,
        >::new(canvas, o),
        measured_at as i32,
        since_key,
    )?;
    Ok(Some(too_wide))
}
