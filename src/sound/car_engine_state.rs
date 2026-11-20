//! `sound::car_engine_state`: a car's sound object brought up to date for the turn: the revs eased
//! between the physics' readings, the pitch and volume inputs of the engine sounds, the engine's
//! state, the gear-change and back-fire cues, the listener's distance and the Doppler-like factors.
//!
//! Per turn it:
//! - takes a new revs reading in (the last two readings' times and revs kept), or else
//!   extrapolates the revs between readings (at least 1000), unless the replay holds them;
//! - works out the engine sounds' inputs: revs, throttle (over 256) and speed;
//! - steps the engine's state (no revs, up to 1000, running, cut), the smoothed throttle
//!   and its direction and turns, the gear-change cue and the back-fire cues (the logic
//!   is `rc2k_core::sound::engine`);
//! - works out the listener's distance, how fast it changes, and the Doppler-like pitch
//!   and volume factors.
//!
//! Returns the object's sound kind number. Refuses (before writing) a kind past the 36 of the table
//! and a revs extrapolation whose quotient does not fit.

use crate::errors::{BadInput, Refused};
use crate::math::x87::from_int;
use crate::math::x87_float::{X, int, ld, stf};
use crate::race::owner::Race;
use crate::sound::sets_store::KindRecord;
use crate::sound::store::{RaceClock, SoundObjectCell, SurfaceSounds};
use crate::system::owner::System;
use rc2k_core::sound::engine::{Engine, extrapolation, over_256, readings, state};

pub const KIND_COUNT: u32 = 36;
/// The kind of object `o` (its +4, a number into the kinds' table), or a refusal.
pub fn kind_of(
    o: &crate::sound::store::SoundObjectCell,
    kinds: &crate::sound::sets_store::SoundKinds,
) -> Result<u32, BadInput> {
    let n = o.borrow().kind;
    if n >= KIND_COUNT {
        return Err(BadInput("a sound kind past the 36 of the table"));
    }
    kinds.record(n)?;
    Ok(n)
}

/// The object's dwords the engine works on, taken as values.
fn load(o: &crate::sound::store::SoundObjectCell) -> Engine {
    let o = o.borrow();
    Engine {
        new_revs: o.new_revs,
        revs: o.revs,
        throttle: o.throttle,
        gear_event: o.gear_event,
        no_gear: o.no_gear,
        reading_times: o.reading_times,
        reading_revs: o.reading_revs,
        gear: o.gear,
        last_gear: o.last_gear,
        gear_notice: o.gear_notice,
        countdown: o.countdown,
        wait_flag: o.drop_latched,
        last_direction: o.last_direction,
        state: o.state,
        last_state: o.last_state,
        smooth_throttle: o.smooth_throttle,
        throttle_lead: o.throttle_lead,
        turns_up: o.turns_up,
        direction: o.direction,
        turns_down: o.turns_down,
        drop_sound: o.slots[5].sound,
        drop_start: o.slots[5].start,
        backfire_start: o.slots[6].start,
        backfire_playing: o.slots[6].playing,
        backfire_volume: o.slots[6].volume,
        backfire_wait: o.backfire_wait,
        gear_slot_start: o.gear_slot.start,
    }
}

/// The dwords `now` has changed from `before` put back in the object (the others are left alone).
fn store(o: &crate::sound::store::SoundObjectCell, before: &Engine, now: &Engine) {
    let mut o = o.borrow_mut();
    let put = |cell: &mut u32, was: u32, is: u32| {
        if was != is {
            *cell = is;
        }
    };
    put(&mut o.new_revs, before.new_revs, now.new_revs);
    put(&mut o.revs, before.revs, now.revs);
    put(&mut o.gear_event, before.gear_event, now.gear_event);
    put(&mut o.no_gear, before.no_gear, now.no_gear);
    for n in 0..2 {
        put(
            &mut o.reading_times[n],
            before.reading_times[n],
            now.reading_times[n],
        );
        put(
            &mut o.reading_revs[n],
            before.reading_revs[n],
            now.reading_revs[n],
        );
    }
    put(&mut o.last_gear, before.last_gear, now.last_gear);
    put(&mut o.gear_notice, before.gear_notice, now.gear_notice);
    put(&mut o.countdown, before.countdown, now.countdown);
    put(&mut o.drop_latched, before.wait_flag, now.wait_flag);
    put(
        &mut o.last_direction,
        before.last_direction,
        now.last_direction,
    );
    put(&mut o.state, before.state, now.state);
    put(&mut o.last_state, before.last_state, now.last_state);
    put(
        &mut o.smooth_throttle,
        before.smooth_throttle,
        now.smooth_throttle,
    );
    put(
        &mut o.throttle_lead,
        before.throttle_lead,
        now.throttle_lead,
    );
    put(&mut o.turns_up, before.turns_up, now.turns_up);
    put(&mut o.direction, before.direction, now.direction);
    put(&mut o.turns_down, before.turns_down, now.turns_down);
    put(&mut o.slots[5].sound, before.drop_sound, now.drop_sound);
    put(&mut o.slots[5].start, before.drop_start, now.drop_start);
    put(
        &mut o.slots[6].start,
        before.backfire_start,
        now.backfire_start,
    );
    put(
        &mut o.slots[6].playing,
        before.backfire_playing,
        now.backfire_playing,
    );
    put(
        &mut o.slots[6].volume,
        before.backfire_volume,
        now.backfire_volume,
    );
    put(
        &mut o.backfire_wait,
        before.backfire_wait,
        now.backfire_wait,
    );
    put(
        &mut o.gear_slot.start,
        before.gear_slot_start,
        now.gear_slot_start,
    );
}

