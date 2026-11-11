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
//! `asm_reference` in the tests.
//!
//! Caller: `physics::corner_hits_car` (its tail). Refuses a null block (before anything)
//! and what `place_sound_e1_other` refuses (what was written put back).

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::race::numbers_store::Numbers;
use crate::race::owner::Race;
use crate::session::{Dword, Float};

use crate::math::x87::loaded;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics::scale_vector::scale_vector;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

/// The block's speed (+0x198 ..) turned by its matrix (+0x9CC) into +0x1A4 ...
fn turn_speed(block: &Car) {
    let turned = rotate_point(
        &block.car_inverse_matrix,
        [0, 1, 2].map(|k| extended(block.velocity[k].bits())),
    );
    for (cell, e) in block.local_velocity.iter().zip(turned) {
        cell.set_bits(rounded(e));
    }
}

fn steps(race: &Race, other_car: u32, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let race_owner = race;
    for k in 0..3 {
        race_owner.physics.contact.contact_point[k]
            .set_bits(loaded(&race_owner.physics.contact.corner_turned[k]).to_bits());
    }
    let zero = ZERO;
    let note = [
        race.shared.work.current_car.get(),
        race_owner.physics.contact.other_car_number.get(),
        race_owner.physics.contact.corner_in[0].bits(),
        zero,
        race_owner.physics.contact.corner_in[2].bits(),
        race_owner.physics.contact.contact_point[0].bits(),
        zero,
        race_owner.physics.contact.contact_point[2].bits(),
    ];
    for (k, v) in note.iter().enumerate() {
        race_owner.physics.contact.bump_note[k].set(*v);
    }
    for v in &race.shared.physics_tables.contacts[6].force {
        v.set_bits(0);
    }
    race.shared.car.contact_flag.set(1);
    let other_car = crate::race::trap::car_at(other_car)?;
    other_car.contact_flag.set(1);
    relative_speed(&race.shared.car, &race.physics.contact, &other_car.velocity);
    race_owner
        .physics
        .contact
        .bump_speed_length
        .set_bits(rounded({
            vector_length(&race_owner.physics.contact.bump_relative_speed)
        }));
    // The car `other_car_block` names (the same as `other_car` in play).
    let met_car = Car::at(race_owner.physics.contact.other_car_block.get());
    share_out_speeds(&race.shared.car, &race.physics.contact, &met_car.velocity);
    // 0x45F98C: pushed apart by the places' difference.
    let other = Car::at(race_owner.physics.contact.other_car_block.get());
    push_apart(
        &race.shared.car,
        &race.physics.contact,
        &race.course.numbers,
        &other.position,
    );
    let factor = extended(race_owner.course.numbers.three.bits());
    scale_vector(&race_owner.physics.contact.bump_push, factor);
    scene.drawing.scene.scratch.set(rounded(factor));
    add_push(&race.shared.car, &race.physics.contact, &other.velocity);
    turn_speed(met_car);
    turn_speed(&race.shared.car);
    { crate::physics::place_sound_e1_other::place_sound_e1_other(race) }?;
    Ok(())
}

