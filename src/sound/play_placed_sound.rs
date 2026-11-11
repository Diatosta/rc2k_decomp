//! `sound::play_placed_sound` (0x40F11C): plays a sound that has a place around the
//! listener.
//!
//! Nothing for a sound with bit 0x8000.
//! Without bit 0x100000 in `g_soundMode` (no 3D): the volume = request's volume x the
//! sample record's float +0x24 x the request's +0x24, as a whole number held to 0 ..
//! 0x40 and stored over +8; the angle `fpatan` of +0x14 over +0xC is stored at +0x18
//! and 64.0 - |angle| x 40.74 (`flt_512500`, `flt_512504`), as a whole number, at +0x1C:
//! the place between left and right, which goes as 0 when +0xC is zero. With bit
//! 0x2000000 of the mode (Dolby) and an angle below zero its second byte is inverted.
//! With 3D: the volume as a whole number (not stored) and the three floats as they are.
//! Then +4 = +4 x +0x20 as a whole number, and `sound::play_sound` with the sound, the
//! volume, the place (3D: the three floats) and the rate +4.
//! The x87's arithmetic is plain Rust over `X` (`math/x87_float.rs`), rounding as the control
//! word says; the original's instructions are `asm_reference` in the tests.
//! Eight callers (the race's sounds).
//! Refuses, without 3D, a sound past the 0x1CD sample records.

use crate::errors::BadInput;
use crate::scene::x87_mem::{Base, cells, fistp, fld, fstp};
use crate::sound::sets_store::SoundRequest;
use rc2k_core::sound::placed::{self, MODE_3D};

// ---- game data ----

// ---- the x87's arithmetic, with the original's instructions ----

/// `fld a; fmul b; fmul c; fistp`.
fn product3_to_int(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], [0u32]);
    x87_1(&v[..], cells(&mut out));
    out[0]
}

/// `fld a; fmul b; fistp`.
pub fn product_to_int(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], [0u32]);
    x87_2(&v[..], cells(&mut out));
    out[0]
}

/// `fld a; fistp`.
fn to_int(a: u32) -> u32 {
    let (v, mut out) = ([a], [0u32]);
    x87_3(&v[..], cells(&mut out));
    out[0]
}

/// `fld y; fld x; fpatan; fst angle; fabs; fmul per_angle; fsubr centre; fistp`: the
/// angle's bits and the whole number.
fn angle_and_place(y: u32, x: u32, per_angle: u32, centre: u32) -> (u32, u32) {
    let (v, mut out) = ([y, x, per_angle, centre], [0u32; 2]);
    x87_4(&v[..], cells(&mut out));
    (out[0], out[1])
}

/// Plays the request's sound at its place around the listener; `mode` is the sound mode.
/// Outside the 3D modes the place sent on is the pan worked out here.
pub fn play_placed_sound(
    request: &SoundRequest,
    mode: u32,
    audio: &crate::audio::owner::Audio,
) -> Result<(), BadInput> {
    // Converted: `rc2k_core::sound::placed` (`played`, `sample_of`, `volume`,
    // `place`); the products and the angle are the original's x87.
    let sound = request.sound.get();
    if !placed::played(sound) {
        return Ok(());
    }
    let (volume_sent, pan, place_b, place_c);
    if mode & MODE_3D == 0 {
        let number = placed::sample_of(sound)?;
        let record = &audio.sample_records.records[number as usize];
        let factor = record.get().min_distance;
        let volume = placed::volume(product3_to_int(
            request.volume.get(),
            factor,
            request.volume_factor.get(),
        ));
        request.volume.set(volume);
        let clock = &crate::sound::owner::sound().race.clock;
        let (angle, place) = angle_and_place(
            request.place[2].get(),
            request.place[0].get(),
            clock.pan_per_radian.bits(),
            clock.pan_centre.bits(),
        );
        request.angle.set(angle);
        request.pan.set(place);
        let place = placed::place(place, request.place[0].get(), angle, mode);
        // Outside 3D `play_sound` sends no place beyond the pan, so the other two values are 0.
        (volume_sent, pan, place_b, place_c) = (volume, place, 0, 0);
    } else {
        (volume_sent, pan, place_b, place_c) = (
            to_int(request.volume.get()),
            request.place[0].get(),
            request.place[1].get(),
            request.place[2].get(),
        );
    }
    let rate = product_to_int(request.pitch.get(), request.pitch_factor.get());
    request.pitch.set(rate);
    crate::sound::play_sound::play_sound(sound, volume_sent, rate, [pan, place_b, place_c]);
    Ok(())
}

/// In the game.
pub fn play_placed_sound_in_game(
    request: &SoundRequest,
    audio: &crate::audio::owner::Audio,
) -> Result<(), BadInput> {
    play_placed_sound(request, audio.state.sound_mode.get(), audio)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [o + 0] = ([v + 0] * [v + 4]) * [v + 8]`
fn x87_1(v: impl Base, o: impl Base) {
    let t0 = fld(v, 0);
    let t1 = fld(v, 4);
    let t2 = t0 * t1;
    let t3 = fld(v, 8);
    let t4 = t2 * t3;
    fistp(o, 0, t4);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [o + 0] = [v + 0] * [v + 4]`
fn x87_2(v: impl Base, o: impl Base) {
    let t0 = fld(v, 0);
    let t1 = fld(v, 4);
    let t2 = t0 * t1;
    fistp(o, 0, t2);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [o + 0] = [a + 0]`
fn x87_3(a: impl Base, o: impl Base) {
    let t0 = fld(a, 0);
    fistp(o, 0, t0);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
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

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(v, o) "fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]", "fistp dword ptr [{o}]");
    reference!(ref_1(v, o) "fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fistp dword ptr [{o}]");
    reference!(ref_2(a, o) "fld dword ptr [{a}]", "fistp dword ptr [{o}]");
    reference!(ref_3(v, o) "fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", "fpatan", "fst dword ptr [{o}]", "fabs", "fmul dword ptr [{v} + 8]", "fsubr dword ptr [{v} + 12]", "fistp dword ptr [{o} + 4]");

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
                reference: |a| unsafe { ref_0(a[0], a[1]) },
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
                reference: |a| unsafe { ref_1(a[0], a[1]) },
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
                reference: |a| unsafe { ref_2(a[0], a[1]) },
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
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| x87_4(a[0], a[1]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
