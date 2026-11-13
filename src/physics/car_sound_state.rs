//! `physics::car_sound_state` (0x45CACE): the work car's engine sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`) told the car's state: +0x10 1; +0x14 the revs
//! (+0x1E4); +0x54, +0x68, +0x7C, +0x90 `g_car` +0x930; the share +0x28C =
//! +0x288 / `sound_speed_scale` and +0x290 1.0 (with +0x254 set, those and +0x294 0), into the
//! object's four voices (+0x4C .., x256 at +0x50 .., +0x294 at +0x44 ..); +0x18 +0x638
//! held to 0 ..= 0x100; +0xE8 the gear; +0x1C +0x1C0; +0xAC +0x248 & +0x24C; +0x0C
//! +0x6F0 ^ 1; the car's place less the listener's (`g_viewBlocks[0]`'s with
//! `players` 1, else the work car's own +0x6A4 ..) turned by `g_viewTurnMatrix`, x
//! 1/65536 into +0x2C ..; the car's +0xD4 .. turned into +0x38 ... All in `pusha` /
//! `popa`.
//!
//! the exact x87 one (the runs over the object, plain Rust over `X`) stays
//! for `physics::engine_sound`.
//!
//! Caller: `physics::engine_sound`. Refuses (before anything) a null sound object, or a
//! null view block when it is read.

use crate::effects::cell_x87::CellX87;
use crate::errors::{BadInput, Refused};
use crate::math::x87::{ext_times, ints_diff_80};
use crate::math::x87_float::X;
use crate::physics::car_store::Car;
use crate::race::matrices_store::SceneMatrices;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::scene::matrix3::{extended, rounded_value};
use crate::scene::rotate_point::rotate_point;
use crate::session::PlainDword;

/// 0x45CB02 .. 0x45CB88 (the shares into the voices), the integer steps between in Rust.
fn shares(numbers: &Numbers, shared: &RaceShared, so: &crate::sound::store::SoundObject) {
    x87_1(shared);
    if shared.car().all_wheels_off.get() != 0 {
        shared.car().sound_slip_share.set(0.0);
        shared.car().sound_level.set(0.0);
        shared.car().sound_speed.set(0.0);
    }
    x87_2(numbers, shared.car(), so);
}

/// The car the sound is heard from: view 0's block with one player (none if it has no
/// block), else the work car.
fn listener(shared: &RaceShared) -> Option<&crate::physics::car_store::Car> {
    if shared.view.get().players.get() == 1 {
        shared
            .car_blocks()
            .of(shared.view.get().view_blocks[0].get())
    } else {
        Some(shared.car())
    }
}

