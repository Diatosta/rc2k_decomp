//! `net::lobby_loop` (0x417A0D): the lobby: no players, not racing (`racing` 0), the
//! background; the list's clocks, no editing, item 3 (the chat); the clocks of the players'
//! listing and of the sync noted, the sync table to clear (`sync_first` 1), the sync
//! counter (`sync`) up by one; the car item on, the stage item only for the host.
//! Each frame (`net::lobby_frame` through `screens::run_frame`): once a second the players
//! listed (`net::list_players_regs`); a key (`menu::list_take_key`): to the chat message
//! while it is typed (`net::chat_key`); else, unless this player is ready (state bit 1),
//! left (0x69) and right (0x68) move over the items 0 .. 3 skipping those off (and not below
//! 1 after a race, `after_race`); Enter on 0 makes this player ready (state & ~6 | 1, the
//! highlight to 3), on 1 the car (`net::choose_car`), on 2 the stage (`net::choose_stage`),
//! each then the players listed, on 3 a chat message (`net::begin_chat`); 0x0F switches the
//! gaps' display (`show_gaps`); Escape leaves (-1); 'R' and the lexicon's key as in the
//! other lists. On the host, with two players or more all ready (each state's bit 1, those
//! with bit 0x80 left out), each player not yet answering is sent "@" and the sync counter
//! once a second ('@' messages, `net::dplay_send` to its id) until all have (`synced`,
//! filled by the receiving thread); then everyone has raced (`all_raced` 1) and it ends
//! with 1, as on the others when `all_raced` is 1 (set by the host's message). In the
//! original, all players with state bit 4 (raced) ends the "after a race" (`after_race`
//! 0); here no player with bit 2 (still racing), as a player who joined since
//! never has bit 4, and then the host opens the session again (`net::enable_new_players`).
//! The game closing ends it with -1.
//!
//! Returns 1 (the race) or -1 (left).
//! A failed listing would give -2, but `net::list_players_regs` never comes back with the
//! carry (section 16). Caller: `net::start_link`.

use crate::errors::Refused;
use crate::net::lobby::{list_key, other_key, run_frame};
use crate::net::owner::Net;
use crate::net::store::NetVars;
use crate::net::store::PlayerRecord;
use crate::owners::Owners;

/// The players' states and-ed together (those with bit 0x80 left out), starting from
/// `bits`, as the original's al: over `count` players (at least one turn: `count` > 0).
pub fn all_states(vars: &NetVars, bits: u8, count: u32) -> u8 {
    let state = |n: u32| vars.player_block(n).record.state();
    // Converted: `rc2k_core::net::players::all_in_state`.
    rc2k_core::net::players::all_in_state(bits, count, state)
}

