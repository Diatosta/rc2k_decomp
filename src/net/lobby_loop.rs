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
use crate::net::lobby::{list_key, other_key, run_frame};
use crate::net::owner::net;
use crate::net::store::PlayerRecord;
use crate::system::owner::system;

/// The players' states and-ed together (those with bit 0x80 left out), starting from
/// `bits`, as the original's al: over `count` players (at least one turn: `count` > 0).
pub fn all_states(bits: u8, count: u32) -> u8 {
    let state = |n: u32| net().vars.player_block(n).record.state();
    // Converted: `rc2k_core::net::players::all_in_state`.
    rc2k_core::net::players::all_in_state(bits, count, state)
}

/// The sync with the other players (the host's): the table cleared the first time (this
/// player answered), once a second message `kind` and the sync counter sent to each player
/// that has not answered; true when all have. Shared with `net::race_wait`.
pub fn sync(kind: u8) -> Result<bool, Refused> {
    let net = net();
    let first = net.vars.sync_first.get();
    net.vars.sync_first.set(0);
    if first != 0 {
        for n in 0..8 {
            net.vars.synced.set(n, 0);
        }
        net.vars.set_synced(net.vars.player.get() as usize, 1);
    }
    let now = crate::system::tick_count::tick_count();
    if (now
        .wrapping_sub(net.vars.sync_clock.get())
        .wrapping_sub(1000) as i32)
        >= 0
    {
        net.vars.sync_clock.set(now);
        // Counted up with `jb` at the end: at least one turn.
        let mut n = 0;
        loop {
            if net.vars.synced(n as usize) != 1 {
                let id = net.vars.player_block(n).id.get();
                net.vars.send_buffer.set(0, kind);
                net.vars.send_buffer.set(1, net.vars.sync.get());
                crate::net::dplay_send::dplay_send(id, &net.vars.send_buffer.read()[..2]);
            }
            n += 1;
            if n >= net.vars.players.get() {
                break;
            }
        }
    }
    // The answers added up in a byte (again at least one turn).
    let (mut n, mut answered) = (0u32, 0u8);
    loop {
        answered = answered.wrapping_add(net.vars.synced(n as usize));
        n += 1;
        if n >= net.vars.players.get() {
            break;
        }
    }
    Ok(n == answered as u32)
}

/// An item move: from the highlight by `step` while it lands on an item that is off.
fn step_item(step: u32) {
    let vars = &net().vars;
    loop {
        crate::install::owner::install().pages.choice.set(
            crate::install::owner::install()
                .pages
                .choice
                .get()
                .wrapping_add(step),
        );
        let c = crate::install::owner::install().pages.choice.get();
        if c == 1 && vars.car_item.get() != 1 {
            continue;
        }
        if c == 2 && vars.stage_item.get() != 1 {
            continue;
        }
        break;
    }
}

/// The Rust version: 1 when the race is on, -1 (`u32::MAX`) when the lobby was left.
pub fn lobby_loop() -> Result<u32, Refused> {
    let net = net();
    let system = system();
    net.vars.players.set(0);
    net.vars.racing.set(0);
    crate::net::show_background::show_background()?;
    let now = system.session.clocks.key_ticks.get();
    crate::install::owner::install().pages.move_clock.set(now);
    crate::install::owner::install().pages.key_clock.set(now);
    net.vars.edited.set(u32::MAX);
    crate::install::owner::install().pages.choice.set(3);
    let tick = crate::system::tick_count::tick_count();
    net.vars.clock_a.set(tick);
    net.vars.sync_clock.set(tick);
    net.vars.sync_first.set(1);
    net.vars.sync.set(net.vars.sync.get().wrapping_add(1));
    net.vars.car_item.set(1);
    net.vars.stage_item.set(1);
    if net.vars.own.state() & 0x10 == 0 {
        net.vars.stage_item.set(0);
    }
    loop {
        run_frame(crate::net::lobby_frame::lobby_frame)?;
        let tick = crate::system::tick_count::tick_count();
        if (tick.wrapping_sub(net.vars.clock_a.get()).wrapping_sub(1000) as i32) >= 0 {
            net.vars.clock_a.set(tick);
            // The original's `jb` after it is never taken (`list_players_regs` loses
            // the carry, section 16).
            crate::net::list_players::list_players_regs()?;
        }
        crate::install::owner::install()
            .pages
            .frame_clock
            .set(system.session.clocks.key_ticks.get());
        let key = list_key()?;
        let mut moved = false;
        if net.vars.edited.get() != u32::MAX {
            crate::net::chat_key::chat_key(key)?;
        } else {
            let ready = net.vars.own.state() & 1 != 0;
            if !ready && key == 0x69 {
                step_item(u32::MAX);
                if net.vars.after_race.get() == 1
                    && (crate::install::owner::install().pages.choice.get() as i32) < 1
                {
                    crate::install::owner::install().pages.choice.set(1);
                }
                if (crate::install::owner::install().pages.choice.get() as i32) < 0 {
                    crate::install::owner::install().pages.choice.set(0);
                }
                moved = true;
            } else if !ready && key == 0x68 {
                step_item(1);
                if (crate::install::owner::install().pages.choice.get() as i32) > 3 {
                    crate::install::owner::install().pages.choice.set(3);
                }
                moved = true;
            } else if key == 0x1C {
                match crate::install::owner::install().pages.choice.get() {
                    0 => {
                        net.vars
                            .own
                            .head
                            .set(PlayerRecord::STATE, net.vars.own.state() & 0xF9 | 1);
                        crate::install::owner::install().pages.choice.set(3);
                        moved = true;
                    }
                    1 | 2 => {
                        if crate::install::owner::install().pages.choice.get() == 1 {
                            crate::net::choose_car::choose_car()?;
                        } else {
                            crate::net::choose_stage::choose_stage()?;
                        }
                        crate::net::list_players::list_players_regs()?;
                        moved = true;
                    }
                    3 => {
                        crate::net::begin_chat::begin_chat()?;
                        moved = true;
                    }
                    _ => {}
                }
            } else if key == 0x0F {
                net.vars.show_gaps.set(net.vars.show_gaps.get() ^ 1);
                moved = true;
            } else if key == 1 {
                crate::install::owner::install().pages.choice.set(u32::MAX);
                return Ok(crate::install::owner::install().pages.choice.get());
            } else {
                other_key(key)?;
            }
        }
        if moved {
            crate::install::owner::install()
                .pages
                .move_clock
                .set(crate::install::owner::install().pages.frame_clock.get());
        }
        if net.vars.own.state() & 0x10 != 0
            && net.vars.players.get() >= 2
            && all_states(1, net.vars.players.get()) == 1
            && sync(b'@')?
        {
            net.vars.all_raced.set(1);
        }
        if net.vars.all_raced.get() == 1 {
            crate::install::owner::install().pages.choice.set(1);
            return Ok(crate::install::owner::install().pages.choice.get());
        }
        if net.vars.players.get() >= 1 && all_states(4, net.vars.players.get()) == 4 {
            net.vars.after_race.set(0);
        }
        if system.session.flags.quitting.get() == 1 {
            break;
        }
    }
    crate::install::owner::install().pages.choice.set(u32::MAX);
    Ok(crate::install::owner::install().pages.choice.get())
}
