//! The review menus' three replay items (what each starts is a reading of the viewer's
//! modes 1, 2 and 3). A and B do nothing while the multi-player flag is 1 (a reading: the replay
//! not to be had); see `race/replay_mode.rs`.

use crate::errors::Refused;
use crate::race::replay_viewer::Mode;

/// `menu::replay_item_a`: unless `RaceState.settings.no_replay` is 1, with a replay there
/// (`ReplayFileBlock.replay_closed` 1) `race::replay_mode_1`; else half a second's sleep and the
/// menu's sound 8.
pub fn replay_item_a(
    race: &'static crate::race::owner::Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    if race.shared.state.settings.no_replay.get() != 1
        && race.shared.replay_file().replay_closed.get() == 1
    {
        return crate::race::replay_mode::replay_mode(race, o, Mode::LastRace);
    }
    crate::system::sleep::sleep(500);
    crate::sound::play_menu_sound::play_menu_sound_in_game(o.sound, o.audio, o.menus, o.system, 8);
    Ok(())
}

/// `menu::replay_item_b`: unless `RaceState.settings.no_replay` is 1, `race::replay_mode_3`.
pub fn replay_item_b(
    race: &'static crate::race::owner::Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    if race.shared.state.settings.no_replay.get() != 1 {
        crate::race::replay_mode::replay_mode(race, o, Mode::SaveLast)?;
    }
    Ok(())
}

/// `menu::replay_item_c`: the viewer's mode 2.
pub fn replay_item_c(
    race: &'static crate::race::owner::Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    crate::race::replay_mode_2::replay_mode_2(race, o)
}
