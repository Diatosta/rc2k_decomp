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

use crate::errors::Refused;
use crate::framework::{CARRY_FLAG, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    APPLY_SOUND, CLEAR_FRONT, CLEAR_RACE_SOUNDS, CLOCK_A, COPY_RECEIVED, FIRST_HOST,
    FRONT_END_SCENE, LOADING_PICTURE, LOG_L2_NL, PLAYER, PLAYERS, Part, RACE_SOUNDS, RACING,
    RUN_FRAME, SET_MENU_MODE, SET_RACE_MODE, SET_UP_ENGINE, SET_UP_RACE_SOUNDS, STOP_MUSIC,
    SYNC_CLOCK, SYNC_FIRST, Saved, TAKE_KEY, TICK, TIDY_EVERYTHING, a, answer, block,
    dispatch_lobby, dword, g, log_text, p, region, run_both, test_eax,
};
use crate::net::lobby_loop::set_players;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::lets_rally", rva(0x4170E0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

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
    let byte = |at: u32| unsafe { (at as *const u8).read() } as u32;
    r.eax = byte(r.esi + 0x25);
    p(CAR, r.eax);
    r.eax = byte(r.esi + 0x26);
    p(COLOURS, r.eax);
    r.ebx = block(0);
    r.ecx = 0;
    loop {
        if byte(r.ebx + 0x24) & 0x10 == 0 {
            break;
        }
        r.ebx = r.ebx.wrapping_add(crate::net::lobby::BLOCK);
        r.ecx = r.ecx.wrapping_add(1);
        if r.ecx >= g(PLAYERS) {
            r.set_carry(true);
            return Ok(left(&r));
        }
    }
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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, lets_rally);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        LOG_L2_NL,
        STOP_MUSIC,
        SET_RACE_MODE,
        LOADING_PICTURE,
        SET_UP_RACE_SOUNDS,
        RACE_SOUNDS,
        SET_UP_RACE,
        TICK,
        LIST_PLAYERS,
        RUN_FRAME,
        TAKE_KEY,
        RACE_WAIT,
        CLEAR_FRONT,
        COPY_RECEIVED,
        LETS_RACE,
        TIDY_EVERYTHING,
        SET_MENU_MODE,
        CLEAR_RACE_SOUNDS,
        APPLY_SOUND,
        SET_UP_ENGINE,
        FRONT_END_SCENE,
        RACE_OVER,
    ];
    // pusha; pusha; lea eax, aNetLetsRally
    let copy = match crate::net::lobby::copy(
        0x4170E0,
        0x11E,
        &[0x60, 0x60, 0x8D, 0x05, 0x2A, 0x2E, 0x5C, 0x00],
        &[
            0x8, 0x76, 0x7B, 0x80, 0x85, 0x8A, 0x8F, 0x94, 0xAD, 0xB4, 0xBE, 0xC7, 0xD0, 0xD5,
            0xDA, 0xE0, 0xE5, 0xEA, 0xEF, 0xF4, 0xF9, 0xFE, 0x103, 0x108, 0x112,
        ],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        dword(RACING),
        dword(PLAYERS),
        dword(PLAYER),
        dword(FIRST_HOST),
        dword(CLOCK_A),
        dword(SYNC_CLOCK),
        dword(SYNC_FIRST),
        region(0x75ACD8, 0x14),
        dword(COLOURS),
        (block(0), 3 * crate::net::lobby::BLOCK as usize),
    ];
    let saved = Saved::new(&all);
    // (what, the states, the turn Escape comes, the turn everyone waits, the race's ebx)
    type Case = (&'static str, Vec<u8>, u32, u32, u32);
    let cases: Vec<Case> = vec![
        ("Escape", vec![0x10, 0x02], 2, 99, 0),
        ("finished", vec![0x10, 0x02], 99, 2, 0),
        ("not finished", vec![0x02, 0x12], 99, 0, u32::MAX),
        ("only the host", vec![0x10], 99, 0, 0),
    ];
    for (what, states, escape_at, go_at, finished) in cases {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &all,
            CARRY_FLAG,
            &[],
            &|| {
                p(PLAYERS, states.len() as u32);
                p(PLAYER, 1);
                set_players(&states);
                unsafe {
                    ((block(1) + 0x25) as *mut u8).write(0x22);
                    ((block(1) + 0x26) as *mut u8).write(0x33);
                }
                let mut turns = 0;
                answer(Box::new(move |place, r| match place {
                    TICK => r.eax = 0x7777,
                    TAKE_KEY => {
                        let key = if turns >= escape_at { 1 } else { 0 };
                        r.eax = r.eax & !0xFF | key;
                        r.set_zero(key == 0);
                    }
                    RACE_WAIT => {
                        r.eax = (turns >= go_at) as u32;
                        turns += 1;
                    }
                    LETS_RACE => {
                        r.eax = 4321;
                        r.ebx = finished;
                    }
                    _ => {}
                }));
            },
            &lets_rally,
        );
    }
    drop(saved);
    t.finish()
}
