//! `net::fill_own_player` (0x4181A3): this machine's player set up for the network game from
//! player 1 (`g_player1`): its name (`strings::copy_printable`, 0x1F room) to `player_name`;
//! its record (`own`): state 0, the car, its colours, the nationality, the stage (0 is made
//! 0x29 with direction 0) and its direction, the race number, a random byte
//! (`math::next_random`), the time and total 0; and the race's copy of the player's
//! settings (the co-driver, the race number, the view, the HUD's switches, the steering),
//! the HUD's bits -1, and the next field after them 0.
//!
//! Takes `name`.
//! Callers: `net::start_link`, `net::open_lobby`.

use crate::errors::Refused;
use crate::menu::owner::Play;
use crate::menu::player::Player;
use crate::net::owner::Net;
use crate::race::owner::Race;
use crate::session::Dword;
use rc2k_core::net::record;

/// A setting copied: (to, from player 1's record).
type Setting = (&'static Dword, fn(&Player) -> u32);

/// The settings copied.
const SETTINGS: [Setting; 13] = [
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].codriver_sex,
        |p| p.codriver_sex,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].race_number,
        |p| p.race_number,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].view,
        |p| p.view,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].map_area,
        |p| p.map_area,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].rear_mirror,
        |p| p.rear_mirror,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].rev_counter,
        |p| p.rev_counter,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].message_display,
        |p| p.message_display,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].car_info,
        |p| p.car_info,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].driver_info,
        |p| p.driver_info,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].progress_bar,
        |p| p.progress_bar,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].pace_notes,
        |p| p.pace_notes,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].metric_units,
        |p| p.metric_units,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.cars[0].steering_sensitivity,
        |p| p.steering_sensitivity,
    ),
];
/// Copied after the HUD's bits: (to, from).
const MORE: [(&Dword, &Dword); 5] = [
    (
        &crate::race::owner::RACE.shared.state.settings.lens_flare,
        &crate::menu::owner::MENUS.screens.option_work.lens_flare,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.reflections,
        &crate::menu::owner::MENUS.screens.option_work.reflections,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.car_shadows,
        &crate::menu::owner::MENUS.screens.option_work.car_shadows,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.car_detail,
        &crate::menu::owner::MENUS.screens.option_work.car_detail,
    ),
    (
        &crate::race::owner::RACE.shared.state.settings.headlights,
        &crate::menu::owner::MENUS.screens.option_work.headlights,
    ),
];
pub fn fill_own_player(
    race: &Race,
    net: &'static Net,
    play: &Play,
    name: &[u8],
) -> Result<(), Refused> {
    // `strings::copy_printable` with 0x1F room: at most 0x1E characters of `name` looked at.
    let printable = crate::strings::copy_printable::printable_from(name, 0x1F)?;
    net.vars
        .player_name
        .write_at(0, &printable, "a printable name")?;
    // Converted: `rc2k_core::net::record::own_record`.
    let player = play.players.players[0].get();
    let own = record::own_record(
        player.car_number as u8,
        player.gearing as u8,
        player.nationality as u8,
        race.shared.state.settings.stage.get() as u8,
        race.shared.state.settings.direction.get() as u8,
        player.race_number as u8,
    );
    for (n, b) in own.into_iter().enumerate() {
        net.vars.own.head.set(n, b);
    }
    net.vars
        .own
        .head
        .set(record::RANDOM, { crate::math::next_random::next_random() }
            as u8);
    net.vars.own.race_time.set(0);
    net.vars.own.total.set(0);
    for (to, from) in SETTINGS {
        to.set(from(&player));
    }
    race.shared.state.results.hud_bits_current.set(u32::MAX);
    for (to, from) in MORE {
        to.set(from.get());
    }
    race.shared.state.results.finished.set(0);
    Ok(())
}