/// The object `so` brought up to date; its sound kind's number returned.
pub fn car_engine_state(
    so: &crate::sound::store::SoundObjectCell,
    sound: &crate::sound::owner::Sound,
    race: &Race,
    system: &System,
) -> Result<u32, Refused> {
    // The logic is `rc2k_core::sound::engine` (`extrapolation`, `readings`, `state`, `over_256`);
    // the floats go through the x87 helpers.
    let kind = kind_of(so, sound.race.kinds.get())?;
    let record = sound.race.kinds.get().record(kind)?;
    let before = load(so);
    let mut object = before;
    let clock = sound.race.clock.get().borrow().last;
    // The revs extrapolated, worked out first: its division may trap.
    let change = extrapolation(&object, clock, || {
        race.shared.replay_file().hold_car_revs.get()
    })
    .map_err(BadInput::from)?;
    readings(&mut object, clock, change);
    store(so, &before, &object);
    revs_input(so, &record);
    let throttle = so.borrow().throttle;
    so.borrow_mut().throttle_input = f32::from_bits(over_256(from_int(throttle as i32).to_bits()));
    speed_input(so, sound.race.surface.get());
    let before = load(so);
    let mut object = before;
    let low = state(&mut object, || record.flags, clock);
    store(so, &before, &object);
    if let Some(low) = low {
        gear_pitch(so, low, system);
    }
    distance_and_approach(so, &sound.race.clock.get().borrow());
    so.borrow_mut().volume_factor = 0.0;
    ease_pitch_factor(
        so,
        &sound.race.clock.get().borrow(),
        sound.race.surface.get(),
    );
    so.borrow_mut().volume_factor = 1.0;
    // The float's bit pattern compared as a signed integer: 1.0f32 is 0x3F80_0000.
    if so.borrow().distance.to_bits() as i32 > 0x3F80_0000 {
        volume_factor_by_distance(so);
    }
    Ok(kind)
}

// ---- the float arithmetic, step by step as the x87 rounds it ----

/// The revs input of the engine sounds: `(revs - kind.revs_low) * kind.revs_scale`.
fn revs_input(so: &SoundObjectCell, kind: &KindRecord) {
    let revs = int(so.borrow().revs as i32);
    let above_low = revs - ld(kind.revs_low);
    so.borrow_mut().revs_input = stf(above_low * ld(kind.revs_scale));
}

/// The speed input: `|speed| * surface.speed_scale`.
fn speed_input(so: &SoundObjectCell, surface: &SurfaceSounds) {
    let speed = ld(so.borrow().speed).abs();
    so.borrow_mut().speed_input = stf(speed * ld(surface.constants.speed_scale));
}

/// The gear-change cue's pitch from the clock's low four bits `low`: `low / 60 + 0.75` (the factors
/// as the game's float constants). The machine's scratch dword is left holding 0.75.
fn gear_pitch(so: &SoundObjectCell, low: u32, system: &System) {
    system.machine().scratch.set(low);
    let pitch = gear_pitch_value(low);
    system.machine().scratch.set(GEAR_PITCH_BASE);
    so.borrow_mut().gear_pitch = pitch;
}

/// The pitch: `low * (1 / 60) + 0.75`, rounded to a float.
fn gear_pitch_value(low: u32) -> f32 {
    stf(int(low as i32) * ld(GEAR_PITCH_PER_STEP) + ld(GEAR_PITCH_BASE))
}

