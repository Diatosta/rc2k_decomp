//! `physics::bump_cars` (0x45F7B1): the work car and another (`other_car_block`, its number
//! `other_car_number`) bumped. The corner met (`corner_turned` ..) into `contact_point` ..; a
//! note of the meeting at `bump_note` .. (the current car, the other, the corner, the
//! point; two parts 0, as `zero`); contact record 6's force 0, the work car's contact
//! flag = 1, the other's contact flag = 1; the speed between them into
//! `bump_relative_speed` and its length into `bump_speed_length`; the speeds shared out
//! (each axis a x `bump_share_a` + b x `bump_share_b` and the other way); the places'
//! difference / 65536 x the push scale into `bump_push`, times `three`
//! (`physics::scale_vector`), added to the work car's speed and taken from the other's;
//! both speeds turned into the cars' frames (`scene::rotate_point`);
//! `physics::place_sound_e1_other`.
//!
//! The flow and the calls are Rust; the straight x87 runs are plain Rust over `X`
//! (`math/x87_float.rs`) in the named helpers below, the original's instructions being
//! recorded in the tests' `reference_vectors`.
//!
//! Caller: `physics::corner_hits_car` (its tail). Refuses a null block (before anything)
//! and what `place_sound_e1_other` refuses (what was written put back).

use crate::effects::cell_x87::CellX87;
use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::step_families::ContactInputs;
use crate::race::numbers_store::Numbers;
use crate::session::{Dword, Float};

use crate::math::x87::loaded;
use crate::math::x87_float::int;
use crate::physics::scale_vector::scale_vector;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

/// The block's speed (+0x198 ..) turned by its matrix (+0x9CC) into +0x1A4 ...
fn turn_speed(block: &Car) {
    let turned = rotate_point(
        &block.car_inverse_matrix,
        [0, 1, 2].map(|k| extended(block.velocity[k].get())),
    );
    for (cell, e) in block.local_velocity.iter().zip(turned) {
        cell.set_bits(rounded(e));
    }
}

fn steps(
    c: &ContactInputs<'_>,
    other_car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    for k in 0..3 {
        c.contact.contact_point[k].set(loaded(&c.contact.corner_turned[k]));
    }
    let zero = ZERO;
    let note = [
        c.work.current_car.get(),
        c.contact.other_car_number.get(),
        c.contact.corner_in[0].bits(),
        zero,
        c.contact.corner_in[2].bits(),
        c.contact.contact_point[0].bits(),
        zero,
        c.contact.contact_point[2].bits(),
    ];
    for (k, v) in note.iter().enumerate() {
        c.contact.bump_note[k].set(*v);
    }
    for v in &c.tables.contacts[6].force {
        v.set(0.0);
    }
    c.car.contact_flag.set(1);
    other_car.contact_flag.set(1);
    relative_speed(c.car, c.contact, &other_car.velocity);
    c.contact
        .bump_speed_length
        .set_bits(rounded(vector_length(&c.contact.bump_relative_speed)));
    // The car `other_car_block` names is `other_car`.
    share_out_speeds(c.car, c.contact, &other_car.velocity);
    // 0x45F98C: pushed apart by the places' difference.
    push_apart(c.car, c.contact, c.numbers, &other_car.position);
    let factor = extended(c.numbers.three.get());
    scale_vector(&c.contact.bump_push, factor);
    scene.drawing.scene.scratch.set(rounded(factor));
    add_push(c.car, c.contact, &other_car.velocity);
    turn_speed(other_car);
    turn_speed(c.car);
    { crate::physics::place_sound_e1_other::place_sound_e1_other(&c.sound) }?;
    Ok(())
}

