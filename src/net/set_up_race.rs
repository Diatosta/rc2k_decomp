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
use crate::net::lobby::{g, log, log_int_nl, p};
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
const RULE: &std::ffi::CStr = c"------------------------"; // "------------------------"
const START: &std::ffi::CStr = c"LOADING NETWORK START   "; // "LOADING NETWORK START   "
const COMPLETE: &std::ffi::CStr = c"LOADING NETWORK COMPLETE"; // "LOADING NETWORK COMPLETE"
const RULE_2: &std::ffi::CStr = c"------------------------";
/// The counts logged: (the label, the value's variable).
const COUNTS: [(&std::ffi::CStr, Option<usize>); 11] = [
    (c"[shaunnumcars]", Some(RACE_CARS)), // "[shaunnumcars]"
    (c"[shaunnumplayers]", Some(ONE)),    // "[shaunnumplayers]"
    (c"[localnpn]", None),                // "[localnpn]"
    (c"[shauncarnum1]", Some(0x623858)),  // "[shauncarnum1]" ..
    (c"[shauncarnum2]", Some(0x62385C)),
    (c"[shauncarnum3]", Some(0x623860)),
    (c"[shauncarnum4]", Some(0x623864)),
    (c"[shauncarnum5]", Some(0x623868)),
    (c"[shauncarnum6]", Some(0x62386C)),
    (c"[shauncarnum7]", Some(0x623870)),
    (c"[shauncarnum8]", Some(0x623874)),
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
    log(RULE.as_ptr() as u32, true, 1);
    log(START.as_ptr() as u32, true, 1);
    for (label, value) in COUNTS {
        log(label.as_ptr() as u32, false, 1);
        // `None`: this machine's player (`NET.player`), not in the image's old range.
        log_int_nl(value.map_or_else(|| NET.player.get(), g), 1);
    }
    unsafe { crate::race::stage_loading::load_stage() }?;
    log(COMPLETE.as_ptr() as u32, true, 1);
    log(RULE_2.as_ptr() as u32, true, 1);
    Ok(())
}