/// 1 / 60 as a float.
const GEAR_PITCH_PER_STEP: u32 = 0x3C88_8889;
/// 0.75 as a float.
const GEAR_PITCH_BASE: u32 = 0x3F40_0000;

/// The distance to the listener (the root of the place's squares) and how fast it
/// changes (`approach = (old - new distance) * turns a second`).
fn distance_and_approach(so: &SoundObjectCell, clock: &RaceClock) {
    let old = ld(so.borrow().distance);
    let [x, y, z] = [0, 1, 2].map(|n| ld(so.borrow().place[n]));
    let distance = (x * x + (y * y + z * z)).sqrt();
    so.borrow_mut().distance = stf(distance);
    so.borrow_mut().approach = stf((old - distance) * ld(clock.turns_a_second));
}

/// The pitch factor eased toward `(approach + sound speed) * (1 / sound speed)`:
/// `((target - factor) * blend) + factor`.
fn ease_pitch_factor(so: &SoundObjectCell, clock: &RaceClock, surface: &SurfaceSounds) {
    let target = (ld(so.borrow().approach) + ld(clock.sound_speed)) * ld(clock.over_sound_speed);
    let factor = ld(so.borrow().pitch_factor);
    let eased = (target - factor) * ld(surface.constants.pitch_blend) + factor;
    so.borrow_mut().pitch_factor = stf(eased);
}

/// The volume factor for a far car: `1 / distance`.
fn volume_factor_by_distance(so: &SoundObjectCell) {
    let distance = so.borrow().distance;
    so.borrow_mut().volume_factor = stf(X(1.0) / ld(distance));
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87_mem::{HostCell, view};
    use crate::memory::session::Cell32;
    use crate::sound::asm_layout::{OBJECT_EXTENT, SURFACE_EXTENT, with_object, with_surface};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "revs_input",
                operands: vec![
                    Operand::Sized {
                        len: OBJECT_EXTENT,
                        reads: &[(0x14, In::I)],
                        writes: &[(0x100, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(4, In::F), (0xC, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    let kind = view::<crate::sound::sets_store::KindRecord>(a[1], 0);
                    with_object(a[0], |so| revs_input(so, kind));
                },
            },
            Block {
                name: "speed_input",
                operands: vec![
                    Operand::Sized {
                        len: OBJECT_EXTENT,
                        reads: &[(0x1C, In::F)],
                        writes: &[(0x108, Out::F)],
                    },
                    Operand::Sized {
                        len: SURFACE_EXTENT,
                        reads: &[(0x18, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    with_object(a[0], |so| {
                        with_surface(a[1], |sf| speed_input(so, sf));
                    });
                },
            },
            Block {
                name: "gear_pitch_bits",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0x210, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    let low = HostCell(a[0]).bits();
                    HostCell(a[1] + 0x210).set_bits(gear_pitch_value(low).to_bits());
                },
            },
            Block {
                name: "distance_and_approach",
                operands: vec![
                    Operand::Sized {
                        len: OBJECT_EXTENT,
                        reads: &[(0x2C, In::F), (0x30, In::F), (0x34, In::F), (0x134, In::F)],
                        writes: &[(0x134, Out::F), (0x138, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(8, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    let clock = view::<RaceClock>(a[1], 0);
                    with_object(a[0], |so| distance_and_approach(so, clock));
                },
            },
            Block {
                name: "ease_pitch_factor",
                operands: vec![
                    Operand::Sized {
                        len: OBJECT_EXTENT,
                        reads: &[(0x138, In::F), (0x13C, In::F)],
                        writes: &[(0x13C, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x14, In::F), (0x18, In::F)],
                        writes: &[],
                    },
                    Operand::Sized {
                        len: SURFACE_EXTENT,
                        reads: &[(8, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    let clock = view::<RaceClock>(a[1], 0);
                    with_object(a[0], |so| {
                        with_surface(a[2], |sf| ease_pitch_factor(so, clock, sf));
                    });
                },
            },
            Block {
                name: "volume_factor_by_distance",
                operands: vec![Operand::Sized {
                    len: OBJECT_EXTENT,
                    reads: &[(0x134, In::F)],
                    writes: &[(0x140, Out::F)],
                }],
                ulps: 0,
                plain: |a| {
                    with_object(a[0], volume_factor_by_distance);
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("sound.car_engine_state", &blocks(), 1200, 100);
    }
}
