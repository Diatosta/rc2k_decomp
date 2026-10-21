//! `net::lobby_loop` (0x417A0D): the lobby: no players, not racing (`dword_52AD84` 0), the
//! background; the list's clocks, no editing, item 3 (the chat); the clocks of the players'
//! listing and of the sync noted, the sync table to clear (`dword_52AD8C` 1), the sync
//! counter (`byte_52A914`) up by one; the car item on, the stage item only for the host.
//! Each frame (`net::lobby_frame` through `screens::run_frame`): once a second the players
//! listed (`net::list_players_regs`); a key (`menu::list_take_key`): to the chat message
//! while it is typed (`net::chat_key`); else, unless this player is ready (state bit 1),
//! left (0x69) and right (0x68) move over the items 0 .. 3 skipping those off (and not below
//! 1 after a race, `dword_52AD80`); Enter on 0 makes this player ready (state & ~6 | 1, the
//! highlight to 3), on 1 the car (`net::choose_car`), on 2 the stage (`net::choose_stage`),
//! each then the players listed, on 3 a chat message (`net::begin_chat`); 0x0F switches the
//! gaps' display (`dword_52AC30`); Escape leaves (-1); 'R' and the lexicon's key as in the
//! other lists. On the host, with two players or more all ready (each state's bit 1, those
//! with bit 0x80 left out), each player not yet answering is sent "@" and the sync counter
//! once a second ('@' messages, `net::dplay_send` to its id) until all have (`byte_52AD90`,
//! filled by the receiving thread); then everyone has raced (`dword_52AD78` 1) and it ends
//! with 1, as on the others when `dword_52AD78` is 1 (set by the host's message). All
//! players' state bit 4 (racing) clear (or no player) ends the "after a race"
//! (`dword_52AD80` 0). The game closing ends it with -1.
//!
//! In:  nothing
//! Out: eax = 1 (the race), -1 (left); carry set if negative (else the flags of
//!      `test eax, eax`); the rest kept (`pusha` .. `popa`)
//! A failed listing would give -2, but `net::list_players_regs` never comes back with the
//! carry (section 16). Caller: `net::start_link`. Its calls through `part` (the frames
//! through the game). The whole screen runs inside it: ungated. Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins (the keys and the clock
//! scripted, the other players' states given), against the Rust version.

use crate::errors::Refused;
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::choose_connection::chosen;
use crate::net::lobby::{
    ALL_RACED, BACK_FROM_RACE, BLOCK, CAR_ITEM, CHOICE, CLOCK, CLOCK_A, EDITED, FLAGS, FRAME_CLOCK,
    KEY_CLOCK, LIST_TAKE_KEY, MAYBE_LEXICON, MOVE_CLOCK, OWN, PLAYER, PLAYERS, Part, QUITTING,
    RACING, RUN_FRAME, SCREENSHOT, SCREENSHOTS_ON, SEND, SEND_BUFFER, SHOW_GAPS, STAGE_ITEM, SYNC,
    SYNC_CLOCK, SYNC_FIRST, SYNCED, Saved, TICK, a, answer, block, dispatch_lobby, dword, g, game,
    gb, p, pb, region, run_both,
};
use crate::net::types::NetPlayerBlock;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::lobby_loop", rva(0x417A0D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const SHOW_BACKGROUND: u32 = 0x4173E7; // net::show_background
const FRAME: u32 = 0x417D95; // net::lobby_frame
const LIST_PLAYERS: u32 = 0x417A03; // net::list_players_regs
const CHAT_KEY: u32 = 0x4180E4; // net::chat_key
const CHOOSE_CAR: u32 = 0x4182FA; // net::choose_car
const CHOOSE_STAGE: u32 = 0x418347; // net::choose_stage
const BEGIN_CHAT: u32 = 0x4180C5; // net::begin_chat

/// The players' states and-ed together (those with bit 0x80 left out), starting from
/// `bits`, as the original's al: over `count` players (at least one turn: `count` > 0).
pub fn all_states(r: &mut Regs, bits: u8, count: u32) -> u8 {
    let state = |n: u32| gb_at(block(n) + field!(NetPlayerBlock, record.state));
    // Converted: `rc2k_core::net::players::all_in_state`; the registers as the loop leaves
    // them (ah the last player's state, again read).
    let al = rc2k_core::net::players::all_in_state(bits, count, state);
    let ah = state(count.wrapping_sub(1));
    r.eax = r.eax & !0xFFFF | (ah as u32) << 8 | al as u32;
    r.ebx = block(0).wrapping_add(BLOCK.wrapping_mul(count));
    r.ecx = 0;
    al
}

