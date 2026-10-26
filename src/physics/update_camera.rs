//! `physics::update_camera` (0x437A53, two chunks): the camera's step.
//! `physics::camera_low_switch`; the car's place into `dword_6416E8` ..; the light record
//! of `unk_719FFC` (`g_lightRecords` + 0x134 n: places x 1/65536, two less
//! `flt_613FC0`, into `flt_6065C8` .., `flt_606624` ..; +0xC then +0x1C into
//! `dword_606630`; +0x24 into `flt_606570`; +0x20 x 1/65536, kept for the cars marked
//! in `dword_5C5AC0` (n below 0x22), else +0x28, into `flt_606574`). With no switch under way and a
//! fixed mode (0x2A .. 0x2D, 9, 10: `g_nearestCamera2` / `g_replayCameras2` of the
//! checkpoint `unk_71A070`; 0x2E .. 0x31, 7, 8: the first two) the second chunk: the
//! camera at that face's first scene point, `physics::camera_height_bias`, aimed at the
//! car (`fpatan` heading and pitch, x 65536 x `flt_5C5E28` x `flt_5C5DD4`, << 4, the
//! pitch negated), the distance (`physics::vector_length`) into `flt_60631C`, the zoom
//! `unk_71A6A0` (`dword_608B1C`, or 0x180 x the distance / `flt_5C5D48` held at least
//! `flt_5C5D68` with the record's +0x50), the roll and turns 0. Else: the camera at the
//! car (also into `dword_5CCDA0` .., the heading into `dword_5CCD60`); a switch under way
//! ends after `unk_71A670` (the mode back, `unk_71AF88` = 8, modes 7 on to 0) or its share
//! (held 0 ..= 1) into `unk_71A674`; `physics::take_view_a` of the mode and `take_view_b`
//! of the mode before; their figures blended by the share into the turn targets
//! (`dword_608B50` ..), the offsets (`dword_608B30` .., `608B74` ..) and the turns
//! (`unk_71A694` .., << 16); the shares (`dword_608BE0` ..); the zoom held at least
//! `flt_6062D0`; `physics::ease_view_angles`, `ease_view_blend`,
//! `physics::camera_avoid_ground`; the offset's length into `dword_5CCD80`,
//! `dword_6062BC` into `dword_5CCD40`.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them; no flags read (the caller
//!      returns)
//!
//! Caller: `physics::camera_frame`. Refuses a mode past 0x33 on the blending path (the
//! original reads past `off_606320`), and what the ground checks refuse; everything written
//! put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::camera_store::CAMERA_WORK;
use crate::physics::camera_view::{rd, record};
use crate::physics::car_store::CAR;
use crate::physics::flow::{hold_high, hold_low};
use crate::race::course_store::COURSE;
use crate::race::frame_store::FRAME_WORK;
use crate::race::kind_store::KIND_TABLES;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::numbers_store::NUMBERS;
use crate::race::view_vars_store::VIEW_VARS;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::rounded;
use crate::scene::x87_mem::{fild, fistp, fld, fstp};
use rc2k_core::physics::camera_view;

/// The fixed camera (the second chunk, 0x4380E9 ..): `second` the second tables.
fn fixed_camera(second: bool) {
    let (nearest, replay) = if second {
        (
            &crate::race::checkpoints_store::CHECKPOINTS.nearest_camera_2[0],
            &crate::race::checkpoints_store::CHECKPOINTS.replay_cameras_2[0],
        )
    } else {
        (
            &crate::race::checkpoints_store::CHECKPOINTS.nearest_camera_1[0],
            &crate::race::checkpoints_store::CHECKPOINTS.replay_cameras_1[0],
        )
    };
    let face = crate::race::checkpoints_store::nth(
        replay,
        crate::race::checkpoints_store::nth(nearest, CAR.checkpoint.get() as usize).get() as usize,
    )
    .get();
    let face = (face << 6).wrapping_add(COURSE.course_faces.get());
    let point = rd(face + 4)
        .wrapping_mul(12)
        .wrapping_add(COURSE.scene_points.get());
    for k in 0..3 {
        CAR.camera_position[k as usize].set(rd(point + 4 * k));
    }
    crate::physics::camera_height_bias::camera_height_bias();
    let d = |k: usize| {
        CAR.position[k]
            .get()
            .wrapping_sub(CAR.camera_position[k].get())
    };
    let (dx, dy, dz) = (d(0), d(1), d(2));
    let v = [dx, dz, dy];
    x87_1(
        (crate::pointers::game_address(v.as_ptr())) as usize,
        crate::race::store::asm_base() as usize,
    );
    CAR.camera_heading.set(CAR.camera_heading.get() << 4);
    CAR.camera_pitch
        .set((CAR.camera_pitch.get() << 4).wrapping_neg());
    let w = [dx, dy, dz];
    x87_2(
        (crate::pointers::game_address(w.as_ptr())) as usize,
        crate::race::store::asm_base() as usize,
    );
    DRAW.scratch.set(dz);
    VIEW_VARS.camera_distance.set_bits(rounded({
        crate::physics::vector_length::vector_length(VIEW_VARS.camera_vector[0].addr())
    }));
    CAR.camera_zoom
        .set_bits(CAMERA_WORK.camera_zoom_kept.bits());
    if rd(record() + 0x50) != 0 {
        CAR.camera_zoom.set_bits(0x180);
        x87_3(crate::race::store::asm_base() as usize);
        hold_low(&CAR.camera_zoom, &NUMBERS.k384);
    }
    CAR.camera_roll_turn.set(0);
    for turn in &CAR.camera_offset_turns {
        turn.set(0);
    }
}

