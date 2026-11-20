//! The lobby: players listed once a second, keys to the chat message while one is typed, else
//! left and right move over the items (ready, car, stage, chat, and after a race the one that saves its replay), Enter acts on one, 0x0F toggles
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
use rc2k_core::net::screens;

/// The players' states and-ed together (those with bit 0x80 left out), starting from `bits`,
/// over `count` players (at least one turn: `count` > 0).
pub(super) fn all_states(vars: &NetShared, bits: State, count: u32) -> State {
    let state = |n: u32| vars.player_block_or_end(n).record.state();
    // The logic is in `rc2k_core::net::players::all_in_state`.
    rc2k_core::net::players::all_in_state(bits, count, state)
}

/// The sync with the other players (the host's): the table cleared the first time (this
/// player answered), once a second the host's call (`ready`: everyone ready, else everyone
/// done) with the sync counter sent to each player that has not answered; true when all have.
/// Shared with `net::race_wait`.
pub fn sync(
    net: &Net,
    clock: &crate::clock::tick_count::PictureClock,
    ready: bool,
) -> Result<bool, Refused> {
    let first = net.vars.sync_first.get();
    net.vars.sync_first.set(0);
    if first != 0 {
        net.vars.clear_synced();
        net.vars.set_synced(net.race_slot.player() as usize, true)?;
    }
    let now = crate::clock::tick_count::tick_count(clock);
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
            if n >= net.race_slot.players() {
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
        if n >= net.race_slot.players() {
            break;
        }
    }
    Ok(all_answered)
}

/// The lobby's own background back in the picture buffer, after a screen of another (the replay
/// slots', the options') drew its own into it.
fn restore_background(o: Owners<'static>) -> Result<(), Refused> {
    crate::net::show_background::show_background(o)
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
        if c == 1 && net.lobby().car_item.get() != 1 {
            continue;
        }
        if c == 2 && vars.stage_item.get() != 1 {
            continue;
        }
        break;
    }
}

/// Takes the keys that come in the next moments and drops them: a key held down from the screen
/// before (a race's results) repeats for a while.
fn settle_keys(input: &crate::input::owner::Input) {
    let until = std::time::Instant::now() + std::time::Duration::from_millis(400);
    while std::time::Instant::now() < until {
        if crate::input::take_key::take_key(input) == 0 {
            crate::clock::sleep::sleep(5);
        }
    }
}

