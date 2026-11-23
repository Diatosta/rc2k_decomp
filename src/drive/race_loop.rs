//! The race played until it ends: each time round the clock is read, a window not active pauses,
//! unpaused `input::update` runs; in the attract mode a key or the credits' end leaves; Esc, or
//! a player's pause binding going down, asks to quit (`drive::quit_question`) and the answer goes
//! into the cars; else `race::race_keys`, `drive::race_step`, `race::tyre_change` and the race's
//! frame (`drive::race_frame`) run, and all players out ends it. Paused (the window not
//! active): in a network race the car's keys are cleared, else `drive::pause_until_key`.
//! Refuses what its calls refuse.

use crate::drive::race_parts::{
    input_update, players_all_out, race_frame, race_keys, restart_clock, tyre_change,
};
use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use rc2k_core::net::net_race::NetRace;

/// Whether the pause binding is down for a view's car (the key word's bit 0x40).
fn pause_down(
    shared: &RaceShared,
    view: &crate::memory::session::PlainDword,
) -> Result<bool, BadInput> {
    let blocks = shared.car_blocks();
    let car = blocks.of(view.get()).ok_or(BadInput("a view's block"))?;
    Ok(car.input_keys & 0x40 != 0)
}

/// Whether the third or fourth player's pause binding is down (a split screen of three or four
/// players: their cars' blocks; the first two are the views').
fn more_players_pause_down(race: &Race) -> bool {
    let players = race.shared.view.get().players.get().min(4) as usize;
    (2..players).any(|n| race.car_blocks().cars[n].input_keys & 0x40 != 0)
}

/// A view's pause binding going down asks the quit question once: the binding must be let go
/// before it asks again, and the steps just after an answer do not count (the answer clears the
/// cars' keys and the held binding comes back with the next step).
struct PauseBinding {
    was_down: bool,
    settling: u32,
}

impl PauseBinding {
    const SETTLE_TURNS: u32 = 2;

    fn new() -> Self {
        Self {
            was_down: false,
            settling: 0,
        }
    }

    /// Takes this turn's state; true when the binding has just gone down.
    fn pressed(&mut self, down: bool) -> bool {
        if self.settling != 0 {
            self.settling -= 1;
            self.was_down = true;
            return false;
        }
        let pressed = down && !self.was_down;
        self.was_down = down;
        pressed
    }

    fn answered(&mut self) {
        self.settling = Self::SETTLE_TURNS;
        self.was_down = true;
    }
}

pub(crate) fn net_player_car(
    net_race: NetRace,
    shared: &RaceShared,
) -> Result<std::cell::RefMut<'_, crate::physics::car_store::Car>, BadInput> {
    let handle = *shared
        .work()
        .car_blocks
        .get(net_race.player as usize)
        .ok_or(BadInput("the network player's car"))?;
    let index = crate::race::blocks_store::CarBlocks::index_of(handle)
        .ok_or(BadInput("the network player's car"))?;
    Ok(std::cell::RefMut::map(shared.car_blocks_mut(), |blocks| {
        &mut blocks.cars[index]
    }))
}

fn put_answer(race: &Race, net: &crate::net::owner::Net, answer: u32) -> Result<(), BadInput> {
    let shared = &race.shared;
    if let Some(net_race) = net.race_slot.get() {
        net_player_car(net_race, shared)?.stop_state = answer;
        return Ok(());
    }
    let players = shared.view.get().players.get();
    let seats = crate::race::race_cars::seats(race);
    for car in 0..seats {
        race.car_blocks_mut().cars[car].stop_state = answer;
    }
    // One player's race (and no race of more than four) stays as asked; a car that has
    // finished keeps its finish.
    if !(2..=4).contains(&players) {
        return Ok(());
    }
    for car in 0..seats {
        if race.car_blocks().cars[car].finish_flag != 0 {
            race.car_blocks_mut().cars[car].stop_state = 2;
        }
    }
    Ok(())
}

/// Whether a network race with the host's computer drivers goes on while this machine is
/// paused (the quit question up, or the window not active): the other machines' cars and the
/// bots do not stop, so a machine that stopped stepping would see the bots jump when the host's
/// next message for them arrived (or, on the host, the others would see them run on alone).
/// Races without bots stop as before.
pub(crate) fn net_goes_on_paused(o: crate::owners::Owners<'static>) -> bool {
    o.net.race_slot.is_on() && o.net.race_slot.bots() != 0
}

/// One step of a race that goes on while paused (`net_goes_on_paused`): the own car is held
/// still (full brake and handbrake, `drive::car_input_words`) and the race's step runs, which
/// sends and receives as usual.
pub(crate) fn step_while_paused(
    race: &Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    if let Some(net_race) = o.net.race_slot.get() {
        net_player_car(net_race, &race.shared)?.input_keys = 0;
    }
    let held = &crate::drive::car_input_words::HELD;
    held.store(true, std::sync::atomic::Ordering::Relaxed);
    let stepped = { crate::drive::race_step::race_step(o, race) };
    held.store(false, std::sync::atomic::Ordering::Relaxed);
    stepped?;
    Ok(())
}

