//! `sound::play_placed_sound`: plays a sound that has a place around the listener.
//!
//! Nothing for a sound with bit 0x8000. Without bit 0x100000 in the sound mode (no 3D): the volume
//! is the request's volume x the sample record's float x the request's volume factor, as a whole
//! number held to 0 .. 0x40 and stored over the request's volume; the angle (the arctangent of the
//! place's third float over its first) is stored in the request, and 64.0 - |angle| x 40.74
//! (`RaceClock.pan_centre`, `RaceClock.pan_per_radian`), as a whole number, is the place between
//! left and right (0 when the first float is zero). With bit 0x2000000 of the mode (Dolby) and an
//! angle below zero its second byte is inverted. With 3D: the volume as a whole number (not stored)
//! and the three floats as they are. Then the pitch is the pitch x the request's pitch factor as a
//! whole number, and `sound::play_sound` plays the sound with the volume, the place (3D: the three
//! floats) and that rate. The x87 arithmetic is plain Rust over `X` (`math/x87_float.rs`), rounding
//! as the control word says; it is checked in the tests' `reference_vectors`. Refuses, without 3D,
//! a sound past the 0x1CD sample records.

use crate::errors::BadInput;
use crate::math::x87_mem::{Base, cells, fistp, fld, fstp};
use crate::sound::sets_store::SoundRequest;
use rc2k_core::sound::placed::{self, MODE_3D};

// ---- game data ----

// ---- the x87-precision arithmetic ----

/// `a * b * c` as a whole number.
fn product3_to_int(a: u32, b: u32, c: f32) -> u32 {
    let (v, mut out) = ([a, b, c.to_bits()], [0u32]);
    x87_1(&v[..], cells(&mut out));
    out[0]
}

/// `a * b` as a whole number.
fn product_to_int(a: u32, b: f32) -> u32 {
    let (v, mut out) = ([a, b.to_bits()], [0u32]);
    x87_2(&v[..], cells(&mut out));
    out[0]
}

/// `a` as a whole number.
fn to_int(a: u32) -> u32 {
    let (v, mut out) = ([a], [0u32]);
    x87_3(&v[..], cells(&mut out));
    out[0]
}

/// The angle `atan2(y, x)` (its bits) and `centre - |angle| * per_angle` as a whole number.
fn angle_and_place(y: f32, x: f32, per_angle: f32, centre: f32) -> (u32, u32) {
    let (v, mut out) = (
        [
            y.to_bits(),
            x.to_bits(),
            per_angle.to_bits(),
            centre.to_bits(),
        ],
        [0u32; 2],
    );
    x87_4(&v[..], cells(&mut out));
    (out[0], out[1])
}

/// Plays the request's sound at its place around the listener, in the audio's sound mode.
/// Outside the 3D modes the place sent on is the pan worked out here.
pub(super) fn play_placed_sound(
    request: &SoundRequest,
    owner: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), BadInput> {
    let mode = audio.state.sound_mode.get();
    // The logic is `rc2k_core::sound::placed` (`played`, `sample_of`, `volume`, `place`); the
    // products and the angle are x87-precision.
    let sound = request.sound.get();
    if !placed::played(sound) {
        return Ok(());
    }
    let (volume_sent, pan, place_b, place_c);
    if mode & MODE_3D == 0 {
        let number = placed::sample_of(sound)?;
        let factor = crate::audio::sample_records::SAMPLE_CONSTANTS[number as usize].min_distance;
        let volume = placed::volume(product3_to_int(
            request.volume.get(),
            factor,
            request.volume_factor.get(),
        ));
        request.volume.set(volume);
        let clock = owner.race.clock.get();
        let (angle, place) = angle_and_place(
            request.place[2].get(),
            request.place[0].get(),
            clock.pan_per_radian.get(),
            clock.pan_centre.get(),
        );
        request.angle.set(angle);
        request.pan.set(place);
        let place = placed::place(place, request.place[0].get(), angle as i32, mode);
        // Outside 3D `play_sound` sends no place beyond the pan, so the other two values are 0.
        (volume_sent, pan, place_b, place_c) = (volume, place, 0, 0);
    } else {
        (volume_sent, pan, place_b, place_c) = (
            to_int(request.volume.get()),
            request.place[0].bits(),
            request.place[1].bits(),
            request.place[2].bits(),
        );
    }
    let rate = product_to_int(request.pitch.get(), request.pitch_factor.get());
    request.pitch.set(rate);
    crate::sound::play_sound::play_sound_in(
        audio,
        sound,
        volume_sent,
        rate,
        [pan, place_b, place_c],
    );
    Ok(())
}

// ---- the x87-precision arithmetic ----

/// An x87-precision run in plain Rust over `X`, written as the expressions it computes
/// (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [o + 0] = ([v + 0] * [v + 4]) * [v + 8]`
fn x87_1(v: impl Base, o: impl Base) {
    let t0 = fld(v, 0);
    let t1 = fld(v, 4);
    let t2 = t0 * t1;
    let t3 = fld(v, 8);
    let t4 = t2 * t3;
    fistp(o, 0, t4);
}

/// An x87-precision run in plain Rust over `X`, written as the expressions it computes
/// (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [o + 0] = [v + 0] * [v + 4]`
fn x87_2(v: impl Base, o: impl Base) {
    let t0 = fld(v, 0);
    let t1 = fld(v, 4);
    let t2 = t0 * t1;
    fistp(o, 0, t2);
}

/// An x87-precision run in plain Rust over `X`, written as the expressions it computes
/// (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [o + 0] = [a + 0]`
fn x87_3(a: impl Base, o: impl Base) {
    let t0 = fld(a, 0);
    fistp(o, 0, t0);
}

/// An x87-precision run in plain Rust over `X`, written as the expressions it computes
/// (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[o + 0] = atan2([v + 0], [v + 4])`
/// `int [o + 4] = [v + 0xC] - (|atan2([v + 0], [v + 4])| * [v + 8])`
fn x87_4(v: impl Base, o: impl Base) {
    let t0 = fld(v, 0);
    let t1 = fld(v, 4);
    let t2 = t0.atan2(t1);
    fstp(o, 0, t2);
    let t3 = t2.abs();
    let t4 = fld(v, 8);
    let t5 = t3 * t4;
    let t6 = fld(v, 0xC);
    let t7 = t6 - t5;
    fistp(o, 4, t7);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "play_placed_sound line 56",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F), (4, In::F), (8, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::I)],
                    },
                ],
                ulps: 0,
                plain: |a| x87_1(a[0], a[1]),
            },
            Block {
                name: "play_placed_sound line 64",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F), (4, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::I)],
                    },
                ],
                ulps: 0,
                plain: |a| x87_2(a[0], a[1]),
            },
            Block {
                name: "play_placed_sound line 72",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::I)],
                    },
                ],
                ulps: 0,
                plain: |a| x87_3(a[0], a[1]),
            },
            Block {
                name: "play_placed_sound line 80",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F), (4, In::F), (8, In::F), (0xC, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F), (4, Out::I)],
                    },
                ],
                ulps: 1,
                plain: |a| x87_4(a[0], a[1]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("sound.play_placed_sound", &blocks(), 1200, 100);
    }
}
