//! `drive::race_loop` (0x444D46): the race played until it ends. The eight blocks' +0xFAC 10,
//! `WindowLoopState.paused` (paused) and `scene_switch` 0; the clock (`scene::restart_clock`) into
//! `clock_a`, `5CC3B0`; 0x20 dwords at `marks` and `play_end` -1. Then, with the
//! tenth's base `race_clock_base` the clock each time round: a window not active pauses
//! (`WindowLoopState.paused` 1); unpaused, `input::update`; in the attract mode (`in_credits`) a key
//! (`drive::any_key_down`) or `Credits.done` ends it; key 1 (Esc) asks to quit
//! (`drive::quit_question`: no goes on, else the answer into the cars' +0xB0 (a replay's car's
//! only), with two cars a finished car's 2) and ends it; the views' bit 0x40 (+0x468,
//! `g_viewBlocks`; the second with two views) while not holding (`hold_count` 0) pauses; else
//! `race::race_keys`, `drive::race_step`, `race::tyre_change`, all out (`race::players_all_out`)
//! ends it, and the race's frame (`drive::race_frame`, stream A's: through the game). Paused: in a
//! replay its car's +0x468 0, else `drive::pause_until_key`; unpaused again.
//!
//! Caller: `drive::play_race`. Refuses what its calls refuse.

use crate::drive::race_parts::{
    input_update, players_all_out, race_frame, race_keys, restart_clock, tyre_change,
};
use crate::errors::{BadInput, Refused};
use crate::net::owner::Net;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;

fn view_bit(shared: &RaceShared, view: &crate::session::PlainDword) -> Result<bool, BadInput> {
    let car = shared
        .car_blocks()
        .of(view.get())
        .ok_or(BadInput("a view's block"))?;
    Ok(car.input_keys.get() & 0x40 != 0)
}

/// The network player's car block.
fn net_player_car(
    net: &Net,
    shared: &RaceShared,
) -> Result<&'static crate::physics::car_store::Car, BadInput> {
    let handle = shared
        .work
        .get()
        .car_blocks
        .get(net.vars.player.get() as usize)
        .ok_or(BadInput("the network player's car"))?;
    shared
        .car_blocks()
        .of(handle.get())
        .ok_or(BadInput("the network player's car"))
}

/// The quit answer `answer` put into the cars (0x444EB5 ..).
fn put_answer(race: &Race, net: &Net, answer: u32) -> Result<(), BadInput> {
    let shared = &race.shared;
    if net.vars.session.get() != 0 {
        net_player_car(net, shared)?.stop_state.set(answer);
        return Ok(());
    }
    race.car_blocks().cars[0].stop_state.set(answer);
    race.car_blocks().cars[1].stop_state.set(answer);
    if shared.view.get().players.get() != 2 {
        return Ok(());
    }
    if race.car_blocks().cars[0].finish_flag.get() != 0 {
        race.car_blocks().cars[0].stop_state.set(2);
    }
    if race.car_blocks().cars[1].finish_flag.get() != 0 {
        race.car_blocks().cars[1].stop_state.set(2);
    }
    Ok(())
}

pub fn race_loop(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (input, system, net) = (o.input, o.system, o.net);
    let devices = &input.devices;
    for c in 0..8 {
        race.car_blocks().cars[c as usize].reset_marker.set(10);
    }
    system.window_loop().paused.set(0);
    race.shared.vars.get().scene_switch.set(0);
    restart_clock(race);
    race.frame
        .clock_ticks()
        .clock_a
        .set(race.shared.physics_tables.get().scene_clock.get());
    race.frame
        .clock_ticks()
        .sound_tick_last
        .set(race.shared.physics_tables.get().scene_clock.get());
    for mark in &race.frame.clock_ticks().marks {
        mark.set(u32::MAX);
    }
    race.replay.replay_view().play_end.set(u32::MAX);
    loop {
        restart_clock(race);
        race.shared
            .physics_tables
            .get()
            .race_clock_base
            .set(race.shared.physics_tables.get().scene_clock.get());
        // 0x444DE9: a frame.
        let paused = loop {
            if system.session.flags.window_active.get() == 0 {
                system.window_loop().paused.set(1);
            }
            if system.window_loop().paused.get() != 0 {
                break true;
            }
            let begun = crate::debug::timing::start();
            input_update(o);
            crate::debug::timing::stop(crate::debug::timing::Stage::Input, begun);
            if race.shared.work.get().in_credits.is_set() {
                if race.frame.credits().done.get() != 0 {
                    crate::framework::log("drive::race_loop: the credits race ends: rolled off");
                    return Ok(());
                }
                let or = crate::drive::any_key_down::any_key_down(input);
                if or != 0 {
                    // Which keys (`g_keysPressed`, by scan code), and the window's own state
                    // of them (`g_keysDown`), for the credits race that ends at once after
                    // the intro.
                    let keys = |table: &crate::session::Bytes<0x80>| {
                        (0..0x80usize)
                            .filter(|&n| table.get(n) != 0)
                            .map(|n| format!("{n:#x}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    };
                    crate::framework::log(&format!(
                        "drive::race_loop: the credits race ends: keys pressed [{}], down [{}], \
                         frame {}",
                        keys(&devices.keys_pressed),
                        keys(&devices.keys_down),
                        race.shared.physics_tables.get().scene_clock.get()
                    ));
                    return Ok(());
                }
            }
            if devices.keys_pressed.get(1) != 0 {
                let answer = { crate::drive::quit_question::quit_question(race, o) }?;
                if answer == 0 {
                    break false;
                }
                if race.shared.work.get().in_credits.is_set() {
                    crate::framework::log(&format!(
                        "drive::race_loop: the credits race ends: the quit question (Escape latched), answer {answer:#x}, frame {}",
                        race.shared.physics_tables.get().scene_clock.get()
                    ));
                }
                put_answer(race, net, answer)?;
                return Ok(());
            }
            if race.shared.vars.get().hold_count.get() == 0 {
                if view_bit(&race.shared, &race.shared.view.get().view_blocks[0])? {
                    break true;
                }
                if race.shared.view.get().views.get() != 1
                    && view_bit(&race.shared, &race.shared.view.get().view_blocks[1])?
                {
                    break true;
                }
            }
            if system.window_loop().paused.get() != 0 {
                break true;
            }
            race_keys(race, o)?;
            let begun = crate::debug::timing::start();
            { crate::drive::race_step::race_step(o, race) }?;
            crate::debug::timing::stop(crate::debug::timing::Stage::Physics, begun);
            tyre_change(o)?;
            if players_all_out(race, net)? {
                if race.shared.work.get().in_credits.is_set() {
                    crate::framework::log(&format!(
                        "drive::race_loop: the credits race ends: every player out, frame {}",
                        race.shared.physics_tables.get().scene_clock.get()
                    ));
                }
                return Ok(());
            }
            race_frame(race, o)?;
        };
        if !paused {
            continue;
        }
        // 0x444F0F: paused.
        if net.vars.session.get() != 0 {
            net_player_car(net, &race.shared)?.input_keys.set(0);
        } else {
            { crate::drive::pause_until_key::pause_until_key(race, o) }?;
        }
        system.window_loop().paused.set(0);
    }
}
