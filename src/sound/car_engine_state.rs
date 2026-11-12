//! `sound::car_engine_state` (0x40D8E2): a car's sound object brought up to date for the
//! turn: the revs eased between the physics' readings, the pitch and volume inputs of
//! the engine sounds, the engine's state, the gear-change and back-fire cues, the
//! listener's distance and the Doppler-like factors.
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
//! Returns the object's sound kind number. One caller, `sound::car_sounds`.
//! Refuses (before writing) a kind past the 36 of the table and a revs extrapolation
//! whose quotient does not fit (the original's `idiv` traps).

use crate::errors::{BadInput, Refused};
use crate::math::x87::from_int;
use crate::math::x87_float::{X, int, ld, stf};
use crate::race::owner::Race;
use crate::sound::sets_store::KindRecord;
use crate::sound::store::{RaceClock, SoundObject, SurfaceSounds};
use crate::system::owner::System;
use rc2k_core::sound::engine::{Engine, extrapolation, over_256, readings, state};

pub const KIND_COUNT: u32 = 36;
/// The kind of object `o` (its +4, a number into the kinds' table), or a refusal.
pub fn kind_of(
    o: &crate::sound::store::SoundObject,
    kinds: &crate::sound::sets_store::SoundKinds,
) -> Result<u32, BadInput> {
    let n = o.kind.get();
    if n >= KIND_COUNT {
        return Err(BadInput("a sound kind past the 36 of the table"));
    }
    kinds.record(n)?;
    Ok(n)
}

/// The object's dwords the engine works on, taken as values.
fn load(o: &crate::sound::store::SoundObject) -> Engine {
    Engine {
        new_revs: o.new_revs.get(),
        revs: o.revs.get(),
        throttle: o.throttle.get(),
        gear_event: o.gear_event.get(),
        no_gear: o.no_gear.get(),
        reading_times: [o.reading_times[0].get(), o.reading_times[1].get()],
        reading_revs: [o.reading_revs[0].get(), o.reading_revs[1].get()],
        gear: o.gear.get(),
        last_gear: o.last_gear.get(),
        gear_notice: o.gear_notice.get(),
        countdown: o.countdown.get(),
        wait_flag: o.drop_latched.get(),
        last_direction: o.last_direction.get(),
        state: o.state.get(),
        last_state: o.last_state.get(),
        smooth_throttle: o.smooth_throttle.get(),
        throttle_lead: o.throttle_lead.get(),
        turns_up: o.turns_up.get(),
        direction: o.direction.get(),
        turns_down: o.turns_down.get(),
        drop_sound: o.slots[5].sound.get(),
        drop_start: o.slots[5].start.get(),
        backfire_start: o.slots[6].start.get(),
        backfire_playing: o.slots[6].playing.get(),
        backfire_volume: o.slots[6].volume.get(),
        backfire_wait: o.backfire_wait.get(),
        gear_slot_start: o.gear_slot.start.get(),
    }
}

/// The dwords `now` has changed from `before` put back in the object (the others are left
/// alone, as the original never wrote them).
fn store(o: &crate::sound::store::SoundObject, before: &Engine, now: &Engine) {
    use crate::session::Dword;
    let put = |cell: &Dword, was: u32, is: u32| {
        if was != is {
            cell.set(is);
        }
    };
    put(&o.new_revs, before.new_revs, now.new_revs);
    put(&o.revs, before.revs, now.revs);
    put(&o.gear_event, before.gear_event, now.gear_event);
    put(&o.no_gear, before.no_gear, now.no_gear);
    for n in 0..2 {
        put(
            &o.reading_times[n],
            before.reading_times[n],
            now.reading_times[n],
        );
        put(
            &o.reading_revs[n],
            before.reading_revs[n],
            now.reading_revs[n],
        );
    }
    put(&o.last_gear, before.last_gear, now.last_gear);
    put(&o.gear_notice, before.gear_notice, now.gear_notice);
    put(&o.countdown, before.countdown, now.countdown);
    put(&o.drop_latched, before.wait_flag, now.wait_flag);
    put(&o.last_direction, before.last_direction, now.last_direction);
    put(&o.state, before.state, now.state);
    put(&o.last_state, before.last_state, now.last_state);
    put(
        &o.smooth_throttle,
        before.smooth_throttle,
        now.smooth_throttle,
    );
    put(&o.throttle_lead, before.throttle_lead, now.throttle_lead);
    put(&o.turns_up, before.turns_up, now.turns_up);
    put(&o.direction, before.direction, now.direction);
    put(&o.turns_down, before.turns_down, now.turns_down);
    put(&o.slots[5].sound, before.drop_sound, now.drop_sound);
    put(&o.slots[5].start, before.drop_start, now.drop_start);
    put(&o.slots[6].start, before.backfire_start, now.backfire_start);
    put(
        &o.slots[6].playing,
        before.backfire_playing,
        now.backfire_playing,
    );
    put(
        &o.slots[6].volume,
        before.backfire_volume,
        now.backfire_volume,
    );
    put(&o.backfire_wait, before.backfire_wait, now.backfire_wait);
    put(
        &o.gear_slot.start,
        before.gear_slot_start,
        now.gear_slot_start,
    );
}

