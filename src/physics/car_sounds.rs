//! The car's sound objects placed (IDA 2026-10-18): four functions under the sound
//! drive that, when the current car is listed (`dword_641680[g_currentCar]` not -1), take
//! a band (`physics::sound_band_1` .. `_4`), set two words of the car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`) and its place: the car's place (`g_car`
//! +0x134 .. +0x13C, with an offset for one) less the listener's (+0x6A4 ..), `fild` /
//! `fiadd` / `fisub` (exact), turned by `g_viewTurnMatrix`, each part x `flt_5C5DA0`
//! (1/65536); then +0x10 1. All in `pusha` / `popa`.
//!
//! This file: the shared steps and the test of the four. `place` is the exact x87 one, for
//! `place_sound_e1`'s exact version (its Rust callers).

use crate::errors::{BadInput, Refused};
use crate::math::x87_run::ext_times;
use crate::physics::car_store::CAR;
use crate::race::frame_store::FRAME_WORK;
use crate::race::matrices_store::MATRICES;
use crate::race::numbers_store::NUMBERS;
use crate::race::work_store::RACE_WORK;
use crate::scene::matrix3::rounded;
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::{read_floats, write_floats};
use crate::scene::x87_mem::{fild, fstp80};
use crate::session::Dword;
use rc2k_core::physics::sound_places::{PLAY, PLAYING, Slot};

pub fn put(a: u32, v: u32) {
    crate::pointers::write_or_end::<u32>(a, v, "physics::car_sounds's operand")
}

/// Whether car `car` (a number) is listed; refused past 7.
pub fn listed(car: u32) -> Result<bool, Refused> {
    if car > 7 {
        return Err(BadInput("a car number past 7").into());
    }
    Ok(FRAME_WORK.listed_car_sounds[car as usize].get() != u32::MAX)
}

/// The current car's sound object; refused when null.
pub fn object() -> Result<u32, Refused> {
    let o =
        crate::sound::store::RACE_SOUNDS.table[(RACE_WORK.current_car.get() & 7) as usize].get();
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    Ok(o)
}

/// The place (with `offset` added to the car's, when given), turned and scaled, into
/// the object at +`at` .. +`at` + 8.
pub fn place(o: u32, at: u32, offset: Option<[u32; 3]>) {
    // The original's own steps (`fild` the offset, `fiadd` the car's, `fisub` the
    // listener's; without an offset `fild` the car's, `fisub`): each rounds to the
    // control word's precision, so a big difference keeps fewer bits at 24.
    let part = |k: usize| {
        let car = CAR.position[k].get();
        let listener = CAR.camera_position[k].get();
        match offset {
            Some(o) => offset_part(o[k], car, listener),
            None => car_part(car, listener),
        }
    };
    let out = rotate_point(
        MATRICES.view_turn_matrix[0].addr() as u32,
        [part(0), part(1), part(2)],
    );
    let k = NUMBERS.one_over_65536.bits();
    for (n, e) in out.iter().enumerate() {
        put(o + at + 4 * n as u32, rounded(ext_times(*e, k)));
    }
}

/// The converted place (`rc2k_core::physics::sound_places::place`) into the object at
/// +`at` .. +`at` + 8.
pub fn place_plain(o: u32, at: u32, offset: Option<[u32; 3]>) {
    let whole = |x: &[Dword; 3]| [0, 1, 2].map(|k| x[k].get() as i32);
    let p = rc2k_core::physics::sound_places::place(
        whole(&CAR.position),
        whole(&CAR.camera_position),
        offset.map(|o| o.map(|v| v as i32)),
        &read_floats(MATRICES.view_turn_matrix[0].addr() as u32),
        f32::from_bits(NUMBERS.one_over_65536.bits()),
    );
    write_floats(o + at, &p);
}

/// The converted steps' end: the sound and the band in the object's `slot`, the place
/// (with `offset`), the object told to play.
// Converted: `rc2k_core::physics::sound_places`.
pub fn cue_plain(o: u32, slot: Slot, sound: u32, band: u32, offset: Option<[u32; 3]>) {
    put(o + slot.sound, sound);
    put(o + slot.band, band);
    place_plain(o, slot.place, offset);
    put(o + PLAYING, PLAY);
}

/// `fild car; fisub listener`, as an 80-bit value.
fn car_part(car: u32, listener: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    let v = [car, listener];
    x87_1(
        crate::pointers::host_number(v.as_ptr()),
        crate::pointers::host_number(out.as_mut_ptr()),
    );
    out
}

/// `fild offset; fiadd car; fisub listener`, as an 80-bit value.
fn offset_part(offset: u32, car: u32, listener: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    let v = [offset, car, listener];
    x87_2(
        crate::pointers::host_number(v.as_ptr()),
        crate::pointers::host_number(out.as_mut_ptr()),
    );
    out
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `ext [o + 0] = int [v + 0] - int [v + 4]`
fn x87_1(v: usize, o: usize) {
    let t0 = fild(v, 0);
    let t1 = fild(v, 4);
    let t2 = t0 - t1;
    fstp80(o, 0, t2);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `ext [o + 0] = (int [v + 0] + int [v + 4]) - int [v + 8]`
fn x87_2(v: usize, o: usize) {
    let t0 = fild(v, 0);
    let t1 = fild(v, 4);
    let t2 = t0 + t1;
    let t3 = fild(v, 8);
    let t4 = t2 - t3;
    fstp80(o, 0, t4);
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(v, o) "fild dword ptr [{v}]", "fisub dword ptr [{v} + 4]", "fstp tbyte ptr [{o}]");
    reference!(ref_1(v, o) "fild dword ptr [{v}]", "fiadd dword ptr [{v} + 4]", "fisub dword ptr [{v} + 8]", "fstp tbyte ptr [{o}]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "car_sounds line 110",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (4, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::T)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| x87_1(a[0], a[1]),
            },
            Block {
                name: "car_sounds line 119",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (4, In::I), (8, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::T)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
