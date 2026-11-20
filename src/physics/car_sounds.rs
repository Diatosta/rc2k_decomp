//! The car's sound objects placed: four functions under the sound drive that, when the current car
//! is listed (`listed_car_sounds` entry not -1), take a band (`physics::sound_bands`), set
//! two words of the car's sound object and its place: the car's place (with an offset for one) less
//! the listener's, turned by the heard turn (`race::heard_turn::turn`), each part times
//! `one_over_65536`.
//!
//! This file holds the shared steps and the test of the four. `place` is the exact x87 version,
//! used by `place_sound_e1`'s exact version (its Rust callers).

use crate::errors::{BadInput, Refused};
use crate::math::matrix3::rounded_value;
use crate::math::x87::ext_times;
use crate::math::x87_float::{f80, int};
use crate::memory::session::{PlainDword, PlainFloat};
use crate::physics::car_store::Car;
use crate::physics::step_families::SoundInputs;
use crate::race::frame_store::FrameWork;
use crate::race::matrices_store::SceneMatrices;
use crate::race::numbers_store::Numbers;
use crate::scene::rotate_point::rotate_point;
use crate::sound::store::SoundObject;
use rc2k_core::physics::sound_places::{FIRST, PLAY, SECOND, Slot};

/// Whether car `car` (a number) is listed; refused past 7.
pub fn listed(frame_work: &FrameWork, car: u32) -> Result<bool, Refused> {
    if car > 7 {
        return Err(BadInput("a car number past 7").into());
    }
    Ok(frame_work.listed_car_sounds[car as usize].get() != u32::MAX)
}

/// The current car's sound object.
pub fn object<'a>(c: &SoundInputs<'a>) -> &'a SoundObject {
    crate::sound::store::current_object(c.owner, c.work)
}

/// The place (with `offset` added to the car's, when given), turned and scaled, into
/// the three floats `into`.
pub fn place(
    numbers: &Numbers,
    matrices: &SceneMatrices,
    heard_turn: &crate::race::heard_turn::HeardTurn,
    car: &Car,
    listener: &Car,
    into: &[PlainFloat; 3],
    offset: Option<[u32; 3]>,
) {
    // The steps are the x87's (load the offset, add the car's, subtract the listener's; without
    // an offset load the car's, subtract): each rounds to the control word's precision, so a
    // big difference keeps fewer bits at 24.
    let part = |k: usize| {
        let here = car.position[k].get();
        let listener = listener.camera_position[k].get();
        match offset {
            Some(o) => offset_part(o[k], here, listener),
            None => car_part(here, listener),
        }
    };
    let out = rotate_point(heard_turn.turn(matrices), [part(0), part(1), part(2)]);
    let k = numbers.one_over_65536.get();
    for (cell, e) in into.iter().zip(&out) {
        cell.set(rounded_value(ext_times(*e, k)));
    }
}

/// The converted place (`rc2k_core::physics::sound_places::place`) into the three floats
/// `into`.
fn place_plain(
    numbers: &Numbers,
    matrices: &SceneMatrices,
    heard_turn: &crate::race::heard_turn::HeardTurn,
    car: &Car,
    listener: &Car,
    into: &[PlainFloat; 3],
    offset: Option<[u32; 3]>,
) {
    let whole = |x: &[PlainDword; 3]| [0, 1, 2].map(|k| x[k].get() as i32);
    let p = rc2k_core::physics::sound_places::place(
        whole(&car.position),
        whole(&listener.camera_position),
        offset.map(|o| o.map(|v| v as i32)),
        &std::array::from_fn(|n| heard_turn.turn(matrices)[n].get()),
        numbers.one_over_65536.get(),
    );
    for (cell, x) in into.iter().zip(&p) {
        cell.set(*x);
    }
}

/// The object's one-off sound slot that `slot` (the core's offsets) names: the sound, its
/// band (the volume) and its place. Any other slot is refused.
fn one_off(
    o: &SoundObject,
    slot: Slot,
) -> Result<(&PlainDword, &PlainDword, &[PlainFloat; 3]), BadInput> {
    match slot {
        FIRST => Ok((&o.hit_sound, &o.hit_volume, &o.hit_place)),
        SECOND => Ok((&o.cue_sound, &o.cue_volume, &o.cue_place)),
        _ => Err(BadInput("a sound object's slot not worked out")),
    }
}

/// The converted steps' end: the sound and the band in the object's `slot`, the place
/// (with `offset`), the object told to play.
// Core: `rc2k_core::physics::sound_places`.
pub(super) fn cue_plain(
    c: &SoundInputs<'_>,
    o: &SoundObject,
    slot: Slot,
    sound: u32,
    band: u32,
    offset: Option<[u32; 3]>,
) -> Result<(), BadInput> {
    let (sound_cell, band_cell, place_cells) = one_off(o, slot)?;
    sound_cell.set(sound);
    band_cell.set(band);
    place_plain(
        c.numbers,
        c.matrices,
        c.heard_turn,
        c.car,
        c.listener,
        place_cells,
        offset,
    );
    // `PLAYING`: the play switch, the cell the revs' new reading flag shares.
    o.new_revs.set(PLAY);
    Ok(())
}

/// The car's place less the listener's, as an 80-bit value.
fn car_part(car: u32, listener: u32) -> [u8; 10] {
    x87_1([car, listener])
}

/// The offset plus the car's place less the listener's, as an 80-bit value.
fn offset_part(offset: u32, car: u32, listener: u32) -> [u8; 10] {
    x87_2([offset, car, listener])
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `ext [o + 0] = int [v + 0] - int [v + 4]`
fn x87_1(v: [u32; 2]) -> [u8; 10] {
    let t0 = int(v[0] as i32);
    let t1 = int(v[1] as i32);
    let t2 = t0 - t1;
    f80(t2)
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `ext [o + 0] = (int [v + 0] + int [v + 4]) - int [v + 8]`
fn x87_2(v: [u32; 3]) -> [u8; 10] {
    let t0 = int(v[0] as i32);
    let t1 = int(v[1] as i32);
    let t2 = t0 + t1;
    let t3 = int(v[2] as i32);
    let t4 = t2 - t3;
    f80(t4)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87_mem::{Base, view};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

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
                plain: |a| a[1].set_tbyte(0, x87_1(*view(a[0], 0))),
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
                plain: |a| a[1].set_tbyte(0, x87_2(*view(a[0], 0))),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.car_sounds", &blocks(), 1200, 100);
    }
}
