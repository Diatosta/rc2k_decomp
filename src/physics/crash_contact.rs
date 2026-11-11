//! `physics::crash_contact` (0x45EDB5): the body's deepest point against the ground.
//! `unk_71AA0C` = 0; `physics::lowest_body_point`; unless the depth (`dword_6D3068`) is
//! below `dword_613EF8` and, with `unk_71A22C` at 0, below 0 (each by the sign of the
//! `cmp`), only `physics::sound_cue_e5`. Else, for the point n (`dword_6D306C`): its three
//! floats from `dword_6D1E14[n]`, `dword_6D2114[n]`, `dword_6D2014[n]` into `unk_71AA00`
//! .., `unk_71AA0C` = 1; the contact taken (`take_contact`: the car's speed, the face's
//! normal, the turn speed, the point); the car's up (0, 1, 0 turned by `g_crashMatrix`,
//! `scene::rotate_point`) into `flt_6D31C4` .., minus its dot with the normal into
//! `flt_6D31D0`, and from that the slide limit `flt_6D3100`; `physics::contact_force`
//! into force record 21 (`g_forceRecords` +0x2A0), `physics::place_sound_e5`; the force
//! into `dword_6D3018` .., the record's point 0; `flt_6D3014` = (`unk_71A0A0` +
//! `flt_5C5D60`) / `flt_5C5D60` held at most 1, times `unk_71A160`; `unk_71AA10` =
//! `dword_6D310C`.
//!
//! Caller: `physics::car_physics_step`, every physics step. Refuses a point past 0x3F on the long
//! path (the tables' room), and what its calls refuse; what was written is put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87::{loaded, mul};
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_run::neg_rounded;
use crate::physics::contact_force::contact_force;
use crate::physics::dot_product::dot_product;
use crate::physics::flow::hold_high;
use crate::physics::lowest_body_point::lowest_body_point;
use crate::physics::place_sound_e5::place_sound_e5;
use crate::physics::sound_cue_e5::sound_cue_e5;
use crate::race::owner::Race;
use crate::race::owner_physics::Physics;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::session::Float;
use rc2k_core::physics::contacts;

/// Three floats copied as bits, in order.
fn copy3(to: &[Float], from: &[Float]) {
    for k in 0..3 {
        to[k].set_bits(from[k].bits());
    }
}

/// The sign of a `cmp`'s difference (`js`).
pub fn sign(v: u32) -> bool {
    (v as i32) < 0
}

/// The contact of point `n` taken (0x45EE25 / 0x45F02D): the car's speed (`unk_71A170`
/// ..) into `dword_6D30A8` ..; the normal of its face (`unk_71ADC8[n]` in
/// `g_courseFaces`: +0x18, +0x30, +0x38, or with `dword_6D2914[n]` at 1 +0x1C, +0x34,
/// +0x3C) into `qword_6D3078` ..; the turn speed (`unk_71A15C` ..) into `dword_6D30B4`
/// ..; the point (`flt_6D2214[n]`, `flt_6D2314[n]`, `flt_6D2414[n]`) into
/// `dword_6D3090` ...
pub fn take_contact(race: &Race, n: usize) -> Result<(), Refused> {
    copy3(
        &race.physics.contact.contact_car_speed,
        &race.shared.car.velocity,
    );
    let face = race.shared.car.probe_point_face[n].get();
    let parts = contacts::normal_offsets(race.physics.contact.probe_face_flip[n].get());
    for (k, &o) in parts.iter().enumerate() {
        let v = crate::race::course_places::face_field::<u32>(face, o, "a face's normal")?;
        race.physics.contact.contact_normal[k].set_bits(loaded(v));
    }
    copy3(
        &race.physics.contact.contact_turn_speed,
        &race.shared.car.spin,
    );
    race.physics.contact.contact_point[0].set_bits(race.physics.contact.probe_turned_x[n].bits());
    race.physics.contact.contact_point[1].set_bits(race.physics.contact.probe_turned_y[n].bits());
    race.physics.contact.contact_point[2].set_bits(race.physics.contact.probe_turned_z[n].bits());
    Ok(())
}

/// The deepest point's number (`dword_6D306C`) for the long path; refused past 0x3F.
pub fn point_number(physics: &Physics) -> Result<usize, Refused> {
    let n = physics.contact.lowest_point.get();
    contacts::check_body_point(n)?;
    Ok(n as usize)
}

