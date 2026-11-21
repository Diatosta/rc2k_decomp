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
use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;
use crate::physics::step_families::SoundInputs;
use crate::race::frame_store::FrameWorkStore;
use crate::race::numbers_store::Numbers;
use crate::scene::matrix::Matrix;
use crate::scene::rotate_point::rotate_point_by;
use crate::sound::store::SoundObjectCell;
use rc2k_core::physics::sound_places::{FIRST, PLAY, SECOND, Slot};

/// Whether car `car` (a number) is listed; refused past 7.
pub fn listed(frame_work: &FrameWorkStore, car: u32) -> Result<bool, Refused> {
    if car > 7 {
        return Err(BadInput("a car number past 7").into());
    }
    Ok(frame_work.borrow().listed_car_sounds[car as usize] != u32::MAX)
}

/// The current car's sound object.
pub fn object<'a>(c: &SoundInputs<'a>) -> &'a SoundObjectCell {
    crate::sound::store::current_object(c.owner, c.work)
}

/// The place (with `offset` added to the car's, when given), turned and scaled.
pub fn place(
    numbers: &Numbers,
    view_turn: &Matrix,
    heard_turn: &crate::race::heard_turn::HeardTurn,
    car: &Car,
    listener: &Car,
    offset: Option<[u32; 3]>,
) -> [f32; 3] {
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
    let out = rotate_point_by(&heard_turn.turn(view_turn), [part(0), part(1), part(2)]);
    let k = numbers.one_over_65536.get();
    out.map(|e| rounded_value(ext_times(e, k)))
}

/// The converted place (`rc2k_core::physics::sound_places::place`).
fn place_plain(
    numbers: &Numbers,
    view_turn: &Matrix,
    heard_turn: &crate::race::heard_turn::HeardTurn,
    car: &Car,
    listener: &Car,
    offset: Option<[u32; 3]>,
) -> [f32; 3] {
    let whole = |x: &[PlainDword; 3]| [0, 1, 2].map(|k| x[k].get() as i32);
    let turn = heard_turn.turn(view_turn);
    rc2k_core::physics::sound_places::place(
        whole(&car.position),
        whole(&listener.camera_position),
        offset.map(|o| o.map(|v| v as i32)),
        &std::array::from_fn(|n| turn[n]),
        numbers.one_over_65536.get(),
    )
}

/// The converted steps' end: the sound and the band in the object's `slot`, the place
/// (with `offset`), the object told to play. Any slot but the two one-off ones is refused.
// Core: `rc2k_core::physics::sound_places`.
pub(super) fn cue_plain(
    c: &SoundInputs<'_>,
    o: &SoundObjectCell,
    slot: Slot,
    sound: u32,
    band: u32,
    offset: Option<[u32; 3]>,
) -> Result<(), BadInput> {
    if slot != FIRST && slot != SECOND {
        return Err(BadInput("a sound object's slot not worked out"));
    }
    let mut guard = o.borrow_mut();
    let object = &mut *guard;
    let (sound_cell, band_cell) = if slot == FIRST {
        (&mut object.hit_sound, &mut object.hit_volume)
    } else {
        (&mut object.cue_sound, &mut object.cue_volume)
    };
    *sound_cell = sound;
    *band_cell = band;
    let place = place_plain(
        c.numbers,
        &c.view_turn,
        c.heard_turn,
        c.car,
        c.listener,
        offset,
    );
    if slot == FIRST {
        object.hit_place = place;
    } else {
        object.cue_place = place;
    }
    // `PLAYING`: the play switch, the cell the revs' new reading flag shares.
    object.new_revs = PLAY;
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