pub fn race_loop(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (input, system) = (o.input, o.system);
    let devices = &input.devices;
    for c in 0..8 {
        race.car_blocks_mut().cars[c as usize].reset_marker = 10;
    }
    system.window_loop().paused.set(0);
    race.shared.vars.get().scene_switch.set(0);
    crate::remote::race_loop_begins();
    restart_clock(race, &system.session.picture_clock);
    let _racing = crate::race::blend::Racing::begin(race);
    race.frame.clock_ticks_mut().sound_tick_last = race.shared.run().scene_clock;
    race.replay.replay_view().play_end.set(u32::MAX);
    let mut pause_binding = PauseBinding::new();
    let pad_at_start = input.pad_marker();
    loop {
        restart_clock(race, &system.session.picture_clock);
        let scene_clock = race.shared.run().scene_clock;
        race.shared.run_mut().race_clock_base = scene_clock;
        let paused = loop {
            let away = system.session.flags.window_active.get() == 0;
            if away && net_goes_on_paused(o) {
                // Not active, but the race goes on for the others: step it without the keys
                // (the frame is not drawn while the window is not active).
                step_while_paused(race, o)?;
                race_frame(race, o)?;
                if crate::net::left::session_ended() {
                    return Ok(());
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
                continue;
            }
            if away {
                system.window_loop().paused.set(1);
            }
            if system.window_loop().paused.get() != 0 {
                break true;
            }
            let begun = crate::debug::timing::start();
            input_update(o);
            crate::debug::timing::stop(crate::debug::timing::Stage::Input, begun);
            if race.shared.work().in_credits != 0 {
                if race.frame.credits().done != 0 {
                    crate::program_log::log("drive::race_loop: the credits race ends: rolled off");
                    return Ok(());
                }
                let or = crate::drive::any_key_down::any_key_down(input);
                // A pad's A, B or Start ends it as a key does.
                if input.pad_pressed_since(pad_at_start) {
                    crate::program_log::log(
                        "drive::race_loop: the credits race ends: a pad button",
                    );
                    return Ok(());
                }
                if or != 0 {
                    // Which keys (by scan code), and the window's own state of them, for the
                    // credits race that ends at once after the intro.
                    let keys = |table: &crate::memory::session::Bytes<0x80>| {
                        (0..0x80usize)
                            .filter(|&n| table.get(n) != 0)
                            .map(|n| format!("{n:#x}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    };
                    crate::program_log::log(&format!(
                        "drive::race_loop: the credits race ends: keys pressed [{}], down [{}], \
                         frame {}",
                        keys(&devices.keys_pressed),
                        keys(&devices.keys_down),
                        race.shared.run().scene_clock
                    ));
                    return Ok(());
                }
            }
            let binding_down = pause_down(&race.shared, &race.shared.view.get().view_blocks[0])?
                || (race.shared.view.get().views.get() != 1
                    && pause_down(&race.shared, &race.shared.view.get().view_blocks[1])?)
                || more_players_pause_down(race);
            let binding_pressed = pause_binding.pressed(binding_down);
            if devices.keys_pressed.get(1) != 0 || binding_pressed {
                let answer = { crate::drive::quit_question::quit_question(race, o) }?;
                if answer == 0 {
                    pause_binding.answered();
                    break false;
                }
                if race.shared.work().in_credits != 0 {
                    crate::program_log::log(&format!(
                        "drive::race_loop: the credits race ends: the quit question (Escape latched), answer {answer:#x}, frame {}",
                        race.shared.run().scene_clock
                    ));
                }
                put_answer(race, o.net, answer)?;
                return Ok(());
            }
            if system.window_loop().paused.get() != 0 {
                break true;
            }
            race_keys(race, o)?;
            // The host left: the session is over, and this race with it, as if this player had
            // quit it (the car did not finish).
            if crate::net::left::session_ended() {
                race.shared
                    .race_car_mut(race.shared.view.get().local_car.get() as usize)?
                    .stop_state = u32::MAX;
                return Ok(());
            }
            let begun = crate::debug::timing::start();
            { crate::drive::race_step::race_step(o, race) }?;
            crate::debug::timing::stop(crate::debug::timing::Stage::Physics, begun);
            tyre_change(o)?;
            if players_all_out(race, &o.net.race_slot)? {
                if race.shared.work().in_credits != 0 {
                    crate::program_log::log(&format!(
                        "drive::race_loop: the credits race ends: every player out, frame {}",
                        race.shared.run().scene_clock
                    ));
                }
                return Ok(());
            }
            race_frame(race, o)?;
        };
        if !paused {
            continue;
        }
        if let Some(net_race) = o.net.race_slot.get() {
            net_player_car(net_race, &race.shared)?.input_keys = 0;
        } else {
            { crate::drive::pause_until_key::pause_until_key(race, o) }?;
        }
        system.window_loop().paused.set(0);
    }
}
