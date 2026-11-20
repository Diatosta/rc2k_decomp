//! The arcade start menu's five level items: a level not yet open (the unlock cheat off and its
//! opened byte not 1) plays the closed sound; an open one is started at its first stage and the
//! arcade event runs.

use crate::errors::Refused;
use crate::owners::Owners;

/// Arcade level `level` (1 .. 5).
pub fn arcade_level(o: Owners<'static>, level: u32) -> Result<(), Refused> {
    let (menus, play) = (o.menus, &o.menus.play);
    use rc2k_core::menu::arcade::{CLOSED_SOUND, level_open};
    let open = level_open(play.cheats.read().cheats, || {
        play.arcade.opened(level as usize - 1)
    });
    if !open {
        crate::sound::play_menu_sound::play_menu_sound(
            o.sound,
            o.audio,
            o.menus,
            &o.system.session.picture_clock,
            CLOSED_SOUND,
        );
        return Ok(());
    }
    play.arcade.start_level(level);
    crate::menu::take_player1_units::take_player1_units(menus, o.strings)?;
    crate::menu::arcade_event::arcade_event(o)
}
