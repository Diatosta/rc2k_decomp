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
use crate::net::lobby::{a, g, log, log_int_nl, p};
use crate::net::store::NET;
use rc2k_core::net::record;

const LOCAL: usize = 0x63DBA0; // dword_63DBA0
const KINDS: usize = 0x623858; // g_raceCarKinds
const COLOURS: usize = 0x623878; // dword_623878
const NUMBERS: usize = 0x6238D4; // dword_6238D4
const STAGE: usize = 0x623850; // g_stageNumber
const DIRECTION: usize = 0x623854; // dword_623854
const CLEARED: usize = 0x62389C; // dword_62389C
const RACE_CARS: usize = 0x63DB90; // g_raceCars
const ONE: usize = 0x63DB94; // dword_63DB94
const CLEARED_2: usize = 0x621514; // dword_621514
/// The stored blocks' dwords: (to, from).
const STORED: [(usize, usize); 16] = [
    (0x70F424, 0x623878),
    (0x710404, 0x62387C),
    (0x7113E4, 0x623880),
    (0x7123C4, 0x623884),
    (0x7133A4, 0x623888),
    (0x714384, 0x62388C),
    (0x715364, 0x623890),
    (0x716344, 0x623894),
    (0x70FE94, 0x6238D4),
    (0x710E74, 0x6238D8),
    (0x711E54, 0x6238DC),
    (0x712E34, 0x6238E0),
    (0x713E14, 0x6238E4),
    (0x714DF4, 0x6238E8),
    (0x715DD4, 0x6238EC),
    (0x716DB4, 0x6238F0),
];
const RULE: usize = 0x4DDFD1; // "------------------------"
const START: usize = 0x4DDFEA; // "LOADING NETWORK START   "
const COMPLETE: usize = 0x4DE0A7; // "LOADING NETWORK COMPLETE"
const RULE_2: usize = 0x4DE0C0;
/// The counts logged: (the label, the value's variable).
const COUNTS: [(usize, Option<usize>); 11] = [
    (0x4DE003, Some(RACE_CARS)), // "[shaunnumcars]"
    (0x4DE012, Some(ONE)),       // "[shaunnumplayers]"
    (0x4DE024, None),            // "[localnpn]"
    (0x4DE02F, Some(0x623858)),  // "[shauncarnum1]" ..
    (0x4DE03E, Some(0x62385C)),
    (0x4DE04D, Some(0x623860)),
    (0x4DE05C, Some(0x623864)),
    (0x4DE06B, Some(0x623868)),
    (0x4DE07A, Some(0x62386C)),
    (0x4DE089, Some(0x623870)),
    (0x4DE098, Some(0x623874)),
];

/// The Rust version.
pub fn set_up_race() -> Result<(), Refused> {
    // Converted: `rc2k_core::net::record` (`race_players`, `seat`).
    let players = record::race_players(NET.players.get())?;
    p(LOCAL, NET.player.get());
    for n in 0..players {
        let record = &NET.player_block(n).record;
        let seat = record::seat(|n| record.head.get(n));
        let slot = 4 * n as usize;
        p(KINDS + slot, seat.car);
        p(COLOURS + slot, seat.colours);
        p(NUMBERS + slot, seat.race_number);
        if let Some((stage, direction)) = seat.stage {
            p(STAGE, stage);
            p(DIRECTION, direction);
        }
    }
    p(CLEARED, 0);
    p(RACE_CARS, players);
    p(ONE, 1);
    p(CLEARED_2, 0);
    crate::net::reset_cars::reset_cars();
    unsafe {
        crate::race::race_cars::set_up_arcade_cars()?;
        crate::race::race_shell::set_start_states()?;
    }
    for (to, from) in STORED {
        p(to, g(from));
    }
    log(a(RULE), true, 1);
    log(a(START), true, 1);
    for (label, value) in COUNTS {
        log(a(label), false, 1);
        // `None`: this machine's player (`NET.player`), not in the image's old range.
        log_int_nl(value.map_or_else(|| NET.player.get(), g), 1);
    }
    unsafe { crate::race::stage_loading::load_stage() }?;
    log(a(COMPLETE), true, 1);
    log(a(RULE_2), true, 1);
    Ok(())
}
