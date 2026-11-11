//! The arcade start menu's five level items (`arcade_level_1` .. `arcade_level_5` in the
//! original, 0x487CC9, 0x487D10, 0x487D57, 0x487D9E, 0x487DE5): a level not yet open
//! (cheat bit 0 off and `arcade.levels_won` not 1) gets the menu's sound 8; an open
//! one sets `arcade.level` to the level, `arcade.stage` to 0 and runs
//! `menu::take_player1_units` and `menu::arcade_event`.

use crate::errors::Refused;
use crate::owners::Owners;

/// Arcade level `level` (1 .. 5).
pub fn arcade_level(o: Owners<'_>, level: u32) -> Result<(), Refused> {
    let (menus, play) = (o.menus, &o.menus.play);
    // The logic is in `rc2k_core::menu::arcade::level_open`.
    use rc2k_core::menu::arcade::{CLOSED_SOUND, level_open};
    let open = level_open(play.cheats.cheats.get(), || {
        play.arcade.levels_won.get(level as usize - 1)
    });
    if !open {
        crate::sound::play_menu_sound::play_menu_sound_in_game(CLOSED_SOUND);
        return Ok(());
    }
    play.arcade.level.set(level);
    play.arcade.stage.set(0u32);
    crate::menu::take_player1_units::take_player1_units(menus)?;
    crate::menu::arcade_event::arcade_event(o)
}
