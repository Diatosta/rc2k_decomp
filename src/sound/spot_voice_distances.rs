//! `sound::spot_voice_distances`: each voice heard this turn: its distance, the distance's change
//! and the pitch and volume factors, as `sound::car_engine_state` does for a car.
//!
//! For each voice heard (+4 1): +0x40 the root of +8 .. +0x10 squared, +0x44 its change x the turns
//! a second (`RaceClock.turns_a_second`); +0x4C 0 and +0x48 1.0, then unless the voice is new
//! (+0x20 1) +0x48 eased from 1.0 toward (+0x44 + 337.52) x 0.00296; +0x4C 1.0, or 1 / the distance
//! when that is over 1 (compared as integers); +0x20 cleared. See `sound/spot_voices.rs`.

use crate::math::x87_float::X;
use crate::math::x87_float::{ld, stf};
use crate::sound::spot_voices::{VOICE_COUNT, voice};

/// The distance, as the float's bits compared as whole numbers: over 1.0 (a negative or a
/// not-a-number distance counts as the integer compare has it).
fn over_one(distance: f32) -> bool {
    distance.to_bits() as i32 > 0x3F80_0000
}

pub(super) fn spot_voice_distances(sound: &crate::sound::owner::Sound) {
    for n in 0..VOICE_COUNT {
        let v = voice(n, sound);
        if v.heard.get() != 1 {
            continue;
        }
        let (distance, change) = distance_and_change(
            [v.place[0].get(), v.place[1].get(), v.place[2].get()],
            v.distance.get(),
            sound.race.clock.get().turns_a_second.get(),
        );
        v.distance.set(distance);
        v.distance_change.set(change);
        v.volume_factor.set(0.0);
        v.pitch_factor.set(1.0);
        if v.new.get() != 1 {
            v.pitch_factor.set(eased_pitch_factor(
                v.distance_change.get(),
                v.pitch_factor.get(),
                sound.race.clock.get(),
                sound.race.surface.get(),
            ));
        }
        v.volume_factor.set(1.0);
        if over_one(v.distance.get()) {
            v.volume_factor.set(inverse(v.distance.get()));
        }
        v.new.set(0);
    }
}

// ---- the x87-precision arithmetic ----

/// An x87-precision run in plain Rust over `X`: the new distance and its change a
/// second:
/// `distance = sqrt(((x * x) + ((y * y) + (z * z))))`
/// `change = (distance - old_distance) * turns_a_second`
fn distance_and_change(place: [f32; 3], old_distance: f32, turns_a_second: f32) -> (f32, f32) {
    let t0 = ld(old_distance);
    let t1 = ld(place[0]);
    let t2 = ld(place[0]);
    let t3 = t1 * t2;
    let t4 = ld(place[1]);
    let t5 = ld(place[1]);
    let t6 = t4 * t5;
    let t7 = ld(place[2]);
    let t8 = ld(place[2]);
    let t9 = t7 * t8;
    let t10 = t6 + t9;
    let t11 = t3 + t10;
    let t12 = t11.sqrt();
    let t13 = t0 - t12;
    let t14 = ld(turns_a_second);
    let t15 = t13 * t14;
    (stf(t12), stf(t15))
}

/// An x87-precision run in plain Rust over `X`: the pitch factor eased toward the
/// change's pitch:
/// `factor = (((change + sound_speed) * over_sound_speed) - factor) * pitch_blend + factor`
fn eased_pitch_factor(
    change: f32,
    factor: f32,
    ck: &crate::sound::store::RaceClock,
    sf: &crate::sound::store::SurfaceSounds,
) -> f32 {
    let t0 = ld(change);
    let t1 = ld(ck.sound_speed.get());
    let t2 = t0 + t1;
    let t3 = ld(ck.over_sound_speed.get());
    let t4 = t2 * t3;
    let t5 = ld(factor);
    let t6 = t4 - t5;
    let t7 = ld(sf.pitch_blend.get());
    let t8 = t6 * t7;
    let t9 = ld(factor);
    let t10 = t8 + t9;
    stf(t10)
}

/// An x87-precision run in plain Rust over `X`: `1 / distance`.
fn inverse(distance: f32) -> f32 {
    let t0 = X(1.0);
    let t1 = ld(distance);
    let t2 = t0 / t1;
    stf(t2)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87_mem::{HostCell, view};
    use crate::memory::session::Cell32;
    use crate::sound::asm_layout::{Laid, with_laid};
    use crate::sound::store::SurfaceSounds;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    /// The float at byte `offset` of the test buffer at `at`.
    fn float_at(at: usize, offset: usize) -> f32 {
        f32::from_bits(HostCell(at + offset).bits())
    }

    /// Stores `value` at byte `offset` of the test buffer at `at`.
    fn put_float(at: usize, offset: usize, value: f32) {
        HostCell(at + offset).set_bits(value.to_bits());
    }

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "spot_voice_distances line 27",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(8, In::F), (0xC, In::F), (0x10, In::F), (0x40, In::F)],
                        writes: &[(0x40, Out::F), (0x44, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(8, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    let clock = view::<crate::sound::store::RaceClock>(a[1], 0);
                    let place = [float_at(a[0], 8), float_at(a[0], 0xC), float_at(a[0], 0x10)];
                    let (distance, change) = distance_and_change(
                        place,
                        float_at(a[0], 0x40),
                        clock.turns_a_second.get(),
                    );
                    put_float(a[0], 0x40, distance);
                    put_float(a[0], 0x44, change);
                },
            },
            Block {
                name: "spot_voice_distances line 47",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x44, In::F), (0x48, In::F)],
                        writes: &[(0x48, Out::F)],
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
                    let clock = view::<crate::sound::store::RaceClock>(a[1], 0);
                    with_laid::<SurfaceSounds, _>(a[2], |sf| {
                        let factor = eased_pitch_factor(
                            float_at(a[0], 0x44),
                            float_at(a[0], 0x48),
                            clock,
                            sf,
                        );
                        put_float(a[0], 0x48, factor);
                    });
                },
            },
            Block {
                name: "spot_voice_distances line 60",
                operands: vec![Operand::Buffer {
                    reads: &[(0x40, In::F)],
                    writes: &[(0x4C, Out::F)],
                }],
                ulps: 0,
                plain: |a| put_float(a[0], 0x4C, inverse(float_at(a[0], 0x40))),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("sound.spot_voice_distances", &blocks(), 1200, 100);
    }
}
