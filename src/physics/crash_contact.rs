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
//! In:  nothing
//! Out: the registers as `lowest_body_point` and, on the long path, `contact_force` leave
//!      them, eax the depth on the short; no flags read (the caller calls on)
//!
//! Caller: `sub_43B635`, every physics step. Refuses a point past 0x3F on the long path
//! (the tables' room), and what its calls refuse; what was written is put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87::{loaded, mul};
use crate::math::x87_run::neg_rounded;
use crate::physics::car_store::CAR;
use crate::physics::contact_force::contact_force;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::dot_product::dot_product;
use crate::physics::flow::{g, hold_high, p};
use crate::physics::lowest_body_point::lowest_body_point;
use crate::physics::place_sound_e5::place_sound_e5;
use crate::physics::sound_cue_e5::sound_cue_e5;
use crate::physics::tables_store::PHYSICS;
use crate::physics::touch_face::FACES;
use crate::race::numbers_store::NUMBERS;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::scene::x87_mem::{fld, fstp};
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
pub fn take_contact(n: usize) {
    copy3(&CONTACT_WORK.contact_car_speed, &CAR.velocity);
    let face = (CAR.probe_point_face[n].get() << 6).wrapping_add(g(FACES));
    let parts = contacts::normal_offsets(CONTACT_WORK.probe_face_flip[n].get());
    for (k, o) in parts.iter().enumerate() {
        let v = unsafe { ((face + o) as *const u32).read_unaligned() };
        CONTACT_WORK.contact_normal[k].set_bits(loaded(v));
    }
    copy3(&CONTACT_WORK.contact_turn_speed, &CAR.spin);
    CONTACT_WORK.contact_point[0].set_bits(CONTACT_WORK.probe_turned_x[n].bits());
    CONTACT_WORK.contact_point[1].set_bits(CONTACT_WORK.probe_turned_y[n].bits());
    CONTACT_WORK.contact_point[2].set_bits(CONTACT_WORK.probe_turned_z[n].bits());
}

/// The deepest point's number (`dword_6D306C`) for the long path; refused past 0x3F.
pub fn point_number() -> Result<usize, Refused> {
    let n = CONTACT_WORK.lowest_point.get();
    contacts::check_body_point(n)?;
    Ok(n as usize)
}

// Converted: `rc2k_core::physics::contacts::{touches, normal_offsets, check_body_point}`
// (the x87 steps stay).
fn steps() -> Result<(), Refused> {
    CAR.contact_found.set(0);
    unsafe { lowest_body_point() }?;
    let depth = CONTACT_WORK.lowest_depth.get();
    if !contacts::touches(
        depth,
        PHYSICS.crash_depth.get(),
        CAR.all_wheels_off.get() != 0,
    ) {
        unsafe { sound_cue_e5() }?;
        return Ok(());
    }
    let n = point_number()?;
    CAR.contact_point_x.set(CONTACT_WORK.probe_x[n].get());
    CAR.contact_point_ground
        .set(CONTACT_WORK.probe_ground_height[n].get());
    CAR.contact_point_z.set(CONTACT_WORK.probe_z[n].get());
    CAR.contact_found.set(1);
    take_contact(n);
    let up = unsafe {
        rotate_point(
            CAR.crash_matrix[0].addr() as u32,
            [extended(0), extended(0x3F80_0000), extended(0)],
        )
    };
    for (k, e) in up.iter().enumerate() {
        CONTACT_WORK.crash_car_up[k].set_bits(rounded(*e));
    }
    CONTACT_WORK.crash_up_dot.set_bits(neg_rounded(unsafe {
        dot_product(
            CONTACT_WORK.crash_car_up[0].addr() as u32,
            CONTACT_WORK.contact_normal[0].addr() as u32,
        )
    }));
    x87_1(crate::race::store::asm_base() as usize);
    p(
        &PHYSICS.contact_record,
        PHYSICS.forces[21].force[0].addr() as u32,
    );
    unsafe { contact_force() };
    unsafe { place_sound_e5() }?;
    for k in 0..3 {
        CONTACT_WORK.ground_force[k].set(PHYSICS.forces[21].force[k].bits());
    }
    for cell in &PHYSICS.forces[21].point {
        cell.set_bits(0);
    }
    x87_2(crate::race::store::asm_base() as usize);
    hold_high(&CONTACT_WORK.crash_ground_scale, &NUMBERS.one);
    CAR.spin[1].set_bits(mul(
        CONTACT_WORK.crash_ground_scale.bits(),
        crate::physics::car_store::CAR.spin[1].bits(),
    ));
    CAR.contact_value.set(CONTACT_WORK.contact_value.bits());
    Ok(())
}

/// The Rust version. The registers it leaves are read by none of its callers.
pub fn crash_contact() -> Result<(), Refused> {
    steps()
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_1(rc: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let t0 = fld(rc, 0x2D31D0);
        let t1 = fld(rc, 0x1C5D38);
        let t2 = t0 + t1;
        let t3 = fld(rc, 0x1C5D3C);
        let t4 = t2 / t3;
        let t5 = fld(rc, 0x1C5D3C);
        let t6 = t4 + t5;
        let t7 = fld(rc, 0x213FC8);
        let t8 = t6 * t7;
        let t9 = fld(rc, 0x21388C);
        let t10 = t8 * t9;
        let t11 = fld(rc, 0x214294);
        let t12 = t10 * t11;
        let t13 = fld(rc, 0x214170);
        let t14 = t12 * t13;
        fstp(rc, 0x2D3100, t14);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2D3014] = ([rc + 0x31A0A0] + [rc + 0x1C5D60]) / [rc + 0x1C5D60]`
fn x87_2(rc: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let t0 = fld(rc, 0x31A0A0);
        let t1 = fld(rc, 0x1C5D60);
        let t2 = t0 + t1;
        let t3 = fld(rc, 0x1C5D60);
        let t4 = t2 / t3;
        fstp(rc, 0x2D3014, t4);
    }
}

#[cfg(test)]
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
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_1(a[0]),
            },
            Block {
                name: "crash_contact line 120",
                operands: vec![Operand::Buffer {
                    reads: &[(0x1C5D60, In::F), (0x31A0A0, In::F)],
                    writes: &[(0x2D3014, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |a| x87_2(a[0]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
