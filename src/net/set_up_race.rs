//! Sets the network race up from the players' records: each player's car, colours and race
//! number, the host's stage and direction, the race counts; resets the cars' blocks, sets up the
//! arcade cars and start states and loads the stage ("LOADING NETWORK START" ..
//! "LOADING NETWORK COMPLETE"). None or more than eight players are refused.

use crate::errors::Refused;
use crate::net::lobby::{log_bytes, log_int_nl};
use crate::net::store::NetShared;
use crate::race::owner::Race;
use rc2k_core::net::record;

const RULE: &std::ffi::CStr = c"------------------------";
const START: &std::ffi::CStr = c"LOADING NETWORK START   ";
const COMPLETE: &std::ffi::CStr = c"LOADING NETWORK COMPLETE";
const RULE_2: &std::ffi::CStr = c"------------------------";
/// The counts logged: (the label, the value's variable).
const COUNTS: [(&std::ffi::CStr, Count); 11] = [
    (c"[shaunnumcars]", Count::RaceCars),   // "[shaunnumcars]"
    (c"[shaunnumplayers]", Count::Players), // "[shaunnumplayers]"
    (c"[localnpn]", Count::LocalPlayer),    // "[localnpn]"
    (c"[shauncarnum1]", Count::Kind(0)),    // "[shauncarnum1]" ..
    (c"[shauncarnum2]", Count::Kind(1)),
    (c"[shauncarnum3]", Count::Kind(2)),
    (c"[shauncarnum4]", Count::Kind(3)),
    (c"[shauncarnum5]", Count::Kind(4)),
    (c"[shauncarnum6]", Count::Kind(5)),
    (c"[shauncarnum7]", Count::Kind(6)),
    (c"[shauncarnum8]", Count::Kind(7)),
];

/// A count logged: `crate::test_support::own::race().shared.view`'s car and player counts, this machine's
/// player (`crate::test_support::own::net().vars.player`), or a car's kind
/// (`crate::test_support::own::race().shared.vars.race_car_kinds`).
#[derive(Clone, Copy)]
enum Count {
    RaceCars,
    Players,
    LocalPlayer,
    Kind(usize),
}
pub fn set_up_race(
    race: &'static Race,
    vars: &NetShared,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::net::record` (`race_players`, `seat`).
    let players = record::race_players(o.net.race_slot.players())?;
    // The host's computer drivers take cars after the players': the setting comes with the
    // host's record, so every machine reaches the same count.
    let bots = rc2k_core::net::bots::count(crate::net::bots::host_setting(o.net), players);
    o.net.race_slot.set_bots(bots);
    let cars = players + bots;
    race.shared
        .view
        .get()
        .local_car
        .set(o.net.race_slot.player());
    for n in 0..players {
        let record = &vars.player_block(n)?.record;
        let seat = record::seat(|n| record.head.get(n));
        let v = race.shared.vars.get();
        v.race_car_kinds[n as usize].set(seat.car);
        v.car_colours[n as usize].set(seat.colours);
        v.start_numbers[n as usize].set(seat.race_number);
        if let Some((stage, direction)) = seat.stage {
            v.stage_number.set(stage);
            v.direction.set(direction);
        }
    }
    let kinds = crate::net::bots::kinds(race, o.net, bots)?;
    let host_kind = race.shared.vars.get().race_car_kinds[0].get();
    for k in 0..bots {
        let car = (players + k) as usize;
        let v = race.shared.vars.get();
        v.race_car_kinds[car].set(kinds.get(k as usize).copied().unwrap_or(host_kind));
        v.car_colours[car].set(1);
        v.start_numbers[car].set(crate::net::bots::number(race, k));
    }
    race.shared.vars.get().run_kind.set(0);
    race.shared.view.get().race_cars.set(cars);
    race.shared.view.get().players.set(1);
    race.shared.switches.get().editor_mode.set(0);
    crate::net::reset_cars::reset_cars(race)?;
    crate::race::race_cars::set_up_arcade_cars(race)?;
    crate::race::race_shell::set_start_states(race, &o.net.race_slot)?;
    // Each stored block's gearbox kind and start slot value, from the race's tables.
    for (car, colours) in race.car_blocks().cars[..8]
        .iter()
        .zip(&race.shared.vars.get().car_colours)
    {
        car.gearbox_kind.set(colours.get());
    }
    for (car, number) in race.car_blocks().cars[..8]
        .iter()
        .zip(&race.shared.vars.get().start_numbers)
    {
        car.start_slot_value.set(number.get());
    }
    log_bytes(RULE.to_bytes(), true, 1);
    log_bytes(START.to_bytes(), true, 1);
    for (label, value) in COUNTS {
        log_bytes(label.to_bytes(), false, 1);
        let value = match value {
            Count::RaceCars => race.shared.view.get().race_cars.get(),
            Count::Players => race.shared.view.get().players.get(),
            Count::LocalPlayer => o.net.race_slot.player(),
            Count::Kind(n) => race.shared.vars.get().race_car_kinds[n].get(),
        };
        log_int_nl(value, 1);
    }
    { crate::race::stage_loading::load_stage(race, o) }?;
    // The stage bar and the map disc show the players' cars only, not the bots' (the load
    // cleared this).
    race.frame.panel.borrow_mut().network_cars = players;
    log_bytes(COMPLETE.to_bytes(), true, 1);
    log_bytes(RULE_2.to_bytes(), true, 1);
    Ok(())
}