/// `mov al, bits; lea ebx, g_netReceived; mov ecx, g_netPlayers`: the registers before the
/// count is looked at.
pub fn states_start(r: &mut Regs, bits: u8) {
    r.eax = r.eax & !0xFF | bits as u32;
    r.ebx = block(0);
    r.ecx = g(PLAYERS);
}

fn gb_at(at: u32) -> u8 {
    unsafe { (at as *const u8).read() }
}

/// The sync with the other players (the host's): the table cleared the first time (this
/// player answered), once a second message `kind` and the sync counter sent to each player
/// that has not answered; true when all have. Shared with `net::race_wait`.
pub fn sync(r: &mut Regs, kind: u8, part: Part) -> Result<bool, Refused> {
    r.eax = g(SYNC_FIRST);
    p(SYNC_FIRST, 0);
    if r.eax != 0 {
        r.eax = 0;
        r.ecx = 0;
        unsafe { std::ptr::write_bytes(a(SYNCED) as *mut u8, 0, 8) };
        r.eax = g(PLAYER);
        pb(SYNCED + r.eax as usize, 1);
    }
    part(TICK, r)?;
    r.ebx = r.eax;
    r.eax = r.eax.wrapping_sub(g(SYNC_CLOCK));
    if (r.eax.wrapping_sub(1000) as i32) >= 0 {
        p(SYNC_CLOCK, r.ebx);
        r.ebx = block(0);
        r.ecx = 0;
        loop {
            if gb(SYNCED + r.ecx as usize) != 1 {
                let (kept_ebx, kept_ecx) = (r.ebx, r.ecx);
                r.eax = unsafe {
                    ((r.ebx + field!(NetPlayerBlock, id)) as *const u32).read_unaligned()
                };
                r.ebx = a(SEND_BUFFER);
                r.ecx = kind as u32 | (gb(SYNC) as u32) << 8;
                unsafe { (r.ebx as *mut u16).write_unaligned(r.ecx as u16) };
                r.ecx = 2;
                r.edx = 0;
                part(SEND, r)?;
                r.ecx = kept_ecx;
                r.ebx = kept_ebx;
            }
            r.ebx = r.ebx.wrapping_add(BLOCK);
            r.ecx = r.ecx.wrapping_add(1);
            if r.ecx >= g(PLAYERS) {
                break;
            }
        }
    }
    r.ecx = 0;
    r.edx = 0;
    loop {
        let dl = (r.edx as u8).wrapping_add(gb(SYNCED + r.ecx as usize));
        r.edx = r.edx & !0xFF | dl as u32;
        r.ecx = r.ecx.wrapping_add(1);
        if r.ecx >= g(PLAYERS) {
            break;
        }
    }
    Ok(r.ecx == r.edx)
}

/// An item move: from the highlight by `step` while it lands on an item that is off.
fn step_item(step: u32) {
    loop {
        p(CHOICE, g(CHOICE).wrapping_add(step));
        let c = g(CHOICE);
        if c == 1 && g(CAR_ITEM) != 1 {
            continue;
        }
        if c == 2 && g(STAGE_ITEM) != 1 {
            continue;
        }
        break;
    }
}

