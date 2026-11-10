//! `menu::apply_screen_mode` (0x4703C2): the graphics page's "apply": the screen mode chosen
//! set (`gfx::screen_mode_choices` with eax 1, the platform's: DirectDraw modes). Taken: the fonts
//! freed (`gfx::free_fonts`, `gfx::free_glyph_tables`) and built again (`gfx::build_fonts`,
//! `gfx::build_menu_sprites`), the 3D tidied (`render3d_tidy`, `objects_tidy`,
//! `nullsub_6`, `release_textures`), `scene::set_up_front_end_scene`, tune 4, the
//! highlighted item (`dword_755525`) and `dword_74771C` 1, the menu's sound 2. Refused (a
//! set carry): `gfx::screen_mode_choices` with eax 0 (the mode before, a reading), the 3D tidied,
//! the front end's scene, tune 4, the item and `dword_74771C` 3, the menu's sound 8.
//!
//! An item action of the graphics page (0x7548D4). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Screens;
use crate::menu::screen_items_store::slot;

pub fn apply_screen_mode(screens: &Screens) -> Result<(), Refused> {
    let system = crate::system::owner::system();
    let gfx = crate::gfx::owner::gfx();
    // Converted: `rc2k_core::menu::steps::applied_mode`.
    use rc2k_core::menu::steps::{BACK_TO_MODE, TRY_CHOSEN_MODE, applied_mode};
    let display = &gfx.display;
    // `gfx::screen_mode_choices` (0x4045D6): the carry says the chosen mode was not taken.
    let taken =
        !crate::gfx::screen_mode_choices::screen_mode_choices(display, TRY_CHOSEN_MODE != 0)?.carry;
    if taken {
        crate::gfx::free_fonts::free_fonts_in_game(gfx)?;
        crate::gfx::free_glyph_tables::free_glyph_tables(gfx)?;
        crate::gfx::build_fonts::build_fonts(gfx)?;
        crate::gfx::build_menu_sprites::build_menu_sprites_in_game(gfx, system)?;
    } else {
        crate::gfx::screen_mode_choices::screen_mode_choices(display, BACK_TO_MODE != 0)?;
    }
    crate::race::stage_free::render3d_tidy(crate::race::owner::race())?;
    crate::race::stage_free::objects_tidy(crate::race::owner::race())?;
    // `nullsub_6`.
    crate::race::race_end::release_textures();
    crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
    crate::sound::play_tune_4::play_tune_4();
    let applied = applied_mode(taken);
    screens.loop_flags.item.set(applied.item);
    screens.items.current[slot::VISUAL_OPTIONS].set(applied.item);
    crate::sound::play_menu_sound::play_menu_sound_in_game(applied.sound);
    Ok(())
}
