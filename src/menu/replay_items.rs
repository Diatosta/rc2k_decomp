//! The review menus' two replay items (`menu::replay_item_a`, `menu::replay_item_b`; what
//! each starts is a reading: the viewer's modes 1 and 3). Both do nothing while
//! `dword_75ACDC` is 1 (a reading: the replay not to be had); see `race/replay_mode.rs`.

use crate::errors::Refused;
use crate::menu::screen_loop::Carried;
use crate::race::state_store::RACE_STATE;

/// `menu::replay_item_a` (0x47053E): unless `dword_75ACDC` is 1, with a replay there
/// (`dword_5BEC84` 1) `race::replay_mode_1`; else half a second's sleep and the menu's sound
/// 8. `c` the carried values the item gets.
pub unsafe fn replay_item_a(c: &Carried) -> Result<(), Refused> {
    if RACE_STATE.settings.no_replay.get() != 1
        && crate::race::replay_file_store::REPLAY_FILE
            .replay_closed
            .get()
            == 1
    {
        return crate::race::replay_mode::replay_mode(c, 1);
    }
    crate::system::sleep::sleep(500);
    crate::sound::play_menu_sound::play_menu_sound_in_game(8);
    Ok(())
}

/// `menu::replay_item_b` (0x47052F): unless `dword_75ACDC` is 1, `race::replay_mode_3`.
pub unsafe fn replay_item_b(c: &Carried) -> Result<(), Refused> {
    if RACE_STATE.settings.no_replay.get() != 1 {
        crate::race::replay_mode::replay_mode(c, 3)?;
    }
    Ok(())
}