/// The steps. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::camera_view::{fixed_camera, keeps_light_figure,
// switch_over, mode_after_switch, check_modes}` (the x87 sums stay).
pub fn steps() -> Result<(), Refused> {
    crate::physics::camera_low_switch::camera_low_switch()?;
    for k in 0..3 {
        FRAME_WORK.camera_car[k].set(CAR.position[k].get());
    }
    let car = CAR.car_kind.get();
    let light = LIGHT_RECORDS.record_addr(car);
    x87_4(light as usize, crate::race::store::asm_base() as usize);
    VIEW_VARS.light_c.set(rd(light + 0xC));
    VIEW_VARS.light_c.set(rd(light + 0x1C));
    if !camera_view::keeps_light_figure(car, || KIND_TABLES.kind_has_views[car as usize].get()) {
        VIEW_VARS
            .light_b
            .set_bits(crate::math::x87::loaded(rd(light + 0x28)));
    }
    if CAR.camera_switch_active.get() == 0
        && let Some(second) = camera_view::fixed_camera(CAR.camera_view.get())
    {
        fixed_camera(second);
        return Ok(());
    }
    // 0x437BE4: the camera at the car, the views blended.
    for (k, copy) in [&COURSE.place_x, &COURSE.place_y, &COURSE.place_z]
        .into_iter()
        .enumerate()
    {
        let v = CAR.position[k].get();
        copy.set(v);
        CAR.camera_position[k].set(v);
    }
    COURSE.place_heading.set(CAR.car_heading.get());
    if CAR.camera_switch_active.get() != 0 {
        if camera_view::switch_over(
            CAR.step_clock.get(),
            CAR.camera_switch_clock.get(),
            CAR.camera_switch_length.get(),
        ) {
            CAR.camera_switch_active.set(0);
            CAR.camera_switch_share.set_bits(0);
            CAR.camera_view.set(CAR.camera_blend_view.get());
            CAR.value_jump_steps.set(camera_view::SWITCH_HOLD);
            let after = camera_view::mode_after_switch(CAR.camera_view.get());
            if after != CAR.camera_view.get() {
                CAR.camera_view.set(after);
            }
        } else {
            x87_5(crate::race::store::asm_base() as usize);
            hold_high(&CAR.camera_switch_share, &NUMBERS.one);
            hold_low(&CAR.camera_switch_share, &NUMBERS.zero);
        }
    }
    let (mode, before) = (CAR.camera_view.get(), CAR.camera_blend_view.get());
    camera_view::check_modes(mode, before)?;
    crate::physics::take_view_a::take_view_a(VIEW_VARS.view_records[mode as usize].get());
    crate::physics::take_view_b::take_view_b(VIEW_VARS.view_records[before as usize].get());
    x87_6(crate::race::store::asm_base() as usize);
    // The offsets and turns: (to, from b, from a) each a + (b - a) x share, x 65536.
    let (work, offset_turns) = (&CAMERA_WORK, &CAR.camera_offset_turns);
    for (to, x, y) in [
        (
            work.camera_offset_x.addr(),
            work.view_b_offset[0].addr(),
            work.view_a_offset[0].addr(),
        ),
        (
            work.camera_offset_y.addr(),
            work.view_b_offset[1].addr(),
            work.view_a_offset[1].addr(),
        ),
        (
            work.camera_offset_z.addr(),
            work.view_b_offset[2].addr(),
            work.view_a_offset[2].addr(),
        ),
        (
            work.view_turn_x.addr(),
            work.view_b_turn[0].addr(),
            work.view_a_turn[0].addr(),
        ),
        (
            work.view_turn_y.addr(),
            work.view_b_turn[1].addr(),
            work.view_a_turn[1].addr(),
        ),
        (
            work.view_turn_z.addr(),
            work.view_b_turn[2].addr(),
            work.view_a_turn[2].addr(),
        ),
        (
            offset_turns[0].addr(),
            work.view_b_point[0].addr(),
            work.view_a_point[0].addr(),
        ),
        (
            offset_turns[1].addr(),
            work.view_b_point[1].addr(),
            work.view_a_point[1].addr(),
        ),
        (
            offset_turns[2].addr(),
            work.view_b_point[2].addr(),
            work.view_a_point[2].addr(),
        ),
    ] {
        x87_7(
            x as usize,
            y as usize,
            crate::race::store::asm_base() as usize,
            to as usize,
        );
    }
    for turn in &CAR.camera_offset_turns {
        turn.set(turn.get() << 16);
    }
    CAMERA_WORK
        .view_share_pitch
        .set_bits(CAMERA_WORK.view_a_shares[0].bits());
    CAMERA_WORK
        .view_share_heading
        .set_bits(CAMERA_WORK.view_a_shares[1].bits());
    CAMERA_WORK
        .view_share_roll
        .set_bits(CAMERA_WORK.view_a_shares[2].bits());
    CAMERA_WORK.view_blend_aux.set(CAMERA_WORK.view_a_aux.get());
    CAMERA_WORK
        .view_blend_aux_2
        .set(CAMERA_WORK.view_a_aux_2.get());
    x87_8(crate::race::store::asm_base() as usize);
    hold_low(&CAR.camera_zoom, &VIEW_VARS.camera_zoom_hold);
    crate::physics::ease_view_angles::ease_view_angles();
    crate::physics::ease_view_blend::ease_view_blend();
    crate::physics::camera_avoid_ground::steps()?;
    x87_9(crate::race::store::asm_base() as usize);
    COURSE.camera_first.set(VIEW_VARS.view_angles[1].get());
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x31A688] = ((atan2(int [v + 0], int [v + 4]) * [rc + 0x1C5D98]) * [rc + 0x1C5E28]) * [rc + 0x1C5DD4]`
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_1(v: usize, rc: usize) {
    let t0 = fild(v, 0);
    let t1 = fild(v, 4);
    let t2 = t0.atan2(t1);
    let t3 = fld(rc, 0x1C5D98);
    let t4 = t2 * t3;
    let t5 = fld(rc, 0x1C5E28);
    let t6 = t4 * t5;
    let t7 = fld(rc, 0x1C5DD4);
    let t8 = t6 * t7;
    fistp(rc, 0x31A688, t8);
    let t9 = fild(v, 8);
    let t10 = fild(v, 0);
    let t11 = t10 * t10;
    let t12 = fild(v, 4);
    let t13 = t12 * t12;
    let t14 = t11 + t13;
    let t15 = t14.sqrt();
    let t16 = t9.atan2(t15);
    let t17 = fld(rc, 0x1C5D98);
    let t18 = t16 * t17;
    let t19 = fld(rc, 0x1C5E28);
    let t20 = t18 * t19;
    let t21 = fld(rc, 0x1C5DD4);
    let t22 = t20 * t21;
    fistp(rc, 0x31A690, t22);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2062D8] = int [v + 0] / [rc + 0x1C5D98]`
