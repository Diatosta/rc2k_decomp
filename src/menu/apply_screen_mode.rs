//! `menu::apply_screen_mode` (0x4703C2): the graphics page's "apply": the screen mode chosen
//! set (`gfx::screen_mode_choices` with eax 1, the platform's: DirectDraw modes). Taken: the fonts
//! freed (`gfx::free_fonts`, `gfx::free_glyph_tables`) and built again (`gfx::build_fonts`,
//! `gfx::build_menu_sprites`), the 3D tidied (`render3d_tidy`, `objects_tidy`,
//! `nullsub_6`, `release_textures`), `scene::set_up_front_end_scene`, tune 4, the
//! highlighted item (`loop_flags.item`) and the screen mode step 1, the menu's sound 2. Refused (a
//! set carry): `gfx::screen_mode_choices` with eax 0 (the list made again from the race's screen mode, i.e. the mode before the change), the 3D tidied,
//! the front end's scene, tune 4, the item and the screen mode step 3, the menu's sound 8.
//!
//! An item action of the graphics page (0x7548D4). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::owners::Owners;

pub fn apply_screen_mode(o: Owners<'static>) -> Result<(), Refused> {
    let (screens, system, gfx) = (&o.menus.screens, o.system, o.gfx);
    // The logic is in `rc2k_core::menu::steps::applied_mode`.
    use rc2k_core::menu::steps::{BACK_TO_MODE, TRY_CHOSEN_MODE, applied_mode};
    // `gfx::screen_mode_choices` (0x4045D6): the carry says the chosen mode was not taken.
    let taken = !crate::gfx::screen_mode_choices::screen_mode_choices(
        gfx,
        o.d3d,
        o.system,
        TRY_CHOSEN_MODE != 0,
    )?
    .carry;
    if taken {
        crate::gfx::free_fonts::free_fonts_in_game(gfx)?;
        crate::gfx::free_glyph_tables::free_glyph_tables(gfx)?;
        crate::gfx::build_fonts::build_fonts_in(gfx, system, o.strings)?;
        crate::gfx::build_menu_sprites::build_menu_sprites_in_game(
            gfx, system, o.menus, o.strings,
        )?;
    } else {
        crate::gfx::screen_mode_choices::screen_mode_choices(
            gfx,
            o.d3d,
            o.system,
            BACK_TO_MODE != 0,
        )?;
    }
    crate::race::stage_free::render3d_tidy(o.race)?;
    crate::race::stage_free::objects_tidy(o.race)?;
    // `nullsub_6`.
    crate::race::race_end::release_textures(o.scene, o.gfx);
    crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(o)?;
    crate::sound::play_tune_4::play_tune_4(o.sound, o.audio, o.system);
    let applied = applied_mode(taken);
    screens.loop_flags.item.set(applied.item);
    screens.items.current[slot::VISUAL_OPTIONS].set(applied.item);
    crate::sound::play_menu_sound::play_menu_sound_in_game(o, applied.sound);
    Ok(())
}
