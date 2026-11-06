//! `drive::race_loop` (0x444D46): the race played until it ends. The eight blocks' +0xFAC
//! 10, `dword_518398` (paused) and `dword_6235E8` 0; the clock (`scene::restart_clock`)
//! into `dword_5CC3B4`, `5CC3B0`; 0x20 dwords at `unk_5CC1B0` and `dword_5C2538` -1. Then,
//! with the tenth's base `dword_613840` the clock each time round: a window not active
//! pauses (`dword_518398` 1); unpaused, `input::update`; in the attract mode
//! (`dword_71C3E0`) a key (`drive::any_key_down`) or `dword_529854` ends it; key 1 (Esc)
//! asks to quit (`drive::quit_question`: no goes on, else the answer into the cars' +0xB0
//! (a replay's car's only), with two cars a finished car's 2) and ends it; the views' bit
//! 0x40 (+0x468, `g_viewBlocks`; the second with two views) while not holding
//! (`dword_6235EC` 0) pauses; else `race::race_keys`, `drive::race_step`,
//! `race::tyre_change`, all out (`race::players_all_out`) ends it, and the race's frame
//! (`sub_4541BF`, stream A's: through the game). Paused: in a replay its car's +0x468 0,
//! else `drive::pause_until_key`; unpaused again.
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read
//!
//! Caller: `drive::play_race`. Refuses what its calls refuse.

use crate::drive::race_parts::{
    input_update, players_all_out, race_frame, race_keys, restart_clock, tyre_change,
};
use crate::errors::{BadInput, Refused};
use crate::input::owner::input;
use crate::net::owner::net;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::session::Dword;
use crate::system::owner::system;

fn view_bit(view: &Dword) -> Result<bool, BadInput> {
    let car = crate::race::blocks_store::car_of(view.get()).ok_or(BadInput("a view's block"))?;
    Ok(car.input_keys.get() & 0x40 != 0)
}

/// The network player's car block.
fn net_player_car(shared: &Shared) -> Result<&'static crate::physics::car_store::Car, BadInput> {
    let handle = shared
        .work
        .car_blocks
        .get(net().vars.player.get() as usize)
        .ok_or(BadInput("the network player's car"))?;
    crate::race::blocks_store::car_of(handle.get()).ok_or(BadInput("the network player's car"))
}

/// The quit answer `answer` put into the cars (0x444EB5 ..).
fn put_answer(shared: &Shared, answer: u32) -> Result<(), BadInput> {
    if net().vars.session.get() != 0 {
        net_player_car(shared)?.stop_state.set_bits(answer);
        return Ok(());
    }
    CAR_BLOCKS.cars[0].stop_state.set_bits(answer);
    CAR_BLOCKS.cars[1].stop_state.set_bits(answer);
    if shared.view.players.get() != 2 {
        return Ok(());
    }
    if CAR_BLOCKS.cars[0].finish_flag.bits() != 0 {
        CAR_BLOCKS.cars[0].stop_state.set_bits(2);
    }
    if CAR_BLOCKS.cars[1].finish_flag.bits() != 0 {
        CAR_BLOCKS.cars[1].stop_state.set_bits(2);
    }
    Ok(())
}

/// The Rust version. The registers it leaves are read by no caller (liveness: ecx, edi,
/// edx and esi after `drive::play_race`'s call have no read point).
pub fn race_loop(race: &Race) -> Result<(), Refused> {
    let devices = &input().devices;
    let system = system();
    for c in 0..8 {
        CAR_BLOCKS.cars[c as usize].reset_marker.set_bits(10);
    }
    system.session.flags.paused.set(0);
    race.shared.vars.scene_switch.set(0);
    restart_clock();
    race.frame
        .clock_ticks
        .clock_a
        .set(race.shared.physics_tables.scene_clock.get());
    race.frame
        .clock_ticks
        .sound_tick_last
        .set(race.shared.physics_tables.scene_clock.get());
    for mark in &race.frame.clock_ticks.marks {
        mark.set(u32::MAX);
    }
    race.replay.replay_view.play_end.set(u32::MAX);
    loop {
        restart_clock();
        race.shared
            .physics_tables
            .race_clock_base
            .set(race.shared.physics_tables.scene_clock.get());
        // 0x444DE9: a frame.
        let paused = loop {
            if system.session.flags.window_active.get() == 0 {
                system.session.flags.paused.set(1);
            }
            if system.session.flags.paused.get() != 0 {
                break true;
            }
            let begun = crate::debug::timing::start();
            input_update();
            crate::debug::timing::stop(crate::debug::timing::Stage::Input, begun);
            if race.shared.work.in_credits.get() != 0 {
                if race.frame.credits.done.get() != 0 {
                    crate::framework::log("drive::race_loop: the credits race ends: rolled off");
                    return Ok(());
                }
                let or = crate::drive::any_key_down::any_key_down();
                if or != 0 {
                    // Which keys (`g_keysPressed`, by scan code), and the window's own state
                    // of them (`g_keysDown`), for the credits race that ends at once after
                    // the intro (the owner, 2026-10-24).
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
                        race.shared.physics_tables.scene_clock.get()
                    ));
                    return Ok(());
                }
            }
            if devices.keys_pressed.get(1) != 0 {
                let answer = { crate::drive::quit_question::quit_question(race) }?;
                if answer == 0 {
                    break false;
                }
                if race.shared.work.in_credits.get() != 0 {
                    crate::framework::log(&format!(
                        "drive::race_loop: the credits race ends: the quit question (Escape latched), answer {answer:#x}, frame {}",
                        race.shared.physics_tables.scene_clock.get()
                    ));
                }
                put_answer(&race.shared, answer)?;
                return Ok(());
            }
            if race.shared.vars.hold_count.get() == 0 {
                if view_bit(&race.shared.view.view_blocks[0])? {
                    break true;
                }
                if race.shared.view.views.get() != 1 && view_bit(&race.shared.view.view_blocks[1])?
                {
                    break true;
                }
            }
            if system.session.flags.paused.get() != 0 {
                break true;
            }
            race_keys()?;
            let begun = crate::debug::timing::start();
            { crate::drive::race_step::race_step() }?;
            crate::debug::timing::stop(crate::debug::timing::Stage::Physics, begun);
            tyre_change()?;
            if players_all_out()? {
                if race.shared.work.in_credits.get() != 0 {
                    crate::framework::log(&format!(
                        "drive::race_loop: the credits race ends: every player out, frame {}",
                        race.shared.physics_tables.scene_clock.get()
                    ));
                }
                return Ok(());
            }
            race_frame()?;
        };
        if !paused {
            continue;
        }
        // 0x444F0F: paused.
        if net().vars.session.get() != 0 {
            net_player_car(&race.shared)?.input_keys.set_bits(0);
        } else {
            { crate::drive::pause_until_key::pause_until_key(&race.shared) }?;
        }
        system.session.flags.paused.set(0);
    }
}
