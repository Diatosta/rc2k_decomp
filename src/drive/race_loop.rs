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
use crate::physics::tables_store::PHYSICS;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use crate::session::{Dword, FLAGS};

fn view_bit(view: &Dword) -> Result<bool, BadInput> {
    let car = crate::race::blocks_store::car_of(view.get()).ok_or(BadInput("a view's block"))?;
    Ok(car.input_keys.get() & 0x40 != 0)
}

/// The network player's car block.
fn net_player_car() -> Result<&'static crate::physics::car_store::Car, BadInput> {
    let handle = RACE_WORK
        .car_blocks
        .get(crate::net::store::NET.player.get() as usize)
        .ok_or(BadInput("the network player's car"))?;
    crate::race::blocks_store::car_of(handle.get()).ok_or(BadInput("the network player's car"))
}

/// The quit answer `answer` put into the cars (0x444EB5 ..).
fn put_answer(answer: u32) -> Result<(), BadInput> {
    if crate::net::store::NET.session.get() != 0 {
        net_player_car()?.stop_state.set_bits(answer);
        return Ok(());
    }
    CAR_BLOCKS.cars[0].stop_state.set_bits(answer);
    CAR_BLOCKS.cars[1].stop_state.set_bits(answer);
    if RACE_VIEW.players.get() != 2 {
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
    for c in 0..8 {
        CAR_BLOCKS.cars[c as usize].reset_marker.set_bits(10);
    }
    FLAGS.paused.set(0);
    RACE_VARS.scene_switch.set(0);
    restart_clock();
    race.frame
        .clock_ticks
        .clock_a
        .set(PHYSICS.scene_clock.get());
    race.frame
        .clock_ticks
        .sound_tick_last
        .set(PHYSICS.scene_clock.get());
    for mark in &race.frame.clock_ticks.marks {
        mark.set(u32::MAX);
    }
    race.replay.replay_view.play_end.set(u32::MAX);
    loop {
        restart_clock();
        PHYSICS
            .race_clock_base
            .set(crate::physics::tables_store::PHYSICS.scene_clock.get());
        // 0x444DE9: a frame.
        let paused = loop {
            if FLAGS.window_active.get() == 0 {
                FLAGS.paused.set(1);
            }
            if FLAGS.paused.get() != 0 {
                break true;
            }
            let begun = crate::debug::timing::start();
            input_update();
            crate::debug::timing::stop(crate::debug::timing::Stage::Input, begun);
            if RACE_WORK.in_credits.get() != 0 {
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
                        keys(&crate::input::store::DEVICES.keys_pressed),
                        keys(&crate::input::store::DEVICES.keys_down),
                        PHYSICS.scene_clock.get()
                    ));
                    return Ok(());
                }
            }
            if crate::input::store::DEVICES.keys_pressed.get(1) != 0 {
                let answer = { crate::drive::quit_question::quit_question(&race.replay) }?;
                if answer == 0 {
                    break false;
                }
                if RACE_WORK.in_credits.get() != 0 {
                    crate::framework::log(&format!(
                        "drive::race_loop: the credits race ends: the quit question (Escape latched), answer {answer:#x}, frame {}",
                        PHYSICS.scene_clock.get()
                    ));
                }
                put_answer(answer)?;
                return Ok(());
            }
            if RACE_VARS.hold_count.get() == 0 {
                if view_bit(&RACE_VIEW.view_blocks[0])? {
                    break true;
                }
                if RACE_VIEW.views.get() != 1 && view_bit(&RACE_VIEW.view_blocks[1])? {
                    break true;
                }
            }
            if FLAGS.paused.get() != 0 {
                break true;
            }
            race_keys()?;
            let begun = crate::debug::timing::start();
            { crate::drive::race_step::race_step() }?;
            crate::debug::timing::stop(crate::debug::timing::Stage::Physics, begun);
            tyre_change()?;
            if players_all_out()? {
                if RACE_WORK.in_credits.get() != 0 {
                    crate::framework::log(&format!(
                        "drive::race_loop: the credits race ends: every player out, frame {}",
                        PHYSICS.scene_clock.get()
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
        if crate::net::store::NET.session.get() != 0 {
            net_player_car()?.input_keys.set_bits(0);
        } else {
            { crate::drive::pause_until_key::pause_until_key() }?;
        }
        FLAGS.paused.set(0);
    }
}
