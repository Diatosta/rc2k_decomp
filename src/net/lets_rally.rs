//! `net::lets_rally` (0x4170E0): "NET: Lets rally", one network race: racing
//! (`dword_52AD84` 1), `dword_75ACE0` 1, `dword_75ACD8` 0, this player's car and colours to
//! `dword_75ACE8` and `dword_75AE2C`; the first player that is not the host found
//! (`dword_52A924`; none: carry); the music stopped, the race's screen mode, the loading
//! picture, the race's sounds (`race::set_up_race_sound_objects`,
//! `sound::set_up_race_sounds`), the race set up (`net::set_up_race`); the clocks noted, the
//! sync table to clear, the players listed; then empty frames (`screens::run_frame`, eax 0)
//! with a key each (`input::take_key`, ebx 1) until Escape, or until everyone waits
//! (`net::race_wait`): then the front surface cleared, the messages taken
//! (`net::copy_received`) and the race driven (`net::lets_race`); after it, everything
//! tidied (`race::tidy_everything`), the music stopped, the menus' screen mode and loading
//! picture, the race's sounds cleared, the sound settings, the engine and the front end's
//! scene set up again (inside `pusha` .. `popa`); with ebx not 0 (Escape, or the car not
//! finished) carry, else the race's time recorded (`net::race_over`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set as above, else the flags of
//!      `test eax, eax` on the race's time
//! Caller: `net::start_link`. Its calls through `part` (`net::lobby_calls`: the Rust versions,
//! the platform's and the long runners through their hooks). The race runs inside it:
//! ungated. Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins, against the Rust version: Escape, a race finished, not finished, no player
//! but the host.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::framework::Regs;
use crate::net::lobby::{
    APPLY_SOUND, CLEAR_FRONT, CLEAR_RACE_SOUNDS, CLOCK_A, COPY_RECEIVED, FIRST_HOST,
    FRONT_END_SCENE, LOADING_PICTURE, LOG_L2_NL, PLAYER, PLAYERS, Part, RACE_SOUNDS, RACING,
    RUN_FRAME, SET_MENU_MODE, SET_RACE_MODE, SET_UP_ENGINE, SET_UP_RACE_SOUNDS, STOP_MUSIC,
    SYNC_CLOCK, SYNC_FIRST, TAKE_KEY, TICK, TIDY_EVERYTHING, a, block, g, lobby_entry, log_text, p,
    test_eax,
};
use crate::net::types::NetPlayerBlock;
use rc2k_core::net::record;

const TEXT: usize = 0x5C2E2A; // "NET: Lets rally"
const SET_UP_RACE: u32 = 0x463A94; // net::set_up_race
const LIST_PLAYERS: u32 = 0x417A03; // net::list_players_regs
const RACE_WAIT: u32 = 0x4171FE; // net::race_wait
const LETS_RACE: u32 = 0x4640AA; // net::lets_race
const RACE_OVER: u32 = 0x41732E; // net::race_over
const SETTINGS: [(usize, u32); 2] = [(0x75ACE0, 1), (0x75ACD8, 0)];
const CAR: usize = 0x75ACE8; // dword_75ACE8
const COLOURS: usize = 0x75AE2C; // dword_75AE2C

/// The Rust version: the registers it leaves.
pub fn lets_rally(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    log_text(LOG_L2_NL, a(TEXT), &r, part)?;
    p(RACING, 1);
    r.esi = block(g(PLAYER));
    for (x, v) in SETTINGS {
        p(x, v);
    }
    // Converted: `rc2k_core::net::record` (`CAR`, `COLOURS`, `first_not_host`).
    let byte = |at: u32| unsafe { (at as *const u8).read() };
    let own = r.esi + field!(NetPlayerBlock, record);
    r.eax = byte(own + record::CAR as u32) as u32;
    p(CAR, r.eax);
    r.eax = byte(own + record::COLOURS as u32) as u32;
    p(COLOURS, r.eax);
    let state = |n: u32| byte(block(n) + field!(NetPlayerBlock, record.state));
    let Some(first) = record::first_not_host(g(PLAYERS), state) else {
        r.set_carry(true);
        return Ok(left(&r));
    };
    r.ebx = block(first);
    r.ecx = first;
    p(FIRST_HOST, r.ecx);
    for place in [
        STOP_MUSIC,
        SET_RACE_MODE,
        LOADING_PICTURE,
        SET_UP_RACE_SOUNDS,
        RACE_SOUNDS,
        SET_UP_RACE,
        TICK,
    ] {
        part(place, &mut r)?;
    }
    p(CLOCK_A, r.eax);
    p(SYNC_CLOCK, r.eax);
    p(SYNC_FIRST, 1);
    part(LIST_PLAYERS, &mut r)?;
    loop {
        r.eax = 0;
        part(RUN_FRAME, &mut r)?;
        r.ebx = 1;
        part(TAKE_KEY, &mut r)?;
        if r.eax as u8 == 1 {
            break;
        }
        part(RACE_WAIT, &mut r)?;
        if r.eax != 0 {
            part(CLEAR_FRONT, &mut r)?;
            part(COPY_RECEIVED, &mut r)?;
            part(LETS_RACE, &mut r)?;
            break;
        }
    }
    let kept = r;
    for place in [
        TIDY_EVERYTHING,
        STOP_MUSIC,
        SET_MENU_MODE,
        LOADING_PICTURE,
        CLEAR_RACE_SOUNDS,
        APPLY_SOUND,
        SET_UP_ENGINE,
        FRONT_END_SCENE,
        SET_UP_ENGINE,
    ] {
        part(place, &mut r)?;
    }
    r = Regs {
        eflags: r.eflags,
        ..kept
    };
    if r.ebx != 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    part(RACE_OVER, &mut r)?;
    test_eax(&mut r);
    Ok(left(&r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, lets_rally)
}
