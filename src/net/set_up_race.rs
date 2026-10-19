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
//! Out: ebp = the players, esi the last player's block, eax the last value; the rest as
//!      the calls leave them (no `pusha`: its caller `net::lets_rally` has one)
//! The players' loop ends on `jnz` with the count: none, or more than the tables' eight,
//! would run past them: refused. Caller: `net::lets_rally`. Its calls through `part`
//! (`net::lobby_calls`: the Rust versions, the start states through their hook). Shadow
//! mode runs the original alone (it loads a stage). Self-test: a copy whose calls go to
//! stand-ins, against the Rust version, on one, two and eight players, the host first or
//! not.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    ARCADE_CARS, LOAD_STAGE, LOG_INT_L1_NL, LOG_L1, LOG_L1_NL, PLAYER, PLAYERS, Part, START_STATES,
    Saved, block, dispatch_lobby, dword, fill, g, log_text, p, region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::set_up_race", rva(0x463A94), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const RESET_CARS: u32 = 0x463E52; // net::reset_cars
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
const COUNTS: [(usize, usize); 11] = [
    (0x4DE003, RACE_CARS), // "[shaunnumcars]"
    (0x4DE012, ONE),       // "[shaunnumplayers]"
    (0x4DE024, PLAYER),    // "[localnpn]"
    (0x4DE02F, 0x623858),  // "[shauncarnum1]" ..
    (0x4DE03E, 0x62385C),
    (0x4DE04D, 0x623860),
    (0x4DE05C, 0x623864),
    (0x4DE06B, 0x623868),
    (0x4DE07A, 0x62386C),
    (0x4DE089, 0x623870),
    (0x4DE098, 0x623874),
];

/// The Rust version: the registers it leaves.
pub fn set_up_race(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let players = g(PLAYERS);
    if players == 0 || players > 8 {
        return Err(BadInput("no players, or more than eight").into());
    }
    let mut r = *entry;
    p(LOCAL, g(PLAYER));
    r.ebp = 0;
    loop {
        r.esi = block(r.ebp);
        let b = |offset: u32| unsafe { ((r.esi + offset) as *const u8).read() } as u32;
        let slot = 4 * r.ebp as usize;
        r.eax = b(0x25);
        p(KINDS + slot, r.eax);
        r.eax = b(0x26);
        p(COLOURS + slot, r.eax);
        r.eax = b(0x2A);
        p(NUMBERS + slot, r.eax);
        r.eax = b(0x28);
        if b(0x24) & 0x10 != 0 {
            r.eax &= 0x7F;
            p(STAGE, r.eax);
            r.eax = b(0x29) & 1;
            p(DIRECTION, r.eax);
        }
        r.ebp += 1;
        if r.ebp == players {
            break;
        }
    }
    p(CLEARED, 0);
    p(RACE_CARS, players);
    p(ONE, 1);
    p(CLEARED_2, 0);
    part(RESET_CARS, &mut r)?;
    part(ARCADE_CARS, &mut r)?;
    part(START_STATES, &mut r)?;
    for (to, from) in STORED {
        p(to, g(from));
    }
    log_text(LOG_L1_NL, crate::net::lobby::a(RULE), &r, part)?;
    log_text(LOG_L1_NL, crate::net::lobby::a(START), &r, part)?;
    for (label, value) in COUNTS {
        log_text(LOG_L1, crate::net::lobby::a(label), &r, part)?;
        log_text(LOG_INT_L1_NL, g(value), &r, part)?;
    }
    part(LOAD_STAGE, &mut r)?;
    log_text(LOG_L1_NL, crate::net::lobby::a(COMPLETE), &r, part)?;
    log_text(LOG_L1_NL, crate::net::lobby::a(RULE_2), &r, part)?;
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, set_up_race);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        RESET_CARS,
        ARCADE_CARS,
        START_STATES,
        LOG_L1_NL,
        LOG_L1,
        LOG_INT_L1_NL,
        LOAD_STAGE,
    ];
    let outs: Vec<usize> = std::iter::once(0x8F)
        .chain([0x94, 0x99, 0x165, 0x172])
        .chain((0..11).flat_map(|k| [0x17F + 0x19 * k, 0x18B + 0x19 * k]))
        .chain([0x28B, 0x297, 0x2A4])
        .collect();
    // push g_netPlayer
    let copy = match crate::net::lobby::copy(
        0x463A94,
        0x2AB,
        &[0xFF, 0x35, 0x20, 0xA9, 0x52, 0x00],
        &outs,
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = vec![
        region(STAGE, 0xA4),
        region(RACE_CARS, 0x14),
        dword(CLEARED_2),
        dword(PLAYERS),
        dword(PLAYER),
    ];
    for (to, _) in STORED {
        all.push(dword(to));
    }
    for n in 0..8 {
        all.push((block(n) + 0x20, 0x10));
    }
    let saved = Saved::new(&all);
    for (players, host) in [(1u32, 0u32), (2, 1), (8, 5), (8, 9)] {
        run_both(
            &mut t,
            &format!("{players} players, the host {host}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &[],
            &|| {
                fill(STAGE, 0xA4, 51);
                fill(RACE_CARS, 0x14, 52);
                p(PLAYERS, players);
                p(PLAYER, 1);
                for n in 0..8u32 {
                    let at = (block(n) + 0x20) as usize;
                    for k in 0..0x10u32 {
                        unsafe { ((at as u32 + k) as *mut u8).write((n * 0x11 + k * 7) as u8) };
                    }
                    let state = (block(n) + 0x24) as *mut u8;
                    unsafe { state.write(if n == host { 0x91 } else { 0x02 }) };
                }
            },
            &set_up_race,
        );
    }
    drop(saved);
    t.finish()
}
