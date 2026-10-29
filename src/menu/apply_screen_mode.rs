//! `menu::apply_screen_mode` (0x4703C2): the graphics page's "apply": the screen mode chosen
//! set (`sub_4045D6` with eax 1, the platform's: DirectDraw modes). Taken: the fonts freed
//! (`gfx::free_fonts`, `gfx::free_glyph_tables`) and built again (`gfx::build_fonts`,
//! `gfx::build_menu_sprites`), the 3D tidied (`render3d_tidy`, `objects_tidy`,
//! `nullsub_6`, `release_textures`), `scene::set_up_front_end_scene`, tune 4, the
//! highlighted item (`dword_755525`) and `dword_74771C` 1, the menu's sound 2. Refused (a
//! set carry): `sub_4045D6` with eax 0 (the mode before, a reading), the 3D tidied, the
//! front end's scene, tune 4, the item and `dword_74771C` 3, the menu's sound 8.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! An item action of the graphics page (0x7548D4). The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};

/// The Rust version. (The original keeps every register of its caller.)
pub fn apply_screen_mode() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::steps::applied_mode`.
    use rc2k_core::menu::steps::{BACK_TO_MODE, TRY_CHOSEN_MODE, applied_mode};
    // `gfx::screen_mode_choices` (0x4045D6): the carry says the chosen mode was not taken.
    let taken = !crate::gfx::screen_mode_choices::screen_mode_choices(TRY_CHOSEN_MODE != 0)?.carry;
    if taken {
        crate::gfx::free_fonts::free_fonts_in_game()?;
        crate::gfx::free_glyph_tables::free_glyph_tables()?;
        crate::gfx::build_fonts::build_fonts()?;
        crate::gfx::build_menu_sprites::build_menu_sprites_in_game()?;
    } else {
        crate::gfx::screen_mode_choices::screen_mode_choices(BACK_TO_MODE != 0)?;
    }
    crate::race::stage_free::render3d_tidy()?;
    crate::race::stage_free::objects_tidy()?;
    // `nullsub_6`.
    crate::race::race_end::release_textures();
    crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
    crate::sound::play_tune_4::play_tune_4();
    let applied = applied_mode(taken);
    LOOP_FLAGS.item.set(applied.item);
    SCREEN_ITEMS.current[slot::VISUAL_OPTIONS].set(applied.item);
    crate::sound::play_menu_sound::play_menu_sound_in_game(applied.sound);
    Ok(())
}