/// The Rust version: the registers it leaves.
pub fn lobby_loop(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(PLAYERS, 0);
    p(RACING, 0);
    part(SHOW_BACKGROUND, &mut r)?;
    r.eax = g(CLOCK);
    p(MOVE_CLOCK, r.eax);
    p(KEY_CLOCK, r.eax);
    p(EDITED, u32::MAX);
    p(CHOICE, 3);
    part(TICK, &mut r)?;
    p(CLOCK_A, r.eax);
    p(SYNC_CLOCK, r.eax);
    p(SYNC_FIRST, 1);
    pb(SYNC, gb(SYNC).wrapping_add(1));
    p(CAR_ITEM, 1);
    p(STAGE_ITEM, 1);
    if gb(OWN) & 0x10 == 0 {
        p(STAGE_ITEM, 0);
    }
    loop {
        r.eax = game(FRAME);
        part(RUN_FRAME, &mut r)?;
        part(TICK, &mut r)?;
        r.ebx = r.eax;
        r.eax = r.eax.wrapping_sub(g(CLOCK_A));
        if (r.eax.wrapping_sub(1000) as i32) >= 0 {
            p(CLOCK_A, r.ebx);
            part(LIST_PLAYERS, &mut r)?;
            if r.carry() {
                p(CHOICE, 0xFFFF_FFFE);
                return Ok(chosen(entry, &r));
            }
        }
        r.eax = g(CLOCK);
        p(FRAME_CLOCK, r.eax);
        part(LIST_TAKE_KEY, &mut r)?;
        let key = r.eax as u8;
        // None: nothing more this frame but the checks; Some(true): the move clock too.
        let mut moved = None;
        if g(EDITED) != u32::MAX {
            part(CHAT_KEY, &mut r)?;
        } else {
            let ready = gb(OWN) & 1 != 0;
            if !ready && key == 0x69 {
                step_item(u32::MAX);
                if g(BACK_FROM_RACE) == 1 && (g(CHOICE) as i32) < 1 {
                    p(CHOICE, 1);
                }
                if (g(CHOICE) as i32) < 0 {
                    p(CHOICE, 0);
                }
                moved = Some(true);
            } else if !ready && key == 0x68 {
                step_item(1);
                if (g(CHOICE) as i32) > 3 {
                    p(CHOICE, 3);
                }
                moved = Some(true);
            } else if key == 0x1C {
                match g(CHOICE) {
                    0 => {
                        pb(OWN, gb(OWN) & 0xF9 | 1);
                        p(CHOICE, 3);
                        moved = Some(true);
                    }
                    1 | 2 => {
                        part(
                            if g(CHOICE) == 1 {
                                CHOOSE_CAR
                            } else {
                                CHOOSE_STAGE
                            },
                            &mut r,
                        )?;
                        part(LIST_PLAYERS, &mut r)?;
                        moved = Some(true);
                    }
                    3 => {
                        part(BEGIN_CHAT, &mut r)?;
                        moved = Some(true);
                    }
                    _ => {}
                }
            } else if key == 0x0F {
                p(SHOW_GAPS, g(SHOW_GAPS) ^ 1);
                moved = Some(true);
            } else if key == 1 {
                p(CHOICE, u32::MAX);
                return Ok(chosen(entry, &r));
            } else {
                if g(SCREENSHOTS_ON) != 0 && key == 0x52 {
                    part(SCREENSHOT, &mut r)?;
                }
                part(MAYBE_LEXICON, &mut r)?;
            }
        }
        if moved == Some(true) {
            r.ebx = g(FRAME_CLOCK);
            p(MOVE_CLOCK, r.ebx);
        }
        if gb(OWN) & 0x10 != 0 {
            states_start(&mut r, 1);
            if g(PLAYERS) >= 2
                && all_states(&mut r, 1, g(PLAYERS)) == 1
                && sync(&mut r, b'@', part)?
            {
                p(ALL_RACED, 1);
            }
        }
        if g(ALL_RACED) == 1 {
            p(CHOICE, 1);
            return Ok(chosen(entry, &r));
        }
        states_start(&mut r, 4);
        if g(PLAYERS) >= 1 && all_states(&mut r, 4, g(PLAYERS)) == 4 {
            p(BACK_FROM_RACE, 0);
        }
        if g(QUITTING) == 1 {
            break;
        }
    }
    p(CHOICE, u32::MAX);
    Ok(chosen(entry, &r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, lobby_loop);
}

/// The sync's and the loops' variables, for the tests.
pub fn lobby_regions() -> Vec<crate::service::harness::Region> {
    vec![
        dword(PLAYERS),
        dword(RACING),
        dword(CHOICE),
        dword(EDITED),
        dword(KEY_CLOCK),
        dword(FRAME_CLOCK),
        dword(MOVE_CLOCK),
        dword(CLOCK_A),
        region(OWN, 0x40),
        region(SYNCED, 8),
        region(SYNC, 2),
        region(CAR_ITEM, 0xC),
        dword(SCREENSHOTS_ON),
        region(SEND_BUFFER, 4),
        (block(0), 8 * BLOCK as usize),
    ]
}

/// The players' blocks for a test: ids 0x10 + n, states as given.
pub fn set_players(states: &[u8]) {
    for (n, &s) in states.iter().enumerate() {
        let b = block(n as u32);
        unsafe {
            ((b + field!(NetPlayerBlock, id)) as *mut u32).write_unaligned(0x10 + n as u32);
            ((b + field!(NetPlayerBlock, record.state)) as *mut u8).write(s);
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        SHOW_BACKGROUND,
        TICK,
        RUN_FRAME,
        LIST_PLAYERS,
        LIST_TAKE_KEY,
        CHAT_KEY,
        CHOOSE_CAR,
        CHOOSE_STAGE,
        BEGIN_CHAT,
        SCREENSHOT,
        MAYBE_LEXICON,
        SEND,
    ];
    // pusha; mov g_netPlayers, 0
    let copy = match crate::net::lobby::copy(
        0x417A0D,
        0x388,
        &[
            0x60, 0xC7, 0x05, 0x98, 0xAD, 0x52, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        &[
            0x15, 0x3D, 0x89, 0x8E, 0xA8, 0xBD, 0xCB, 0x1BA, 0x1BF, 0x1CF, 0x1D4, 0x1E4, 0x20D,
            0x212, 0x28D, 0x2D5,
        ],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = lobby_regions();
    let saved = Saved::new(&all);
    // (what, this player's state, the players' states (listed), back from a race, the keys,
    // the frames after which the others answer (fills `byte_52AD90`))
    type Case = (&'static str, u8, Vec<u8>, u32, Vec<u32>, u32);
    let cases: Vec<Case> = vec![
        ("Escape", 0x02, vec![0x02, 0x00], 0, vec![], 99),
        (
            "moves",
            0x00,
            vec![0x00],
            0,
            vec![
                0x69, 0x69, 0x69, 0x69, 0x68, 0x68, 0x68, 0x68, 0x0F, 0x52, 0x31,
            ],
            99,
        ),
        (
            "moves after a race",
            0x10,
            vec![0x14, 0x14],
            1,
            vec![0x69, 0x69, 0x69, 0x68],
            99,
        ),
        (
            "car, stage, chat",
            0x10,
            vec![0x10, 0x00],
            0,
            vec![0x69, 0x69, 0x1C, 0x69, 0x1C, 0x68, 0x68, 0x1C, 0x10, 0x1C],
            99,
        ),
        (
            "ready, the host syncs",
            0x10,
            vec![0x11, 0x81, 0x01],
            0,
            vec![0x69, 0x69, 0x69, 0x1C, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10],
            3,
        ),
        (
            "ready, not the host",
            0x00,
            vec![0x01, 0x01],
            0,
            vec![0x69, 0x69, 0x69, 0x1C, 0x69, 0x10],
            99,
        ),
    ];
    for (what, own, states, back, keys, answered) in cases {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &all,
            FLAGS,
            &[],
            &|| {
                pb(OWN, own);
                p(BACK_FROM_RACE, back);
                p(ALL_RACED, 0);
                p(SHOW_GAPS, 0);
                p(PLAYER, 0);
                p(SCREENSHOTS_ON, 1);
                let states = states.clone();
                let mut take = crate::net::settings_loop::keys(keys.clone());
                let mut clock = 0x5000u32;
                let mut frames = 0;
                answer(Box::new(move |place, r| {
                    take(place, r);
                    match place {
                        TICK => {
                            clock += 400;
                            r.eax = clock;
                        }
                        LIST_PLAYERS => {
                            p(PLAYERS, states.len() as u32);
                            set_players(&states);
                        }
                        RUN_FRAME => {
                            frames += 1;
                            if frames >= answered {
                                unsafe { std::ptr::write_bytes(a(SYNCED) as *mut u8, 1, 8) };
                            }
                        }
                        BEGIN_CHAT => p(EDITED, 3),
                        CHAT_KEY => p(EDITED, u32::MAX),
                        _ => {}
                    }
                }));
            },
            &lobby_loop,
        );
    }
    drop(saved);
    t.finish()
}
