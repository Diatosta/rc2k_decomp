//! `net::set_up_race` (0x463A94): the network race set up from the players' records: this
//! machine's player to `dword_63DBA0`; each player's car (record +1) to `g_raceCarKinds`,
//! its colours (+2) to `dword_623878`, its race number (+6) to `dword_6238D4`; the host's
//! (state bit 0x10) stage (+4, low seven bits) to `g_stageNumber` and direction (+5, bit 0)
//! to `dword_623854`; `dword_62389C` 0, `g_raceCars` the players, `dword_63DB94` 1,
//! `dword_621514` 0; the cars' blocks reset (`net::reset_cars`), `race::set_up_arcade_cars`,
//! `race::set_start_states`; the colours and the race numbers into the eight cars' stored
//! blocks; the log ("LOADING NETWORK START", the counts, the cars), the stage loaded
//! (`race::load_stage`), "LOADING NETWORK COMPLETE".
//!
//! In:  nothing
//! Out: nothing its one caller, `net::lets_rally` (which has a `pusha`), reads
//! The players' loop ends on `jnz` with the count: none, or more than the tables' eight,
//! would run past them: refused. Caller: `net::lets_rally`.

use crate::errors::Refused;
use crate::net::lobby::{log_bytes, log_int_nl};
use crate::net::store::NetVars;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use rc2k_core::net::record;

const RULE: &std::ffi::CStr = c"------------------------"; // "------------------------"
const START: &std::ffi::CStr = c"LOADING NETWORK START   "; // "LOADING NETWORK START   "
const COMPLETE: &std::ffi::CStr = c"LOADING NETWORK COMPLETE"; // "LOADING NETWORK COMPLETE"
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

/// A count logged: `crate::race::owner::race().shared.view`'s car and player counts, this machine's player (`crate::net::owner::net().vars.player`), or
/// a car's kind (`crate::race::owner::race().shared.vars.race_car_kinds`).
#[derive(Clone, Copy)]
enum Count {
    RaceCars,
    Players,
    LocalPlayer,
    Kind(usize),
}

/// The Rust version.
pub fn set_up_race(race: &Race, vars: &NetVars) -> Result<(), Refused> {
    // Converted: `rc2k_core::net::record` (`race_players`, `seat`).
    let players = record::race_players(vars.players.get())?;
    race.shared.view.local_car.set(vars.player.get());
    for n in 0..players {
        let record = &vars.player_block(n).record;
        let seat = record::seat(|n| record.head.get(n));
        let v = &race.shared.vars;
        v.race_car_kinds[n as usize].set(seat.car);
        v.car_colours[n as usize].set(seat.colours);
        v.start_numbers[n as usize].set(seat.race_number);
        if let Some((stage, direction)) = seat.stage {
            v.stage_number.set(stage);
            v.direction.set(direction);
        }
    }
    race.shared.vars.run_kind.set(0);
    race.shared.view.race_cars.set(players);
    race.shared.view.players.set(1);
    race.shared.switches.editor_mode.set(0);
    crate::net::reset_cars::reset_cars()?;
    crate::race::race_cars::set_up_arcade_cars(race)?;
    crate::race::race_shell::set_start_states()?;
    // Each stored block's gearbox kind and start slot value, from the race's tables.
    for (car, colours) in CAR_BLOCKS.cars[..8]
        .iter()
        .zip(&race.shared.vars.car_colours)
    {
        car.gearbox_kind.set(colours.get());
    }
    for (car, number) in CAR_BLOCKS.cars[..8]
        .iter()
        .zip(&race.shared.vars.start_numbers)
    {
        car.start_slot_value.set(number.get());
    }
    log_bytes(RULE.to_bytes(), true, 1);
    log_bytes(START.to_bytes(), true, 1);
    for (label, value) in COUNTS {
        log_bytes(label.to_bytes(), false, 1);
        let value = match value {
            Count::RaceCars => race.shared.view.race_cars.get(),
            Count::Players => race.shared.view.players.get(),
            Count::LocalPlayer => vars.player.get(),
            Count::Kind(n) => race.shared.vars.race_car_kinds[n].get(),
        };
        log_int_nl(value, 1);
    }
    { crate::race::stage_loading::load_stage(race) }?;
    log_bytes(COMPLETE.to_bytes(), true, 1);
    log_bytes(RULE_2.to_bytes(), true, 1);
    Ok(())
}
