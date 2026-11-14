//! The lobby: players listed once a second, keys to the chat message while one is typed, else
//! left and right move over the items (ready, car, stage, chat), Enter acts on one, 0x0F toggles
//! the gaps' display, Escape leaves. On the host, once two or more players are all ready, each is
//! sent "@" with the sync counter until all have answered; then everyone races and the loop ends
//! with 1. After a race, the "after a race" state ends when no player is still racing (a player
//! who joined since never has the raced bit), and the host opens the session again
//! (`net::enable_new_players`). Returns 1 (the race) or -1 (left).

use crate::errors::Refused;
use crate::net::lobby::{list_key, other_key, run_frame};
use crate::net::owner::Net;
use crate::net::store::NetShared;
use crate::owners::Owners;
use rc2k_core::net::message::Message;
use rc2k_core::net::players::State;

/// The players' states and-ed together (those with bit 0x80 left out), starting from `bits`,
/// over `count` players (at least one turn: `count` > 0).
pub fn all_states(vars: &NetShared, bits: State, count: u32) -> State {
    let state = |n: u32| vars.player_block_or_end(n).record.state();
    // The logic is in `rc2k_core::net::players::all_in_state`.
    rc2k_core::net::players::all_in_state(bits, count, state)
}

/// The sync with the other players (the host's): the table cleared the first time (this
/// player answered), once a second the host's call (`ready`: everyone ready, else everyone
/// done) with the sync counter sent to each player that has not answered; true when all have.
/// Shared with `net::race_wait`.
pub fn sync(net: &Net, ready: bool) -> Result<bool, Refused> {
    let first = net.vars.sync_first.get();
    net.vars.sync_first.set(0);
    if first != 0 {
        net.vars.clear_synced();
        net.vars.set_synced(net.vars.player.get() as usize, true)?;
    }
    let now = crate::system::tick_count::tick_count();
    if (now
        .wrapping_sub(net.lobby().sync_clock.get())
        .wrapping_sub(1000) as i32)
        >= 0
    {
        net.lobby().sync_clock.set(now);
        // Counted up with `jb` at the end: at least one turn.
        let mut n = 0;
        loop {
            if !net.vars.synced(n as usize)? {
                let id = net.vars.player_block(n)?.id.get();
                let call = Message::Call {
                    ready,
                    round: net.vars.receiver.sync.get(),
                }
                .encode();
                net.lobby()
                    .send_buffer
                    .write_at(0, &call, "the send buffer")?;
                crate::net::dplay_send::dplay_send(&net.link, id, &call);
            }
            n += 1;
            if n >= net.vars.players.get() {
                break;
            }
        }
    }
    // Every listed player answered (a lobby with none still looks at the first).
    let mut all_answered = true;
    let mut n = 0u32;
    loop {
        all_answered &= net.vars.synced(n as usize)?;
        n += 1;
        if n >= net.vars.players.get() {
            break;
        }
    }
    Ok(all_answered)
}

/// An item move: from the highlight by `step` while it lands on an item that is off.
fn step_item(
    list_state: &crate::menu::list_state::ListState,
    net: &Net,
    vars: &NetShared,
    step: u32,
) {
    loop {
        list_state
            .choice
            .set(list_state.choice.get().wrapping_add(step));
        let c = list_state.choice.get();
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
    let list_state = &o.menus.list;
    net.vars.players.set(0);
    net.lobby().racing.set(0);
    crate::net::show_background::show_background(o)?;
    let now = system.session.key_clock.ticks.get();
    list_state.move_clock.set(now);
    list_state.key_clock.set(now);
    net.edit().edited.set(u32::MAX);
    list_state.choice.set(3);
    let tick = crate::system::tick_count::tick_count();
    net.edit().clock_a.set(tick);
    net.lobby().sync_clock.set(tick);
    net.vars.sync_first.set(1);
    net.vars
        .receiver
        .sync
        .set(net.vars.receiver.sync.get().wrapping_add(1));
    net.setup().car_item.set(1);
    net.vars.stage_item.set(1);
    if !net.vars.own.state().is_host() {
        net.vars.stage_item.set(0);
    }
    loop {
        run_frame(o, || crate::net::lobby_frame::lobby_frame(o))?;
        let tick = crate::system::tick_count::tick_count();
        if (tick
            .wrapping_sub(net.edit().clock_a.get())
            .wrapping_sub(1000) as i32)
            >= 0
        {
            net.edit().clock_a.set(tick);
            // A failed listing is never reported here (`list_players_regs` drops the result;
            // kept as is).
            crate::net::list_players::list_players_regs(net)?;
        }
        list_state
            .frame_clock
            .set(system.session.key_clock.ticks.get());
        let key = list_key(list_state, o.input)?;
        let mut moved = false;
        if net.edit().edited.get() != u32::MAX {
            crate::net::chat_key::chat_key(o.input, net, install, key, o.gfx)?;
        } else {
            let ready = net.vars.own.state().is_ready();
            if !ready && key == 0x69 {
                step_item(list_state, net, &net.vars, u32::MAX);
                if net.lobby().after_race.get() == 1 && (list_state.choice.get() as i32) < 1 {
                    list_state.choice.set(1);
                }
                if (list_state.choice.get() as i32) < 0 {
                    list_state.choice.set(0);
                }
                moved = true;
            } else if !ready && key == 0x68 {
                step_item(list_state, net, &net.vars, 1);
                if (list_state.choice.get() as i32) > 3 {
                    list_state.choice.set(3);
                }
                moved = true;
            } else if key == 0x1C {
                match list_state.choice.get() {
                    0 => {
                        net.vars.own.set_state(net.vars.own.state().made_ready());
                        list_state.choice.set(3);
                        moved = true;
                    }
                    1 | 2 => {
                        if list_state.choice.get() == 1 {
                            crate::net::choose_car::choose_car(o, &net.vars)?;
                        } else {
                            crate::net::choose_stage::choose_stage(o, &net.vars)?;
                        }
                        crate::net::list_players::list_players_regs(net)?;
                        moved = true;
                    }
                    3 => {
                        crate::net::begin_chat::begin_chat(net, install, list_state)?;
                        moved = true;
                    }
                    _ => {}
                }
            } else if key == 0x0F {
                net.lobby().show_gaps.set(net.lobby().show_gaps.get() ^ 1);
                moved = true;
            } else if key == 1 {
                list_state.choice.set(u32::MAX);
                return Ok(list_state.choice.get());
            } else {
                other_key(o, key)?;
            }
        }
        if moved {
            list_state.move_clock.set(list_state.frame_clock.get());
        }
        if net.vars.own.state().is_host()
            && net.vars.players.get() >= 2
            && all_states(&net.vars, State::READY, net.vars.players.get()) == State::READY
            && sync(net, true)?
        {
            net.vars.all_raced.set(1);
        }
        if net.vars.all_raced.get() == 1 {
            list_state.choice.set(1);
            return Ok(list_state.choice.get());
        }
        // Waits for no player to be still racing: a player who joined since the race never has
        // the raced bit.
        if net.vars.players.get() >= 1
            && rc2k_core::net::players::none_racing(net.vars.players.get(), |n| {
                net.vars.player_block_or_end(n).record.state()
            })
        {
            net.lobby().after_race.set(0);
            crate::net::disable_new_players::enable_new_players(net);
        }
        if system.session.flags.quitting.is_set() {
            break;
        }
    }
    list_state.choice.set(u32::MAX);
    Ok(list_state.choice.get())
}
