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
//! carry (section 16). Caller: `net::start_link`.

use crate::errors::Refused;
use crate::install::vars::INSTALLER_PAGES;
use crate::net::lobby::{list_key, other_key, run_frame};
use crate::net::store::{NET, PlayerRecord};
use crate::session::CLOCKS;
use crate::session::FLAGS;

/// The players' states and-ed together (those with bit 0x80 left out), starting from
/// `bits`, as the original's al: over `count` players (at least one turn: `count` > 0).
pub fn all_states(bits: u8, count: u32) -> u8 {
    let state = |n: u32| NET.player_block(n).record.state();
    // Converted: `rc2k_core::net::players::all_in_state`.
    rc2k_core::net::players::all_in_state(bits, count, state)
}

/// The sync with the other players (the host's): the table cleared the first time (this
/// player answered), once a second message `kind` and the sync counter sent to each player
/// that has not answered; true when all have. Shared with `net::race_wait`.
pub fn sync(kind: u8) -> Result<bool, Refused> {
    let first = NET.sync_first.get();
    NET.sync_first.set(0);
    if first != 0 {
        unsafe { std::ptr::write_bytes((NET.synced.addr() as u32) as *mut u8, 0, 8) };
        NET.set_synced(NET.player.get() as usize, 1);
    }
    let now = crate::system::tick_count::tick_count();
    if (now.wrapping_sub(NET.sync_clock.get()).wrapping_sub(1000) as i32) >= 0 {
        NET.sync_clock.set(now);
        // Counted up with `jb` at the end: at least one turn.
        let mut n = 0;
        loop {
            if NET.synced(n as usize) != 1 {
                let id = NET.player_block(n).id.get();
                unsafe {
                    ((NET.send_buffer.addr() as u32) as *mut u16)
                        .write_unaligned(kind as u16 | (NET.sync.get() as u16) << 8)
                };
                crate::net::dplay_send::dplay_send(id, 0, NET.send_buffer.addr() as u32, 2);
            }
            n += 1;
            if n >= NET.players.get() {
                break;
            }
        }
    }
    // The answers added up in a byte (again at least one turn).
    let (mut n, mut answered) = (0u32, 0u8);
    loop {
        answered = answered.wrapping_add(NET.synced(n as usize));
        n += 1;
        if n >= NET.players.get() {
            break;
        }
    }
    Ok(n == answered as u32)
}

/// An item move: from the highlight by `step` while it lands on an item that is off.
fn step_item(step: u32) {
    loop {
        INSTALLER_PAGES
            .choice
            .set(INSTALLER_PAGES.choice.get().wrapping_add(step));
        let c = INSTALLER_PAGES.choice.get();
        if c == 1 && NET.car_item.get() != 1 {
            continue;
        }
        if c == 2 && NET.stage_item.get() != 1 {
            continue;
        }
        break;
    }
}

/// The Rust version: 1 when the race is on, -1 (`u32::MAX`) when the lobby was left.
pub fn lobby_loop() -> Result<u32, Refused> {
    NET.players.set(0);
    NET.racing.set(0);
    crate::net::show_background::show_background()?;
    let now = CLOCKS.key_ticks.get();
    INSTALLER_PAGES.move_clock.set(now);
    INSTALLER_PAGES.key_clock.set(now);
    NET.edited.set(u32::MAX);
    INSTALLER_PAGES.choice.set(3);
    let tick = crate::system::tick_count::tick_count();
    NET.clock_a.set(tick);
    NET.sync_clock.set(tick);
    NET.sync_first.set(1);
    NET.sync.set(NET.sync.get().wrapping_add(1));
    NET.car_item.set(1);
    NET.stage_item.set(1);
    if NET.own.state() & 0x10 == 0 {
        NET.stage_item.set(0);
    }
    loop {
        run_frame(crate::net::lobby_frame::lobby_frame)?;
        let tick = crate::system::tick_count::tick_count();
        if (tick.wrapping_sub(NET.clock_a.get()).wrapping_sub(1000) as i32) >= 0 {
            NET.clock_a.set(tick);
            // The original's `jb` after it is never taken (`list_players_regs` loses
            // the carry, section 16).
            crate::net::list_players::list_players_regs()?;
        }
        INSTALLER_PAGES.frame_clock.set(CLOCKS.key_ticks.get());
        let key = list_key()?;
        let mut moved = false;
        if NET.edited.get() != u32::MAX {
            crate::net::chat_key::chat_key(key)?;
        } else {
            let ready = NET.own.state() & 1 != 0;
            if !ready && key == 0x69 {
                step_item(u32::MAX);
                if NET.after_race.get() == 1 && (INSTALLER_PAGES.choice.get() as i32) < 1 {
                    INSTALLER_PAGES.choice.set(1);
                }
                if (INSTALLER_PAGES.choice.get() as i32) < 0 {
                    INSTALLER_PAGES.choice.set(0);
                }
                moved = true;
            } else if !ready && key == 0x68 {
                step_item(1);
                if (INSTALLER_PAGES.choice.get() as i32) > 3 {
                    INSTALLER_PAGES.choice.set(3);
                }
                moved = true;
            } else if key == 0x1C {
                match INSTALLER_PAGES.choice.get() {
                    0 => {
                        NET.own
                            .head
                            .set(PlayerRecord::STATE, NET.own.state() & 0xF9 | 1);
                        INSTALLER_PAGES.choice.set(3);
                        moved = true;
                    }
                    1 | 2 => {
                        if INSTALLER_PAGES.choice.get() == 1 {
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
                NET.show_gaps.set(NET.show_gaps.get() ^ 1);
                moved = true;
            } else if key == 1 {
                INSTALLER_PAGES.choice.set(u32::MAX);
                return Ok(INSTALLER_PAGES.choice.get());
            } else {
                other_key(key)?;
            }
        }
        if moved {
            INSTALLER_PAGES
                .move_clock
                .set(INSTALLER_PAGES.frame_clock.get());
        }
        if NET.own.state() & 0x10 != 0
            && NET.players.get() >= 2
            && all_states(1, NET.players.get()) == 1
            && sync(b'@')?
        {
            NET.all_raced.set(1);
        }
        if NET.all_raced.get() == 1 {
            INSTALLER_PAGES.choice.set(1);
            return Ok(INSTALLER_PAGES.choice.get());
        }
        if NET.players.get() >= 1 && all_states(4, NET.players.get()) == 4 {
            NET.after_race.set(0);
        }
        if FLAGS.quitting.get() == 1 {
            break;
        }
    }
    INSTALLER_PAGES.choice.set(u32::MAX);
    Ok(INSTALLER_PAGES.choice.get())
}
