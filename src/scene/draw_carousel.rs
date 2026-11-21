//! `scene::draw_carousel`: the entries of the main menu's carousel that can be seen.
//!
//! With one entry (the carousel's count = 1): entry 0, at the row place the caller gives
//! (`lone_row`). With none or fewer: nothing. Else the entry before the front one (the carousel's
//! front, going round) at place -1, the front one at 0, the next at 1, and while the carousel turns
//! (the carousel's angle is not 0) the one after at 2; each through `scene::draw_carousel_entry`.

use crate::errors::Refused;
use crate::menu::carousel_store::Carousel;
use crate::menu::owner::Menus;
use crate::scene::draw_carousel_entry::draw_carousel_entry;
use crate::scene::draw_menu_object::draw_menu_object_in_game;

// ---- game data ----

/// The visible entries drawn; `lone_row` is the row place a lone entry is drawn at.
// Logic: `rc2k_core::scene::carousel::drawn`
pub fn draw_carousel(
    menus: &Menus,
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    lone_row: u32,
) -> Result<(), Refused> {
    use rc2k_core::scene::carousel::{Drawn, drawn};
    let race = ctx.race;
    let carousel: &Carousel = &menus.screens.carousel;
    let entry = |e: u32, place: u32| {
        draw_carousel_entry(
            &race.shared,
            ctx.view,
            menus,
            ctx.scene,
            e,
            place,
            &mut || draw_menu_object_in_game(ctx),
        )
    };
    let count: u32 = carousel.read().count;
    match drawn(
        count,
        || carousel.read().front,
        || carousel.read().angle != 0,
    ) {
        Drawn::Nothing => Ok(()),
        Drawn::Lone => entry(0, lone_row),
        Drawn::Row(entries) => {
            for (e, place) in entries {
                entry(e, place)?;
            }
            Ok(())
        }
    }
}