// Converted: `rc2k_core::physics::contacts::{touches, normal_offsets, check_body_point}`
// (the x87 steps stay).
fn steps(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    race.shared.car.contact_found.set(0);
    {
        lowest_body_point(
            &race.shared.car,
            &race.course.numbers,
            &race.physics.contact,
        )
    }?;
    let depth = race.physics.contact.lowest_depth.get();
    if !contacts::touches(
        depth,
        race.shared.physics_tables.crash_depth.get(),
        race.shared.car.all_wheels_off.get() != 0,
    ) {
        { sound_cue_e5(race) }?;
        return Ok(());
    }
    let n = point_number(&race.physics)?;
    race.shared
        .car
        .contact_point_x
        .set(race.physics.contact.probe_x[n].get());
    race.shared
        .car
        .contact_point_ground
        .set(race.physics.contact.probe_ground_height[n].get());
    race.shared
        .car
        .contact_point_z
        .set(race.physics.contact.probe_z[n].get());
    race.shared.car.contact_found.set(1);
    take_contact(race, n)?;
    let up = {
        rotate_point(
            &race.shared.car.crash_matrix,
            [extended(0), extended(0x3F80_0000), extended(0)],
        )
    };
    for (k, e) in up.iter().enumerate() {
        race.physics.contact.crash_car_up[k].set_bits(rounded(*e));
    }
    race.physics.contact.crash_up_dot.set_bits(neg_rounded({
        dot_product(
            &race.physics.contact.crash_car_up,
            &race.physics.contact.contact_normal,
        )
    }));
    x87_1(race);
    contact_force(race, &race.shared.physics_tables.forces[21], scene);
    { place_sound_e5(race) }?;
    for k in 0..3 {
        race.physics.contact.ground_force[k]
            .set(race.shared.physics_tables.forces[21].force[k].bits());
    }
    for cell in &race.shared.physics_tables.forces[21].point {
        cell.set_bits(0);
    }
    x87_2(race);
    hold_high(
        &race.physics.contact.crash_ground_scale,
        &race.course.numbers.one,
    );
    race.shared.car.spin[1].set_bits(mul(
        race.physics.contact.crash_ground_scale.bits(),
        race.shared.car.spin[1].bits(),
    ));
    race.shared
        .car
        .contact_value
        .set(race.physics.contact.contact_value.bits());
    Ok(())
}

pub fn crash_contact(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    steps(race, scene)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_1(race: &Race) {
    let t0 = ld(race.physics.contact.crash_up_dot.bits());
    let t1 = ld(race.course.numbers.one.bits());
    let t2 = t0 + t1;
    let t3 = ld(race.course.numbers.two.bits());
    let t4 = t2 / t3;
    let t5 = ld(race.course.numbers.two.bits());
    let t6 = t4 + t5;
    let t7 = ld(race.shared.physics_tables.gravity.bits());
    let t8 = t6 * t7;
    let t9 = ld(race.shared.physics_tables.step_time.bits());
    let t10 = t8 * t9;
    let t11 = ld(race.shared.physics_tables.wheel_loads[4].bits());
    let t12 = t10 * t11;
    let t13 = ld(race.shared.physics_tables.weight.bits());
    let t14 = t12 * t13;
    race.physics.contact.contact_slide_limit.set_bits(st(t14));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2D3014] = ([rc + 0x31A0A0] + [rc + 0x1C5D60]) / [rc + 0x1C5D60]`
fn x87_2(race: &Race) {
    let t0 = ld(race.shared.car.crash_matrix[5].bits());
    let t1 = ld(race.course.numbers.sixty_four.bits());
    let t2 = t0 + t1;
    let t3 = ld(race.course.numbers.sixty_four.bits());
    let t4 = t2 / t3;
    race.physics.contact.crash_ground_scale.set_bits(st(t4));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fld dword ptr [{rc} + 0x2D31D0]", "fadd dword ptr [{rc} + 0x1C5D38]", "fdiv dword ptr [{rc} + 0x1C5D3C]", "fadd dword ptr [{rc} + 0x1C5D3C]", "fmul dword ptr [{rc} + 0x213FC8]", "fmul dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x214294]", "fmul dword ptr [{rc} + 0x214170]", "fstp dword ptr [{rc} + 0x2D3100]");
    reference!(ref_1(rc) "fld dword ptr [{rc} + 0x31A0A0]", "fadd dword ptr [{rc} + 0x1C5D60]", "fdiv dword ptr [{rc} + 0x1C5D60]", "fstp dword ptr [{rc} + 0x2D3014]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "crash_contact line 103",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5D38, In::F),
                        (0x1C5D3C, In::F),
                        (0x21388C, In::F),
                        (0x213FC8, In::F),
                        (0x214170, In::F),
                        (0x214294, In::F),
                        (0x2D31D0, In::F),
                    ],
                    writes: &[(0x2D3100, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0]) },
                plain: |_| x87_1(crate::race::owner::race()),
            },
            Block {
                name: "crash_contact line 120",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x1C5D60, In::F), (0x31A0A0, In::F)],
                    writes: &[(0x2D3014, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |_| x87_2(crate::race::owner::race()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