/// The object `so` brought up to date; its sound kind's number returned.
pub fn car_engine_state(
    so: &crate::sound::store::SoundObject,
    sound: &crate::sound::owner::Sound,
    race: &Race,
    system: &System,
) -> Result<u32, Refused> {
    // Converted: `rc2k_core::sound::engine` (`extrapolation`, `readings`, `state`,
    // `over_256`); the floats are the original's x87 still.
    let kind = kind_of(so, sound.race.kinds.get())?;
    let record = sound.race.kinds.get().record(kind)?;
    let before = load(so);
    let mut object = before;
    let clock = sound.race.clock.get().last.get();
    // The revs extrapolated, worked out first: its `idiv` may trap.
    let change = extrapolation(&object, clock, || {
        race.shared.replay_file().hold_car_revs.get()
    })
    .map_err(BadInput::from)?;
    readings(&mut object, clock, change);
    store(so, &before, &object);
    revs_input(so, record);
    so.throttle_input.set(f32::from_bits(over_256(
        from_int(so.throttle.get() as i32).to_bits(),
    )));
    speed_input(so, sound.race.surface.get());
    let before = load(so);
    let mut object = before;
    let low = state(&mut object, || record.flags.get(), clock);
    store(so, &before, &object);
    if let Some(low) = low {
        gear_pitch(so, low, system);
    }
    distance_and_approach(so, sound.race.clock.get());
    so.volume_factor.set(0.0);
    ease_pitch_factor(so, sound.race.clock.get(), sound.race.surface.get());
    so.volume_factor.set(1.0);
    if so.distance.bits() as i32 > 0x3F80_0000 {
        volume_factor_by_distance(so);
    }
    Ok(kind)
}

// ---- the float arithmetic, step by step as the x87 rounds it ----

/// The revs input of the engine sounds: `(revs - kind.revs_low) * kind.revs_scale`.
fn revs_input(so: &SoundObject, kind: &KindRecord) {
    let revs = int(so.revs.get() as i32);
    let above_low = revs - ld(kind.revs_low.get());
    so.revs_input
        .set(stf(above_low * ld(kind.revs_scale.get())));
}

/// The speed input: `|speed| * surface.speed_scale`.
fn speed_input(so: &SoundObject, surface: &SurfaceSounds) {
    let speed = ld(so.speed.get()).abs();
    so.speed_input
        .set(stf(speed * ld(surface.speed_scale.get())));
}

/// The gear-change cue's pitch from the clock's low four bits `low`: `low / 60 + 0.75`
/// (the factors as the game's float constants). The original worked in the machine's
/// scratch dword and left 0.75 there.
fn gear_pitch(so: &SoundObject, low: u32, system: &System) {
    system.session.machine.scratch.set(low);
    let pitch = gear_pitch_value(low);
    system.session.machine.scratch.set(GEAR_PITCH_BASE);
    so.gear_pitch.set(pitch);
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
fn distance_and_approach(so: &SoundObject, clock: &RaceClock) {
    let old = ld(so.distance.get());
    let [x, y, z] = [0, 1, 2].map(|n| ld(so.place[n].get()));
    let distance = (x * x + (y * y + z * z)).sqrt();
    so.distance.set(stf(distance));
    so.approach
        .set(stf((old - distance) * ld(clock.turns_a_second.get())));
}

/// The pitch factor eased toward `(approach + sound speed) * (1 / sound speed)`:
/// `((target - factor) * blend) + factor`.
fn ease_pitch_factor(so: &SoundObject, clock: &RaceClock, surface: &SurfaceSounds) {
    let target =
        (ld(so.approach.get()) + ld(clock.sound_speed.get())) * ld(clock.over_sound_speed.get());
    let factor = ld(so.pitch_factor.get());
    let eased = (target - factor) * ld(surface.pitch_blend.get()) + factor;
    so.pitch_factor.set(stf(eased));
}

/// The volume factor for a far car: `1 / distance`.
fn volume_factor_by_distance(so: &SoundObject) {
    so.volume_factor.set(stf(X(1.0) / ld(so.distance.get())));
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::scene::x87_mem::{HostCell, view};
    use crate::session::Cell32;
    use crate::sound::asm_layout::{Laid, with_laid};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "revs_input",
                operands: vec![
                    Operand::Sized {
                        len: SoundObject::EXTENT,
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
                    with_laid::<SoundObject, _>(a[0], |so| revs_input(so, kind));
                },
            },
            Block {
                name: "speed_input",
                operands: vec![
                    Operand::Sized {
                        len: SoundObject::EXTENT,
                        reads: &[(0x1C, In::F)],
                        writes: &[(0x108, Out::F)],
                    },
                    Operand::Sized {
                        len: SurfaceSounds::EXTENT,
                        reads: &[(0x18, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    with_laid::<SoundObject, _>(a[0], |so| {
                        with_laid::<SurfaceSounds, _>(a[1], |sf| speed_input(so, sf));
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
                        len: SoundObject::EXTENT,
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
                    with_laid::<SoundObject, _>(a[0], |so| distance_and_approach(so, clock));
                },
            },
            Block {
                name: "ease_pitch_factor",
                operands: vec![
                    Operand::Sized {
                        len: SoundObject::EXTENT,
                        reads: &[(0x138, In::F), (0x13C, In::F)],
                        writes: &[(0x13C, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x14, In::F), (0x18, In::F)],
                        writes: &[],
                    },
                    Operand::Sized {
                        len: SurfaceSounds::EXTENT,
                        reads: &[(8, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    let clock = view::<RaceClock>(a[1], 0);
                    with_laid::<SoundObject, _>(a[0], |so| {
                        with_laid::<SurfaceSounds, _>(a[2], |sf| ease_pitch_factor(so, clock, sf));
                    });
                },
            },
            Block {
                name: "volume_factor_by_distance",
                operands: vec![Operand::Sized {
                    len: SoundObject::EXTENT,
                    reads: &[(0x134, In::F)],
                    writes: &[(0x140, Out::F)],
                }],
                ulps: 0,
                plain: |a| {
                    with_laid::<SoundObject, _>(a[0], volume_factor_by_distance);
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("sound.car_engine_state", &blocks(), 1200, 100);
    }
}
