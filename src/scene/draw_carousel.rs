//! `scene::draw_carousel` (0x490649): the entries of the main menu's carousel that
//! can be seen.
//!
//! With one entry (`g_carouselCount` = 1): entry 0, the place whatever ecx holds. With none or
//! fewer: nothing. Else the entry before the front one (`g_carouselFront`, going round) at
//! place -1, the front one at 0, the next at 1, and while the carousel turns (`g_carouselAngle`
//! not 0) the one after at 2; each through `scene::draw_carousel_entry`. One caller,
//! `scene::draw_main_menu_carousel`.

use crate::errors::Refused;
use crate::menu::owner::{Menus, Screens};
use crate::scene::draw_carousel_entry::draw_carousel_entry;
use crate::scene::draw_menu_object::draw_menu_object_in_game;

// ---- game data ----

// ---- Rust version ----

/// `entry`: what stands for `scene::draw_carousel_entry`, given the entry and its row
/// place. `lone_row`: ecx at the original's entry, the row place a lone entry is drawn at
/// (the original sets ebx and edx for it, not ecx).
// Converted: `rc2k_core::scene::carousel::drawn`
pub fn draw_carousel(
    screens: &Screens,
    lone_row: u32,
    entry: &mut dyn FnMut(u32, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    use rc2k_core::scene::carousel::{Drawn, drawn};
    {
        let count: u32 = screens.state.carousel.count.get();
        match drawn(
            count,
            || screens.state.carousel.front.get(),
            || screens.state.carousel.angle.get() != 0,
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
}

/// With the Rust versions of what it calls.
pub fn draw_carousel_in_game(
    menus: &Menus,
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    lone_row: u32,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    draw_carousel(&menus.screens, lone_row, &mut |e, place| {
        draw_carousel_entry(
            &race.shared,
            ctx.view,
            menus,
            e,
            place,
            &mut || draw_menu_object_in_game(ctx),
            scene,
        )
    })
}