/// `other_car` the other car's block.
pub fn bump_cars(
    c: &ContactInputs<'_>,
    other_car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    if c.contact.other_car_block.get() == 0 {
        return Err(BadInput("a null car block").into());
    }
    steps(c, other_car, scene)
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// the work car's speed less `d` (the other car's), on each axis, into the relative speed.
fn relative_speed(car: &Car, contact: &ContactWork, d: &[Float; 3]) {
    for (k, other) in d.iter().enumerate() {
        let t0 = car.velocity[k].ld();
        let t1 = other.ld();
        let t2 = t0 - t1;
        contact.bump_relative_speed[k].st(t2);
    }
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// the two cars' speeds shared out on each axis, with a the work car's speed and b the
/// other's (`s`): `a' = a x share_a + b x share_b`, `b' = a x share_b + b x share_a`.
fn share_out_speeds(car: &Car, contact: &ContactWork, s: &[Float; 3]) {
    for (k, other) in s.iter().enumerate() {
        let t0 = car.velocity[k].ld();
        let t1 = contact.bump_share_b.ld();
        let t2 = t0 * t1;
        let t3 = other.ld();
        let t4 = contact.bump_share_a.ld();
        let t5 = t3 * t4;
        let t6 = t2 + t5;
        let t7 = car.velocity[k].ld();
        let t8 = contact.bump_share_a.ld();
        let t9 = t7 * t8;
        let t10 = other.ld();
        let t11 = contact.bump_share_b.ld();
        let t12 = t10 * t11;
        let t13 = t9 + t12;
        car.velocity[k].st(t13);
        other.st(t6);
    }
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// the push on each axis: the places' difference (the work car's less `d`'s) over 65536
/// times the push scale.
fn push_apart(car: &Car, contact: &ContactWork, numbers: &Numbers, d: &[Dword; 3]) {
    for (k, other) in d.iter().enumerate() {
        let t0 = int(car.position[k].get() as i32);
        let t1 = int(other.get() as i32);
        let t2 = t0 - t1;
        let t3 = numbers.k65536.ld();
        let t4 = t2 / t3;
        let t5 = contact.bump_push_scale.ld();
        let t6 = t4 * t5;
        contact.bump_push[k].st(t6);
    }
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// the push added to the work car's speed on each axis, then taken from `d`'s.
fn add_push(car: &Car, contact: &ContactWork, d: &[Float; 3]) {
    for k in 0..3 {
        let t0 = car.velocity[k].ld();
        let t1 = contact.bump_push[k].ld();
        let t2 = t0 + t1;
        car.velocity[k].st(t2);
    }
    for (k, other) in d.iter().enumerate() {
        let t9 = other.ld();
        let t10 = contact.bump_push[k].ld();
        let t11 = t9 - t10;
        other.st(t11);
    }
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::x87_mem::view;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "bump_cars line 66",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A170, In::F), (0x31A174, In::F), (0x31A178, In::F)],
                        writes: &[(0x2D3CD8, Out::F), (0x2D3CDC, Out::F), (0x2D3CE0, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x198, In::F), (0x19C, In::F), (0x1A0, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    relative_speed(
                        race().shared.car(),
                        &race().physics.contact,
                        view(a[1], 0x198),
                    );
                },
            },
            Block {
                name: "bump_cars line 80",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x2D3BAC, In::F),
                            (0x2D3BB0, In::F),
                            (0x31A170, In::F),
                            (0x31A174, In::F),
                            (0x31A178, In::F),
                        ],
                        writes: &[(0x31A170, Out::F), (0x31A174, Out::F), (0x31A178, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x198, In::F), (0x19C, In::F), (0x1A0, In::F)],
                        writes: &[(0x198, Out::F), (0x19C, Out::F), (0x1A0, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    share_out_speeds(
                        race().shared.car(),
                        &race().physics.contact,
                        view(a[1], 0x198),
                    );
                },
            },
            Block {
                name: "bump_cars line 121",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5D98, In::F),
                            (0x2D3B88, In::F),
                            (0x31A10C, In::I),
                            (0x31A110, In::I),
                            (0x31A114, In::I),
                        ],
                        writes: &[(0x2D3B9C, Out::F), (0x2D3BA0, Out::F), (0x2D3BA4, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x134, In::I), (0x138, In::I), (0x13C, In::I)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    push_apart(
                        crate::race::owner::race().shared.car(),
                        &crate::race::owner::race().physics.contact,
                        &crate::race::owner::race().course.numbers,
                        view(a[1], 0x134),
                    );
                },
            },
            Block {
                name: "bump_cars line 142",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x2D3B9C, In::F),
                            (0x2D3BA0, In::F),
                            (0x2D3BA4, In::F),
                            (0x31A170, In::F),
                            (0x31A174, In::F),
                            (0x31A178, In::F),
                        ],
                        writes: &[(0x31A170, Out::F), (0x31A174, Out::F), (0x31A178, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x198, In::F), (0x19C, In::F), (0x1A0, In::F)],
                        writes: &[(0x198, Out::F), (0x19C, Out::F), (0x1A0, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    add_push(
                        race().shared.car(),
                        &race().physics.contact,
                        view(a[1], 0x198),
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.bump_cars", &blocks(), 1200, 100);
    }
}
