//! `menu::apply_screen_mode`: the graphics page's "apply". The chosen screen mode is set
//! (`gfx::screen_mode_choices`, the platform's). Taken: the fonts are freed
//! (`gfx::free_fonts`, `gfx::free_glyph_tables`) and built again (`gfx::build_fonts`,
//! `gfx::build_menu_sprites`), the 3D is tidied (`render3d_tidy`, `objects_tidy`,
//! `release_textures`), `scene::set_up_front_end_scene` runs, tune 4 plays, the highlighted
//! item (`loop_flags.item`) and the screen mode step become 1, and the menu's sound 2 plays.
//! Refused: the choices list is made again from the race's screen mode (the mode before the
//! change), the 3D is tidied, the front end's scene is set up, tune 4 plays, the item and the
//! screen mode step become 3, and the menu's sound 8 plays.
//!
//! An item action of the graphics page.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::owners::Owners;

pub fn apply_screen_mode(o: Owners<'static>) -> Result<(), Refused> {
    let (screens, system, gfx) = (&o.menus.screens, o.system, o.gfx);
    use rc2k_core::menu::steps::{BACK_TO_MODE, TRY_CHOSEN_MODE, applied_mode};
    // `gfx::screen_mode_choices` answers true when the chosen mode was not taken.
    let taken = !crate::gfx::screen_mode_choices::screen_mode_choices(
        gfx,
        &o.gfx.d3d,
        o.system,
        TRY_CHOSEN_MODE != 0,
    )?
    .carry;
    if taken {
        crate::gfx::free_fonts::free_fonts(&gfx.fonts)?;
        crate::gfx::free_glyph_tables::free_glyph_tables(gfx)?;
        crate::gfx::build_fonts::build_fonts_in(gfx, system, o.strings)?;
        crate::gfx::build_menu_sprites::build_menu_sprites_in_game(
            gfx, system, o.menus, o.strings,
        )?;
    } else {
        crate::gfx::screen_mode_choices::screen_mode_choices(
            gfx,
            &o.gfx.d3d,
            o.system,
            BACK_TO_MODE != 0,
        )?;
    }
    crate::race::stage_free::render3d_tidy(o.race, &o.scene.scratch)?;
    crate::race::stage_free::objects_tidy(o.race, &o.scene.scratch)?;
    crate::race::race_end::release_textures(o.scene, o.gfx);
    crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(o)?;
    crate::sound::play_tune_4::play_tune_4(o);
    let applied = applied_mode(taken);
    screens.loop_flags.item.set(applied.item);
    screens.items.current[slot::VISUAL_OPTIONS].set(applied.item);
    crate::sound::play_menu_sound::play_menu_sound_in_game(
        o.sound,
        o.audio,
        o.menus,
        &o.system.session.picture_clock,
        applied.sound,
    );
    Ok(())
}