/// `[rc + 0x2062DC] = int [v + 4] / [rc + 0x1C5D98]`
/// `[rc + 0x2062E0] = int [v + 8] / [rc + 0x1C5D98]`
fn x87_2(v: usize, rc: usize) {
    let t0 = fild(v, 0);
    let t1 = fld(rc, 0x1C5D98);
    let t2 = t0 / t1;
    fstp(rc, 0x2062D8, t2);
    let t3 = fild(v, 4);
    let t4 = fld(rc, 0x1C5D98);
    let t5 = t3 / t4;
    fstp(rc, 0x2062DC, t5);
    let t6 = fild(v, 8);
    let t7 = fld(rc, 0x1C5D98);
    let t8 = t6 / t7;
    fstp(rc, 0x2062E0, t8);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A6A0] = (int [rc + 0x31A6A0] * [rc + 0x20631C]) / [rc + 0x1C5D48]`
fn x87_3(rc: usize) {
    let t0 = fild(rc, 0x31A6A0);
    let t1 = fld(rc, 0x20631C);
    let t2 = t0 * t1;
    let t3 = fld(rc, 0x1C5D48);
    let t4 = t2 / t3;
    fstp(rc, 0x31A6A0, t4);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2065C8] = int [d + 0] * [rc + 0x1C5DA0]`