/// The sync with the other players (the host's): the table cleared the first time (this
/// player answered), once a second message `kind` and the sync counter sent to each player
/// that has not answered; true when all have. Shared with `net::race_wait`.
pub fn sync(net: &Net, kind: u8) -> Result<bool, Refused> {
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
                crate::net::dplay_send::dplay_send(
                    &net.vars,
                    id,
                    &net.vars.send_buffer.read()[..2],
                );
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
fn step_item(install: &crate::install::owner::Install, net: &Net, vars: &NetVars, step: u32) {
    loop {
        install
            .pages
            .choice
            .set(install.pages.choice.get().wrapping_add(step));
        let c = install.pages.choice.get();
        if c == 1 && net.setup().car_item.get() != 1 {
            continue;
        }
        if c == 2 && vars.stage_item.get() != 1 {
            continue;
        }
        break;
    }
}

/// 1 when the race is on, -1 (`u32::MAX`) when the lobby was left.
pub fn lobby_loop(o: Owners<'static>) -> Result<u32, Refused> {
    let (install, net, system) = (o.install, o.net, o.system);
    net.vars.players.set(0);
    net.vars.racing.set(0);
    crate::net::show_background::show_background(
        crate::strings::append_string_by_id::StringOwners::of(o),
        o.gfx,
        o.system,
        &net.vars,
    )?;
    let now = system.session.clocks.key_ticks.get();
    install.pages.move_clock.set(now);
    install.pages.key_clock.set(now);
    net.edit().edited.set(u32::MAX);
    install.pages.choice.set(3);
    let tick = crate::system::tick_count::tick_count();
    net.edit().clock_a.set(tick);
    net.vars.sync_clock.set(tick);
    net.vars.sync_first.set(1);
    net.vars.sync.set(net.vars.sync.get().wrapping_add(1));
    net.setup().car_item.set(1);
    net.vars.stage_item.set(1);
    if net.vars.own.state() & 0x10 == 0 {
        net.vars.stage_item.set(0);
    }
    loop {
        run_frame(o.system, || crate::net::lobby_frame::lobby_frame(o))?;
        let tick = crate::system::tick_count::tick_count();
        if (tick
            .wrapping_sub(net.edit().clock_a.get())
            .wrapping_sub(1000) as i32)
            >= 0
        {
            net.edit().clock_a.set(tick);
            // The original's `jb` after it is never taken (`list_players_regs` loses
            // the carry, section 16).
            crate::net::list_players::list_players_regs(&net.vars)?;
        }
        install
            .pages
            .frame_clock
            .set(system.session.clocks.key_ticks.get());
        let key = list_key(install, o.input)?;
        let mut moved = false;
        if net.edit().edited.get() != u32::MAX {
            crate::net::chat_key::chat_key(o.input, net, install, key, o.gfx)?;
        } else {
            let ready = net.vars.own.state() & 1 != 0;
            if !ready && key == 0x69 {
                step_item(install, net, &net.vars, u32::MAX);
                if net.vars.after_race.get() == 1 && (install.pages.choice.get() as i32) < 1 {
                    install.pages.choice.set(1);
                }
                if (install.pages.choice.get() as i32) < 0 {
                    install.pages.choice.set(0);
                }
                moved = true;
            } else if !ready && key == 0x68 {
                step_item(install, net, &net.vars, 1);
                if (install.pages.choice.get() as i32) > 3 {
                    install.pages.choice.set(3);
                }
                moved = true;
            } else if key == 0x1C {
                match install.pages.choice.get() {
                    0 => {
                        net.vars
                            .own
                            .head
                            .set(PlayerRecord::STATE, net.vars.own.state() & 0xF9 | 1);
                        install.pages.choice.set(3);
                        moved = true;
                    }
                    1 | 2 => {
                        if install.pages.choice.get() == 1 {
                            crate::net::choose_car::choose_car(o, &net.vars)?;
                        } else {
                            crate::net::choose_stage::choose_stage(o, &net.vars)?;
                        }
                        crate::net::list_players::list_players_regs(&net.vars)?;
                        moved = true;
                    }
                    3 => {
                        crate::net::begin_chat::begin_chat(net, install)?;
                        moved = true;
                    }
                    _ => {}
                }
            } else if key == 0x0F {
                net.vars.show_gaps.set(net.vars.show_gaps.get() ^ 1);
                moved = true;
            } else if key == 1 {
                install.pages.choice.set(u32::MAX);
                return Ok(install.pages.choice.get());
            } else {
                other_key(o, key)?;
            }
        }
        if moved {
            install
                .pages
                .move_clock
                .set(install.pages.frame_clock.get());
        }
        if net.vars.own.state() & 0x10 != 0
            && net.vars.players.get() >= 2
            && all_states(&net.vars, 1, net.vars.players.get()) == 1
            && sync(net, b'@')?
        {
            net.vars.all_raced.set(1);
        }
        if net.vars.all_raced.get() == 1 {
            install.pages.choice.set(1);
            return Ok(install.pages.choice.get());
        }
        // Changed: the original waited for every player to have raced, which a
        // player who joined since the race never has; now for none to be still racing.
        if net.vars.players.get() >= 1
            && rc2k_core::net::players::none_racing(net.vars.players.get(), |n| {
                net.vars.player_block(n).record.state()
            })
        {
            net.vars.after_race.set(0);
            crate::net::disable_new_players::enable_new_players(net);
        }
        if system.session.flags.quitting.is_set() {
            break;
        }
    }
    install.pages.choice.set(u32::MAX);
    Ok(install.pages.choice.get())
}