/// The exact x87 version (for `physics::engine_sound`).
pub fn car_sound_state(
    numbers: &Numbers,
    matrices: &SceneMatrices,
    shared: &RaceShared,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let Some(l) = listener(shared) else {
        return Err(BadInput("a null view block").into());
    };
    let so = crate::sound::store::current_object(sound, shared.work.get());
    so.new_revs.set(1);
    so.revs.set(shared.car().engine_sound_revs.get());
    let w = shared.car().wheel_surface_a[4].get();
    for voice in &so.voices {
        voice.surface.set(w);
    }
    shares(numbers, shared, so);
    let n = shared.car().throttle.get() as i32;
    so.throttle.set(n.clamp(0, 0x100) as u32);
    so.gear.set(shared.car().gear.get() as u32);
    so.speed.set(shared.car().mean_drive_speed.get());
    so.flag_ac
        .set(shared.car().front_wheels_off.get() & shared.car().rear_wheels_off.get());
    so.view_flag
        .set(shared.car().camera_mode_is_2_or_3.get() ^ 1);
    let d = |c: &PlainDword, other: &PlainDword| ints_diff_80(c.get() as i32, other.get() as i32);
    let place = [
        d(&shared.car().position[0], &l.camera_position[0]),
        d(&shared.car().position[1], &l.camera_position[1]),
        d(&shared.car().position[2], &l.camera_position[2]),
    ];
    let m = crate::race::heard_turn::turn(matrices);
    let out = rotate_point(m, place);
    let k = numbers.one_over_65536.get();
    for (i, e) in out.iter().enumerate() {
        so.place[i].set(rounded_value(ext_times(*e, k)));
    }
    let out = rotate_point(
        m,
        [
            extended(shared.car().crash_matrix[8].get()),
            extended(shared.car().crash_matrix[9].get()),
            extended(shared.car().crash_matrix[10].get()),
        ],
    );
    for (i, e) in out.iter().enumerate() {
        so.motion[i].set(rounded_value(*e));
    }
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[sound_slip_share] = [slip_speed] / [sound_speed_scale]`
/// `[sound_level] = 1`
fn x87_1(shared: &RaceShared) {
    let t0 = shared.car().slip_speed.ld();
    let t1 = shared.physics_tables.get().sound_speed_scale.ld();
    let t2 = t0 / t1;
    shared.car().sound_slip_share.st(t2);
    let t3 = X(1.0);
    shared.car().sound_level.st(t3);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[o + 0x4C] = [sound_level]`
/// `[o + 0x60] = [sound_level]`
/// `[o + 0x74] = [sound_level]`
/// `[o + 0x88] = [sound_level]`
/// `[o + 0x50] = [sound_slip_share] * [k256]`
/// `[o + 0x64] = [sound_slip_share] * [k256]`
/// `[o + 0x78] = [sound_slip_share] * [k256]`
/// `[o + 0x8C] = [sound_slip_share] * [k256]`
/// `[o + 0x44] = [sound_speed]`
/// `[o + 0x58] = [sound_speed]`
/// `[o + 0x6C] = [sound_speed]`
/// `[o + 0x80] = [sound_speed]`
fn x87_2(numbers: &Numbers, car: &Car, o: &crate::sound::store::SoundObject) {
    let t0 = car.sound_level.ld();
    o.voices[0].share_b.st(t0);
    o.voices[1].share_b.st(t0);
    o.voices[2].share_b.st(t0);
    o.voices[3].share_b.st(t0);
    let t1 = car.sound_slip_share.ld();
    let t2 = numbers.k256.ld();
    let t3 = t1 * t2;
    o.voices[0].share_c.st(t3);
    o.voices[1].share_c.st(t3);
    o.voices[2].share_c.st(t3);
    o.voices[3].share_c.st(t3);
    let t4 = car.sound_speed.ld();
    o.voices[0].share_a.st(t4);
    o.voices[1].share_a.st(t4);
    o.voices[2].share_a.st(t4);
    o.voices[3].share_a.st(t4);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::sound::asm_layout::{Laid, with_laid};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "car_sound_state line 46",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x213EE0, In::F), (0x31A260, In::F)],
                    writes: &[(0x31A264, Out::F), (0x31A268, Out::F)],
                }],
                ulps: 0,
                plain: |_| x87_1(&race().shared),
            },
            Block {
                name: "car_sound_state line 64",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x31A264, In::F),
                            (0x31A268, In::F),
                            (0x31A26C, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Sized {
                        len: crate::sound::store::SoundObject::EXTENT,
                        reads: &[],
                        writes: &[
                            (0x44, Out::F),
                            (0x4C, Out::F),
                            (0x50, Out::F),
                            (0x58, Out::F),
                            (0x60, Out::F),
                            (0x64, Out::F),
                            (0x6C, Out::F),
                            (0x74, Out::F),
                            (0x78, Out::F),
                            (0x80, Out::F),
                            (0x88, Out::F),
                            (0x8C, Out::F),
                        ],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    with_laid::<crate::sound::store::SoundObject, _>(a[1], |so| {
                        x87_2(race().course.numbers.get(), race().shared.car(), so);
                    });
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.car_sound_state", &blocks(), 1200, 100);
    }
}
