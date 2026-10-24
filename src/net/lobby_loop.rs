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
//! Out: 1 (the race), -1 (left); the original also sets the carry if negative (else the
//!      flags of `test eax, eax`); the rest kept (`pusha` .. `popa`)
//! A failed listing would give -2, but `net::list_players_regs` never comes back with the
//! carry (section 16). Caller: `net::start_link`. Its calls through `part` (the frames
//! through the game). The whole screen runs inside it: ungated. Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins (the keys and the clock
//! scripted, the other players' states given), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::field;
use crate::net::lobby::{
    ALL_RACED, BACK_FROM_RACE, CAR_ITEM, CHOICE, CLOCK_A, EDITED, FRAME_CLOCK, KEY_CLOCK,
    MOVE_CLOCK, OWN, PLAYER, PLAYERS, RACING, SEND_BUFFER, SHOW_GAPS, STAGE_ITEM, SYNC, SYNC_CLOCK,
    SYNC_FIRST, SYNCED, a, block, g, gb, list_key, other_key, p, pb, run_frame,
};
use crate::net::types::NetPlayerBlock;
use crate::session::CLOCKS;
use crate::session::FLAGS;

/// The players' states and-ed together (those with bit 0x80 left out), starting from
/// `bits`, as the original's al: over `count` players (at least one turn: `count` > 0).
pub fn all_states(bits: u8, count: u32) -> u8 {
    let state = |n: u32| gb_at(block(n) + field!(NetPlayerBlock, record.state));
    // Converted: `rc2k_core::net::players::all_in_state`.
    rc2k_core::net::players::all_in_state(bits, count, state)
}

fn gb_at(at: u32) -> u8 {
    unsafe { (at as *const u8).read() }
}

/// The sync with the other players (the host's): the table cleared the first time (this
/// player answered), once a second message `kind` and the sync counter sent to each player
/// that has not answered; true when all have. Shared with `net::race_wait`.
pub fn sync(kind: u8) -> Result<bool, Refused> {
    let first = g(SYNC_FIRST);
    p(SYNC_FIRST, 0);
    if first != 0 {
        unsafe { std::ptr::write_bytes(a(SYNCED) as *mut u8, 0, 8) };
        pb(SYNCED + g(PLAYER) as usize, 1);
    }
    let now = crate::system::tick_count::tick_count();
    if (now.wrapping_sub(g(SYNC_CLOCK)).wrapping_sub(1000) as i32) >= 0 {
        p(SYNC_CLOCK, now);
        // Counted up with `jb` at the end: at least one turn.
        let mut n = 0;
        loop {
            if gb(SYNCED + n as usize) != 1 {
                let id = unsafe {
                    ((block(n) + field!(NetPlayerBlock, id)) as *const u32).read_unaligned()
                };
                unsafe {
                    (a(SEND_BUFFER) as *mut u16)
                        .write_unaligned(kind as u16 | (gb(SYNC) as u16) << 8)
                };
                crate::net::dplay_send::dplay_send(id, 0, a(SEND_BUFFER), 2);
            }
            n += 1;
            if n >= g(PLAYERS) {
                break;
            }
        }
    }
    // The answers added up in a byte (again at least one turn).
    let (mut n, mut answered) = (0u32, 0u8);
    loop {
        answered = answered.wrapping_add(gb(SYNCED + n as usize));
        n += 1;
        if n >= g(PLAYERS) {
            break;
        }
    }
    Ok(n == answered as u32)
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

/// The Rust version: 1 when the race is on, -1 (`u32::MAX`) when the lobby was left.
pub fn lobby_loop() -> Result<u32, Refused> {
    p(PLAYERS, 0);
    p(RACING, 0);
    crate::net::show_background::show_background()?;
    let now = CLOCKS.key_ticks.get();
    p(MOVE_CLOCK, now);
    p(KEY_CLOCK, now);
    p(EDITED, u32::MAX);
    p(CHOICE, 3);
    let tick = crate::system::tick_count::tick_count();
    p(CLOCK_A, tick);
    p(SYNC_CLOCK, tick);
    p(SYNC_FIRST, 1);
    pb(SYNC, gb(SYNC).wrapping_add(1));
    p(CAR_ITEM, 1);
    p(STAGE_ITEM, 1);
    if gb(OWN) & 0x10 == 0 {
        p(STAGE_ITEM, 0);
    }
    loop {
        run_frame(crate::net::lobby_frame::lobby_frame)?;
        let tick = crate::system::tick_count::tick_count();
        if (tick.wrapping_sub(g(CLOCK_A)).wrapping_sub(1000) as i32) >= 0 {
            p(CLOCK_A, tick);
            // The original's `jb` after it is never taken (`list_players_regs` loses
            // the carry, section 16).
            crate::net::list_players::list_players_regs()?;
        }
        p(FRAME_CLOCK, CLOCKS.key_ticks.get());
        let key = list_key()?;
        let mut moved = false;
        if g(EDITED) != u32::MAX {
            crate::net::chat_key::chat_key(key)?;
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
                moved = true;
            } else if !ready && key == 0x68 {
                step_item(1);
                if (g(CHOICE) as i32) > 3 {
                    p(CHOICE, 3);
                }
                moved = true;
            } else if key == 0x1C {
                match g(CHOICE) {
                    0 => {
                        pb(OWN, gb(OWN) & 0xF9 | 1);
                        p(CHOICE, 3);
                        moved = true;
                    }
                    1 | 2 => {
                        if g(CHOICE) == 1 {
                            crate::net::choose_car::choose_car()?;
                        } else {
                            crate::net::choose_stage::choose_stage()?;
                        }
                        crate::net::list_players::list_players_regs()?;
                        moved = true;
                    }
                    3 => {
                        crate::net::begin_chat::begin_chat();
                        moved = true;
                    }
                    _ => {}
                }
            } else if key == 0x0F {
                p(SHOW_GAPS, g(SHOW_GAPS) ^ 1);
                moved = true;
            } else if key == 1 {
                p(CHOICE, u32::MAX);
                return Ok(g(CHOICE));
            } else {
                other_key(key)?;
            }
        }
        if moved {
            p(MOVE_CLOCK, g(FRAME_CLOCK));
        }
        if gb(OWN) & 0x10 != 0 && g(PLAYERS) >= 2 && all_states(1, g(PLAYERS)) == 1 && sync(b'@')? {
            p(ALL_RACED, 1);
        }
        if g(ALL_RACED) == 1 {
            p(CHOICE, 1);
            return Ok(g(CHOICE));
        }
        if g(PLAYERS) >= 1 && all_states(4, g(PLAYERS)) == 4 {
            p(BACK_FROM_RACE, 0);
        }
        if FLAGS.quitting.get() == 1 {
            break;
        }
    }
    p(CHOICE, u32::MAX);
    Ok(g(CHOICE))
}