/// `[rc + 0x2065CC] = (int [d + 4] * [rc + 0x1C5DA0]) - [rc + 0x213FC0]`
/// `[rc + 0x2065D0] = int [d + 8] * [rc + 0x1C5DA0]`
/// `[rc + 0x206624] = int [d + 0x10] * [rc + 0x1C5DA0]`
/// `[rc + 0x206628] = (int [d + 0x14] * [rc + 0x1C5DA0]) - [rc + 0x213FC0]`
/// `[rc + 0x20662C] = int [d + 0x18] * [rc + 0x1C5DA0]`
/// `[rc + 0x206570] = [d + 0x24]`
/// `[rc + 0x206574] = int [d + 0x20] * [rc + 0x1C5DA0]`
fn x87_4(d: usize, rc: usize) {
    let t0 = fild(d, 0);
    let t1 = fld(rc, 0x1C5DA0);
    let t2 = t0 * t1;
    fstp(rc, 0x2065C8, t2);
    let t3 = fild(d, 4);
    let t4 = fld(rc, 0x1C5DA0);
    let t5 = t3 * t4;
    let t6 = fld(rc, 0x213FC0);
    let t7 = t5 - t6;
    fstp(rc, 0x2065CC, t7);
    let t8 = fild(d, 8);
    let t9 = fld(rc, 0x1C5DA0);
    let t10 = t8 * t9;
    fstp(rc, 0x2065D0, t10);
    let t11 = fild(d, 0x10);
    let t12 = fld(rc, 0x1C5DA0);
    let t13 = t11 * t12;
    fstp(rc, 0x206624, t13);
    let t14 = fild(d, 0x14);
    let t15 = fld(rc, 0x1C5DA0);
    let t16 = t14 * t15;
    let t17 = fld(rc, 0x213FC0);
    let t18 = t16 - t17;
    fstp(rc, 0x206628, t18);
    let t19 = fild(d, 0x18);
    let t20 = fld(rc, 0x1C5DA0);
    let t21 = t19 * t20;
    fstp(rc, 0x20662C, t21);
    let t22 = fld(d, 0x24);
    fstp(rc, 0x206570, t22);
    let t23 = fild(d, 0x20);
    let t24 = fld(rc, 0x1C5DA0);
    let t25 = t23 * t24;
    fstp(rc, 0x206574, t25);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A674] = (int [rc + 0x319FE0] - int [rc + 0x31A668]) / int [rc + 0x31A670]`
fn x87_5(rc: usize) {
    let t0 = fild(rc, 0x319FE0);
    let t1 = fild(rc, 0x31A668);
    let t2 = t0 - t1;
    let t3 = fild(rc, 0x31A670);
    let t4 = t2 / t3;
    fstp(rc, 0x31A674, t4);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x208B58] = ((int [rc + 0x208B70] - int [rc + 0x208B64]) * [rc + 0x31A674]) + int [rc + 0x208B64]`