/// `other_car` the other car's block (an address in the game's memory).
pub fn bump_cars(
    race: &Race,
    other_car: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    if other_car == 0 || race.physics.contact.other_car_block.get() == 0 {
        return Err(BadInput("a null car block").into());
    }
    steps(race, other_car, scene)
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// the work car's speed less `d` (the other car's), on each axis, into the relative speed.
fn relative_speed(car: &Car, contact: &ContactWork, d: &[Float; 3]) {
    for (k, other) in d.iter().enumerate() {
        let t0 = ld(car.velocity[k].bits());
        let t1 = ld(other.bits());
        let t2 = t0 - t1;
        contact.bump_relative_speed[k].set_bits(st(t2));
    }
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// the two cars' speeds shared out on each axis, with a the work car's speed and b the
/// other's (`s`): `a' = a x share_a + b x share_b`, `b' = a x share_b + b x share_a`.
fn share_out_speeds(car: &Car, contact: &ContactWork, s: &[Float; 3]) {
    for (k, other) in s.iter().enumerate() {
        let t0 = ld(car.velocity[k].bits());
        let t1 = ld(contact.bump_share_b.bits());
        let t2 = t0 * t1;
        let t3 = ld(other.bits());
        let t4 = ld(contact.bump_share_a.bits());
        let t5 = t3 * t4;
        let t6 = t2 + t5;
        let t7 = ld(car.velocity[k].bits());
        let t8 = ld(contact.bump_share_a.bits());
        let t9 = t7 * t8;
        let t10 = ld(other.bits());
        let t11 = ld(contact.bump_share_b.bits());
        let t12 = t10 * t11;
        let t13 = t9 + t12;
        car.velocity[k].set_bits(st(t13));
        other.set_bits(st(t6));
    }
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// the push on each axis: the places' difference (the work car's less `d`'s) over 65536
/// times the push scale.
fn push_apart(car: &Car, contact: &ContactWork, numbers: &Numbers, d: &[Dword; 3]) {
    for (k, other) in d.iter().enumerate() {
        let t0 = int(car.position[k].bits() as i32);
        let t1 = int(other.bits() as i32);
        let t2 = t0 - t1;
        let t3 = ld(numbers.k65536.bits());
        let t4 = t2 / t3;
        let t5 = ld(contact.bump_push_scale.bits());
        let t6 = t4 * t5;
        contact.bump_push[k].set_bits(st(t6));
    }
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// the push added to the work car's speed on each axis, then taken from `d`'s.
fn add_push(car: &Car, contact: &ContactWork, d: &[Float; 3]) {
    for k in 0..3 {
        let t0 = ld(car.velocity[k].bits());
        let t1 = ld(contact.bump_push[k].bits());
        let t2 = t0 + t1;
        car.velocity[k].set_bits(st(t2));
    }
    for (k, other) in d.iter().enumerate() {
        let t9 = ld(other.bits());
        let t10 = ld(contact.bump_push[k].bits());
        let t11 = t9 - t10;
        other.set_bits(st(t11));
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::view;

    // The original's instructions, one function per run.
    reference!(ref_0(rc, d) "fld dword ptr [{rc} + 0x31A170]", "fsub dword ptr [{d} + 0x198]", "fstp dword ptr [{rc} + 0x2D3CD8]", "fld dword ptr [{rc} + 0x31A174]", "fsub dword ptr [{d} + 0x19C]", "fstp dword ptr [{rc} + 0x2D3CDC]", "fld dword ptr [{rc} + 0x31A178]", "fsub dword ptr [{d} + 0x1A0]", "fstp dword ptr [{rc} + 0x2D3CE0]");
    reference!(ref_1(rc, s) "fld dword ptr [{rc} + 0x31A170]", "fmul dword ptr [{rc} + 0x2D3BB0]", "fld dword ptr [{s} + 0x198]", "fmul dword ptr [{rc} + 0x2D3BAC]", ".byte 0xDE, 0xC1", "fld dword ptr [{rc} + 0x31A170]", "fmul dword ptr [{rc} + 0x2D3BAC]", "fld dword ptr [{s} + 0x198]", "fmul dword ptr [{rc} + 0x2D3BB0]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + 0x31A170]", "fstp dword ptr [{s} + 0x198]", "fld dword ptr [{rc} + 0x31A174]", "fmul dword ptr [{rc} + 0x2D3BB0]", "fld dword ptr [{s} + 0x19C]", "fmul dword ptr [{rc} + 0x2D3BAC]", ".byte 0xDE, 0xC1", "fld dword ptr [{rc} + 0x31A174]", "fmul dword ptr [{rc} + 0x2D3BAC]", "fld dword ptr [{s} + 0x19C]", "fmul dword ptr [{rc} + 0x2D3BB0]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + 0x31A174]", "fstp dword ptr [{s} + 0x19C]", "fld dword ptr [{rc} + 0x31A178]", "fmul dword ptr [{rc} + 0x2D3BB0]", "fld dword ptr [{s} + 0x1A0]", "fmul dword ptr [{rc} + 0x2D3BAC]", ".byte 0xDE, 0xC1", "fld dword ptr [{rc} + 0x31A178]", "fmul dword ptr [{rc} + 0x2D3BAC]", "fld dword ptr [{s} + 0x1A0]", "fmul dword ptr [{rc} + 0x2D3BB0]", ".byte 0xDE, 0xC1", "fstp dword ptr [{rc} + 0x31A178]", "fstp dword ptr [{s} + 0x1A0]");
    reference!(ref_2(rc, d) "fild dword ptr [{rc} + 0x31A10C]", "fisub dword ptr [{d} + 0x134]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fmul dword ptr [{rc} + 0x2D3B88]", "fstp dword ptr [{rc} + 0x2D3B9C]", "fild dword ptr [{rc} + 0x31A110]", "fisub dword ptr [{d} + 0x138]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fmul dword ptr [{rc} + 0x2D3B88]", "fstp dword ptr [{rc} + 0x2D3BA0]", "fild dword ptr [{rc} + 0x31A114]", "fisub dword ptr [{d} + 0x13C]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fmul dword ptr [{rc} + 0x2D3B88]", "fstp dword ptr [{rc} + 0x2D3BA4]");
    reference!(ref_3(rc, d) "fld dword ptr [{rc} + 0x31A170]", "fadd dword ptr [{rc} + 0x2D3B9C]", "fstp dword ptr [{rc} + 0x31A170]", "fld dword ptr [{rc} + 0x31A174]", "fadd dword ptr [{rc} + 0x2D3BA0]", "fstp dword ptr [{rc} + 0x31A174]", "fld dword ptr [{rc} + 0x31A178]", "fadd dword ptr [{rc} + 0x2D3BA4]", "fstp dword ptr [{rc} + 0x31A178]", "fld dword ptr [{d} + 0x198]", "fsub dword ptr [{rc} + 0x2D3B9C]", "fstp dword ptr [{d} + 0x198]", "fld dword ptr [{d} + 0x19C]", "fsub dword ptr [{rc} + 0x2D3BA0]", "fstp dword ptr [{d} + 0x19C]", "fld dword ptr [{d} + 0x1A0]", "fsub dword ptr [{rc} + 0x2D3BA4]", "fstp dword ptr [{d} + 0x1A0]");

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
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| {
                    relative_speed(
                        &race().shared.car,
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
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| {
                    share_out_speeds(
                        &race().shared.car,
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
                reference: |a| unsafe { ref_2(a[0], a[1]) },
                plain: |a| {
                    push_apart(
                        &crate::race::owner::race().shared.car,
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
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| {
                    add_push(
                        &race().shared.car,
                        &race().physics.contact,
                        view(a[1], 0x198),
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