/// 1 when the race is on, -1 (`u32::MAX`) when the lobby was left.
pub(super) fn lobby_loop(o: Owners<'static>) -> Result<u32, Refused> {
    let (net, system) = (o.net, o.system);
    let list_state = &o.menus.list;
    net.race_slot.set_players(0);
    net.race_slot.set_bots(0);
    net.lobby().racing.set(0);
    // The last race's loading figures do not show in the next.
    crate::net::loading::clear();
    crate::net::progress::clear();
    crate::net::show_background::show_background(o)?;
    let now = system.session.key_clock.ticks.get();
    list_state.move_clock.set(now);
    list_state.key_clock.set(now);
    net.lobby().edit.edited.set(u32::MAX);
    // A key still held from the race's last screen (Enter) is not an Enter on the chat item,
    // which would open the chat before anyone asked for it.
    settle_keys(o.input);
    list_state.choice.set(3);
    let tick = crate::clock::tick_count::tick_count(&system.session.picture_clock);
    net.lobby().listed_at.set(tick);
    net.lobby().sync_clock.set(tick);
    net.vars.sync_first.set(1);
    net.vars
        .receiver
        .sync
        .set(net.vars.receiver.sync.get().wrapping_add(1));
    net.lobby().car_item.set(1);
    net.vars.stage_item.set(1);
    if !net.vars.own.state().is_host() {
        net.vars.stage_item.set(0);
    }
    let mut syncing = false;
    loop {
        run_frame(o, || crate::net::lobby_frame::lobby_frame(o))?;
        // The session is gone (the host left or put this machine out): the lobby ends.
        if !net.link.has_dplay() {
            break;
        }
        let tick = crate::clock::tick_count::tick_count(&system.session.picture_clock);
        if (tick
            .wrapping_sub(net.lobby().listed_at.get())
            .wrapping_sub(1000) as i32)
            >= 0
        {
            net.lobby().listed_at.set(tick);
            // A failed listing is never reported here (`refresh_players` drops the result;
            // kept as is).
            crate::net::list_players::refresh_players(net)?;
        }
        list_state
            .frame_clock
            .set(system.session.key_clock.ticks.get());
        let key = list_key(list_state, o.input)?;
        let mut moved = false;
        if net.lobby().edit.edited.get() != u32::MAX {
            crate::net::chat_key::chat_key(o.strings, net, key, o.gfx)?;
        } else {
            let ready = net.vars.own.state().is_ready();
            let replay_kept = net.lobby().replay_kept.get() == 1;
            let choice = list_state.choice.get();
            if !ready && (key == 0x66 || key == 0x6B) {
                // Up and down change between the bottom row and the one above it.
                let to = screens::other_row_item(choice, key == 0x66);
                if to != choice {
                    list_state.choice.set(to);
                    moved = true;
                }
            } else if !ready && (key == 0x69 || key == 0x68) && choice >= screens::UPPER_ROW_FIRST {
                // Within the upper row.
                let to = screens::upper_row_step(
                    choice,
                    key == 0x68,
                    replay_kept,
                    net.vars.own.state().is_host(),
                );
                list_state.choice.set(to);
                moved = true;
            } else if !ready && key == 0x69 {
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
                if list_state.choice.get() > 3 {
                    list_state.choice.set(3);
                }
                moved = true;
            } else if ready && (key == 0x69 || key == 0x68) {
                // While ready the highlight moves only between the chat and the ready button,
                // which takes the ready back.
                list_state.choice.set(if key == 0x69 { 0 } else { 3 });
                moved = true;
            } else if key == 0x1C {
                match list_state.choice.get() {
                    0 => {
                        let state = net.vars.own.state();
                        // Pressed again, "ready" is taken back, and the others hear of it at
                        // once.
                        net.vars.own.set_state(if state.is_ready() {
                            state.made_not_ready()
                        } else {
                            state.made_ready()
                        });
                        if state.is_ready() {
                            crate::net::list_players::refresh_players(net)?;
                        }
                        list_state.choice.set(3);
                        moved = true;
                    }
                    1 | 2 => {
                        if list_state.choice.get() == 1 {
                            crate::net::choose_car::choose_car(o, &net.vars)?;
                        } else {
                            crate::net::choose_stage::choose_stage(o, &net.vars)?;
                        }
                        crate::net::list_players::refresh_players(net)?;
                        moved = true;
                    }
                    3 => {
                        crate::net::begin_chat::begin_chat(net, list_state)?;
                        moved = true;
                    }
                    screens::SETTINGS_ITEM => {
                        // The options of the menu after a stage: audio, display, the advanced
                        // page and the players' details; the lobby has its own car button, so
                        // the cars' specifications are left out.
                        crate::menu::in_game_options_items::show_in_game_options(o.menus)?;
                        crate::menu::in_game_options_items::hide_car_specs(o.menus)?;
                        crate::menu::in_game_options_loop::run(o)?;
                        restore_background(o)?;
                        moved = true;
                    }
                    // The host's switch for the computer drivers; the others only see it.
                    screens::BOTS_ITEM if net.vars.own.state().is_host() => {
                        crate::net::bots::cycle_own_setting(net)?;
                        moved = true;
                    }
                    screens::START_ITEM if net.vars.own.state().is_host() => {
                        crate::net::bots::toggle_own_start(net)?;
                        moved = true;
                    }
                    screens::SAVE_REPLAY_ITEM if net.lobby().replay_kept.get() == 1 => {
                        // The last race's replay into a slot chosen (the viewer's screens).
                        let viewed = crate::menu::replay_viewer::replay_mode::replay_mode(
                            o.race,
                            o,
                            crate::menu::replay_viewer::Mode::SaveLast,
                        )?;
                        restore_background(o)?;
                        if viewed == crate::menu::replay_viewer::Viewed::NotSaved {
                            crate::net::show_message::show_own_text(
                                o,
                                rc2k_core::strings::own::id::REPLAY_NOT_SAVED,
                            )?;
                            restore_background(o)?;
                        }
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
        let everyone_ready = net.vars.own.state().is_host()
            && net.race_slot.players() >= 2
            && all_states(&net.vars, State::READY, net.race_slot.players()) == State::READY;
        if everyone_ready {
            syncing = true;
            if sync(net, &system.session.picture_clock, true)? {
                net.vars.all_raced.set(1);
            }
        } else if syncing {
            // A player took the ready back: the answers heard so far are of a round that
            // does not count any more.
            syncing = false;
            net.vars.sync_first.set(1);
            net.vars
                .receiver
                .sync
                .set(net.vars.receiver.sync.get().wrapping_add(1));
        }
        if net.vars.all_raced.get() == 1 {
            list_state.choice.set(1);
            return Ok(list_state.choice.get());
        }
        // Waits for no player to be still racing: a player who joined since the race never has
        // the raced bit.
        if net.race_slot.players() >= 1
            && rc2k_core::net::players::none_racing(net.race_slot.players(), |n| {
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