/// `int [rc + 0x208B50] = ((int [rc + 0x208B68] - int [rc + 0x208B5C]) * [rc + 0x31A674]) + int [rc + 0x208B5C]`
/// `int [rc + 0x208B54] = ((int [rc + 0x208B6C] - int [rc + 0x208B60]) * [rc + 0x31A674]) + int [rc + 0x208B60]`
fn x87_6(rc: usize) {
    let t0 = fild(rc, 0x208B70);
    let t1 = fild(rc, 0x208B64);
    let t2 = t0 - t1;
    let t3 = fld(rc, 0x31A674);
    let t4 = t2 * t3;
    let t5 = fild(rc, 0x208B64);
    let t6 = t4 + t5;
    fistp(rc, 0x208B58, t6);
    let t7 = fild(rc, 0x208B68);
    let t8 = fild(rc, 0x208B5C);
    let t9 = t7 - t8;
    let t10 = fld(rc, 0x31A674);
    let t11 = t9 * t10;
    let t12 = fild(rc, 0x208B5C);
    let t13 = t11 + t12;
    fistp(rc, 0x208B50, t13);
    let t14 = fild(rc, 0x208B6C);
    let t15 = fild(rc, 0x208B60);
    let t16 = t14 - t15;
    let t17 = fld(rc, 0x31A674);
    let t18 = t16 * t17;
    let t19 = fild(rc, 0x208B60);
    let t20 = t18 + t19;
    fistp(rc, 0x208B54, t20);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [t + 0] = ((([x + 0] - [y + 0]) * [rc + 0x31A674]) + [y + 0]) * [rc + 0x1C5D98]`
fn x87_7(x: usize, y: usize, rc: usize, t: usize) {
    let t0 = fld(x, 0);
    let t1 = fld(y, 0);
    let t2 = t0 - t1;
    let t3 = fld(rc, 0x31A674);
    let t4 = t2 * t3;
    let t5 = fld(y, 0);
    let t6 = t4 + t5;
    let t7 = fld(rc, 0x1C5D98);
    let t8 = t6 * t7;
    fistp(t, 0, t8);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A6A0] = int [rc + 0x208BF0]`
fn x87_8(rc: usize) {
    let t0 = fild(rc, 0x208BF0);
    fstp(rc, 0x31A6A0, t0);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_9(rc: usize) {
    let t0 = fild(rc, 0x208B30);
    let t1 = t0 * t0;
    let t2 = fild(rc, 0x208B34);
    let t3 = t2 * t2;
    let t4 = fild(rc, 0x208B38);
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    fistp(rc, 0x1CCD80, t8);
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(v, rc) "fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", ".byte 0xD9, 0xF3", "fmul dword ptr [{rc} + 0x1C5D98]", "fmul dword ptr [{rc} + 0x1C5E28]", "fmul dword ptr [{rc} + 0x1C5DD4]", "fistp dword ptr [{rc} + 0x31A688]", "fild dword ptr [{v} + 8]", "fild dword ptr [{v}]", ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 4]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", ".byte 0xD9, 0xF3", "fmul dword ptr [{rc} + 0x1C5D98]", "fmul dword ptr [{rc} + 0x1C5E28]", "fmul dword ptr [{rc} + 0x1C5DD4]", "fistp dword ptr [{rc} + 0x31A690]");
    reference!(ref_1(v, rc) "fild dword ptr [{v}]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2062D8]", "fild dword ptr [{v} + 4]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2062DC]", "fild dword ptr [{v} + 8]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2062E0]");
    reference!(ref_2(rc) "fild dword ptr [{rc} + 0x31A6A0]", "fmul dword ptr [{rc} + 0x20631C]", "fdiv dword ptr [{rc} + 0x1C5D48]", "fstp dword ptr [{rc} + 0x31A6A0]");
    reference!(ref_3(d, rc) "fild dword ptr [{d}]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fstp dword ptr [{rc} + 0x2065C8]", "fild dword ptr [{d} + 0x4]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fsub dword ptr [{rc} + 0x213FC0]", "fstp dword ptr [{rc} + 0x2065CC]", "fild dword ptr [{d} + 0x8]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fstp dword ptr [{rc} + 0x2065D0]", "fild dword ptr [{d} + 0x10]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fstp dword ptr [{rc} + 0x206624]", "fild dword ptr [{d} + 0x14]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fsub dword ptr [{rc} + 0x213FC0]", "fstp dword ptr [{rc} + 0x206628]", "fild dword ptr [{d} + 0x18]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fstp dword ptr [{rc} + 0x20662C]", "fld dword ptr [{d} + 0x24]", "fstp dword ptr [{rc} + 0x206570]", "fild dword ptr [{d} + 0x20]", "fmul dword ptr [{rc} + 0x1C5DA0]", "fstp dword ptr [{rc} + 0x206574]");
    reference!(ref_4(rc) "fild dword ptr [{rc} + 0x319FE0]", "fisub dword ptr [{rc} + 0x31A668]", "fidiv dword ptr [{rc} + 0x31A670]", "fstp dword ptr [{rc} + 0x31A674]");
    reference!(ref_5(rc) "fild dword ptr [{rc} + 0x208B70]", "fisub dword ptr [{rc} + 0x208B64]", "fmul dword ptr [{rc} + 0x31A674]", "fiadd dword ptr [{rc} + 0x208B64]", "fistp dword ptr [{rc} + 0x208B58]", "fild dword ptr [{rc} + 0x208B68]", "fisub dword ptr [{rc} + 0x208B5C]", "fmul dword ptr [{rc} + 0x31A674]", "fiadd dword ptr [{rc} + 0x208B5C]", "fistp dword ptr [{rc} + 0x208B50]", "fild dword ptr [{rc} + 0x208B6C]", "fisub dword ptr [{rc} + 0x208B60]", "fmul dword ptr [{rc} + 0x31A674]", "fiadd dword ptr [{rc} + 0x208B60]", "fistp dword ptr [{rc} + 0x208B54]");
    reference!(ref_6(x, y, rc, t) "fld dword ptr [{x}]", "fsub dword ptr [{y}]", "fmul dword ptr [{rc} + 0x31A674]", "fadd dword ptr [{y}]", "fmul dword ptr [{rc} + 0x1C5D98]", "fistp dword ptr [{t}]");
    reference!(ref_7(rc) "fild dword ptr [{rc} + 0x208BF0]", "fstp dword ptr [{rc} + 0x31A6A0]");
    reference!(ref_8(rc) "fild dword ptr [{rc} + 0x208B30]", ".byte 0xD8, 0xC8", "fild dword ptr [{rc} + 0x208B34]", ".byte 0xD8, 0xC8", "fild dword ptr [{rc} + 0x208B38]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", "fistp dword ptr [{rc} + 0x1CCD80]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "update_camera line 64",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (4, In::I), (8, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x1C5D98, In::F), (0x1C5DD4, In::F), (0x1C5E28, In::F)],
                        writes: &[(0x31A688, Out::I), (0x31A690, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| x87_1(a[0], a[1]),
            },
            Block {
                name: "update_camera line 89",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I), (4, In::I), (8, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x1C5D98, In::F)],
                        writes: &[(0x2062D8, Out::F), (0x2062DC, Out::F), (0x2062E0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1]),
            },
            Block {
                name: "update_camera line 109",
                operands: vec![Operand::Buffer {
                    reads: &[(0x1C5D48, In::F), (0x20631C, In::F), (0x31A6A0, In::I)],
                    writes: &[(0x31A6A0, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0]) },
                plain: |a| x87_3(a[0]),
            },
            Block {
                name: "update_camera line 133",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0, In::I),
                            (4, In::I),
                            (8, In::I),
                            (0x10, In::I),
                            (0x14, In::I),
                            (0x18, In::I),
                            (0x20, In::I),
                            (0x24, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x1C5DA0, In::F), (0x213FC0, In::F)],
                        writes: &[
                            (0x206570, Out::F),
                            (0x206574, Out::F),
                            (0x2065C8, Out::F),
                            (0x2065CC, Out::F),
                            (0x2065D0, Out::F),
                            (0x206624, Out::F),
                            (0x206628, Out::F),
                            (0x20662C, Out::F),
                        ],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| x87_4(a[0], a[1]),
            },
            Block {
                name: "update_camera line 190",
                operands: vec![Operand::Buffer {
                    reads: &[(0x319FE0, In::I), (0x31A668, In::I), (0x31A670, In::I)],
                    writes: &[(0x31A674, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0]) },
                plain: |a| x87_5(a[0]),
            },
            Block {
                name: "update_camera line 205",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x208B5C, In::I),
                        (0x208B60, In::I),
                        (0x208B64, In::I),
                        (0x208B68, In::I),
                        (0x208B6C, In::I),
                        (0x208B70, In::I),
                        (0x31A674, In::F),
                    ],
                    writes: &[(0x208B50, Out::I), (0x208B54, Out::I), (0x208B58, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0]) },
                plain: |a| x87_6(a[0]),
            },
            Block {
                name: "update_camera line 235",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x1C5D98, In::F), (0x31A674, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1], a[2], a[3]) },
                plain: |a| x87_7(a[0], a[1], a[2], a[3]),
            },
            Block {
                name: "update_camera line 257",
                operands: vec![Operand::Buffer {
                    reads: &[(0x208BF0, In::I)],
                    writes: &[(0x31A6A0, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_7(a[0]) },
                plain: |a| x87_8(a[0]),
            },
            Block {
                name: "update_camera line 266",
                operands: vec![Operand::Buffer {
                    reads: &[(0x208B30, In::I), (0x208B34, In::I), (0x208B38, In::I)],
                    writes: &[(0x1CCD80, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_8(a[0]) },
                plain: |a| x87_9(a[0]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
