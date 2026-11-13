//! Sets this machine's player up for the network game from player 1: its name, its record
//! (state 0, car, colours, nationality, stage and direction, race number, a random byte, time and
//! total 0) and the race's copy of the player's settings (co-driver, race number, view, HUD
//! switches, steering) with the HUD bits -1. Stage 0 becomes 0x29 with direction 0.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::player::Player;
use crate::net::owner::Net;
use crate::race::owner::Race;
use crate::session::DwordLike;
use rc2k_core::net::record;

/// A setting copied: (to, from player 1's record).
type Setting<'a> = (&'a dyn DwordLike, fn(&Player) -> u32);

/// The settings copied.
fn settings(race: &Race) -> [Setting<'_>; 13] {
    [
        (&race.shared.state.settings.cars[0].codriver_sex, |p| {
            p.codriver_sex
        }),
        (&race.shared.state.settings.cars[0].race_number, |p| {
            p.race_number
        }),
        (&race.shared.state.settings.cars[0].view, |p| p.view),
        (&race.shared.state.settings.cars[0].map_area, |p| p.map_area),
        (&race.shared.state.settings.cars[0].rear_mirror, |p| {
            p.rear_mirror
        }),
        (&race.shared.state.settings.cars[0].rev_counter, |p| {
            p.rev_counter
        }),
        (&race.shared.state.settings.cars[0].message_display, |p| {
            p.message_display
        }),
        (&race.shared.state.settings.cars[0].car_info, |p| p.car_info),
        (&race.shared.state.settings.cars[0].driver_info, |p| {
            p.driver_info
        }),
        (&race.shared.state.settings.cars[0].progress_bar, |p| {
            p.progress_bar
        }),
        (&race.shared.state.settings.cars[0].pace_notes, |p| {
            p.pace_notes
        }),
        (&race.shared.state.settings.cars[0].metric_units, |p| {
            p.metric_units
        }),
        (
            &race.shared.state.settings.cars[0].steering_sensitivity,
            |p| p.steering_sensitivity,
        ),
    ]
}
/// Copied after the HUD's bits: (to, from).
fn more<'a>(race: &'a Race, menus: &'a Menus) -> [(&'a dyn DwordLike, &'a dyn DwordLike); 5] {
    [
        (
            &race.shared.state.settings.lens_flare,
            &menus.screens.option_work.lens_flare,
        ),
        (
            &race.shared.state.settings.reflections,
            &menus.screens.option_work.reflections,
        ),
        (
            &race.shared.state.settings.car_shadows,
            &menus.screens.option_work.car_shadows,
        ),
        (
            &race.shared.state.settings.car_detail,
            &menus.screens.option_work.car_detail,
        ),
        (
            &race.shared.state.settings.headlights,
            &menus.screens.option_work.headlights,
        ),
    ]
}
pub fn fill_own_player(race: &Race, net: &Net, menus: &Menus, name: &[u8]) -> Result<(), Refused> {
    // `strings::copy_printable` with 0x1F room: at most 0x1E characters of `name` looked at.
    let printable = crate::strings::copy_printable::printable_from(name, 0x1F)?;
    net.setup()
        .player_name
        .write_at(0, &printable, "a printable name")?;
    // The logic is in `rc2k_core::net::record::own_record`.
    let player = menus.play.players.players[0].get();
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
    for (to, from) in settings(race) {
        to.set(from(&player));
    }
    race.shared.state.results.hud_bits_current.set(u32::MAX);
    for (to, from) in more(race, menus) {
        to.set(from.get());
    }
    race.shared.state.results.finished.set(0);
    Ok(())
}
