//! Sets this machine's player up for the network game from player 1: its name, its record
//! (state 0, car, colours, nationality, stage and direction, race number, a random byte, time and
//! total 0) and the race's copy of the player's settings (co-driver, race number, view, HUD
//! switches, steering) with the HUD bits -1. Stage 0 becomes 0x29 with direction 0.

use crate::errors::Refused;
use crate::memory::session::DwordLike;
use crate::menu::options_store::OptionsSnapshot;
use crate::menu::player::Player;
use crate::menu::players_store::PlayerSlots;
use crate::net::owner::Net;
use crate::race::owner::Race;
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
/// Fills the own player's lobby record from the first player's slot and its name.
pub fn fill_own_player(
    race: &Race,
    net: &Net,
    slots: &PlayerSlots,
    options: &OptionsSnapshot,
) -> Result<(), Refused> {
    let name = slots.names.player_names.read();
    // `strings::copy_printable` with 0x1F room: at most 0x1E characters of `name` looked at.
    let printable = crate::strings::copy_printable::printable_from(&name, 0x1F)?;
    net.setup()
        .player_name
        .write_at(0, &printable, "a printable name")?;
    // The logic is in `rc2k_core::net::record::own_record`.
    let player = slots.records[0].get();
    let own = record::own_record(
        player.car_number as u8,
        player.gearing as u8,
        player.nationality as u8,
        race.shared.state.settings.stage.get() as u8,
        race.shared.state.settings.direction.get() as u8,
        player.race_number as u8,
    );
    net.vars.own.update(|r| {
        for (n, b) in own.into_iter().enumerate() {
            r.set_byte(n, b);
        }
    });
    let random = { race.shared.random.next_random() } as u8;
    net.vars.own.update(|r| {
        r.set_byte(record::RANDOM, random);
        r.set_race_time(0);
        r.set_total(0);
    });
    for (to, from) in settings(race) {
        to.set(from(&player));
    }
    race.shared.state.results.hud_bits_current.set(u32::MAX);
    race.shared
        .state
        .settings
        .set_visual_options(&options.visual());
    race.shared.state.results.finished.set(0);
    Ok(())
}
