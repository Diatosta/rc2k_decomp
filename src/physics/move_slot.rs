//! `physics::move_slot` (0x462485): one step of a loose object's slot (esi, 0x78 bytes of
//! `g_raceSlots`; +4 .. +0xC its place, whole numbers; +0x10 its matrix; +0x50 .. +0x58
//! its speed; +0x68 .. +0x70 its spin; +0x60 its last face):
//! - the place kept (0x71BE90 ..); +0x74 = the spin's length; when not 0, turned by it:
//!   the axis (`scene::unit_vector` of +0x74 .. +0x7C, so two dwords past the slot, as
//!   the original has it) and angle (+0x74 times `flt_61388C`, the step) through
//!   `physics::axis_turn_matrix` (`flt_5C7010`), `scene::square_matrix`, then
//!   `scene::multiply_matrices` with the matrix into `g_turnedMatrix`, copied back;
//! - the place moved by the speed (times the step and 65536, `fistp`'d through
//!   `dword_62400C`); y's speed less gravity (`flt_613FC8`) times the step;
//! - the ground under it (`physics::find_ground`, mask 0x80, last face +0x60): none, the
//!   place put back and the speed and spin cleared; else the face kept, and when below
//!   the ground (the sign of the difference) set on it, the speed halved (y's turned
//!   round), the spin halved and turned round; then, with |y's speed| below 1, the
//!   matrix from `unk_71BA88` (12 dwords), or from `unk_71BA48` when +0 mod 20 is 5 .. 8;
//!   else `dword_62400C` = |y's speed| (and a call to an empty function).
//!
//! By the look of it debris thrown about in a crash (a reading).
//!
//! Refuses (before anything) what `find_ground` would refuse where the slot moves to.
//!
//! In:  esi the slot
//! Out: the registers as the path leaves them (see `move_slot_at`); no flags read
//!
//! Caller: `physics::move_slots` (inside `pusha` / `popa`).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::cells::At;
use crate::math::x87;
use crate::math::x87_run::{abs_compare, abs_of, mul2_to_int, neg_mul, neg_product_plus};
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::physics::ground_store::GROUND_WORK;
use crate::physics::tables_store::PHYSICS;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::matrices_store::MATRICES;
use crate::race::numbers_store::NUMBERS;
use crate::race::work_store::{RACE_WORK, RaceSlot};
use crate::scene::draw_store::DRAW;
use crate::scene::matrix::multiply_matrices;
use crate::scene::matrix3::rounded;
use crate::scene::view_matrix::{square_matrix, unit_vector};
use crate::session::{Cell32, Dword, Float};

/// `from`'s first cells into `to`'s.
fn copy<A: Cell32, B: Cell32>(from: &[A], to: &[B]) {
    for (t, f) in to.iter().zip(from) {
        t.set_bits(f.bits());
    }
}

/// `find_ground`'s refusal at the place the slot at `s` will move to (worked out without
/// writing: the point is put back).
pub fn slot_refusal(s: &RaceSlot) -> Option<&'static str> {
    let dt = PHYSICS.step_time.bits();
    let k65536 = NUMBERS.k65536.bits();
    let moved = |c: &Dword, v: &Float| {
        c.get()
            .wrapping_add(mul2_to_int(v.bits(), dt, k65536) as u32)
    };
    let (x, z) = (
        moved(&s.place[0], &s.motion[0]),
        moved(&s.place[2], &s.motion[2]),
    );
    let keep = (GROUND_WORK.ground_x.get(), GROUND_WORK.ground_z.get());
    GROUND_WORK.ground_x.set(x);
    GROUND_WORK.ground_z.set(z);
    let why = crate::physics::ground::ground_refusal();
    GROUND_WORK.ground_x.set(keep.0);
    GROUND_WORK.ground_z.set(keep.1);
    why
}

/// The Rust version: the slot `s` (the registers it leaves are dropped by its caller's
/// `popa`).
pub fn move_slot(s: &RaceSlot) -> Result<(), Refused> {
    if let Some(why) = slot_refusal(s) {
        return Err(crate::errors::BadInput(why).into());
    }
    let dt = PHYSICS.step_time.bits();
    let k65536 = NUMBERS.k65536.bits();
    copy(&s.place, &RACE_WORK.slot_place);
    s.spin_length
        .set_bits(rounded(crate::physics::vector_length::vector_length(
            &s.spin,
        )));
    if s.spin_length.bits() != 0 {
        // From the spin's length on: two dwords past the slot, as the original has it (the
        // next slot's first, or what follows the table), so by address.
        let past = crate::pointers::game_address(&s.spin_length as *const Float);
        unit_vector(&At(past), &COURSE_LISTS.spin_axis);
        COURSE_LISTS
            .spin_angle
            .set_bits(x87::mul(s.spin_length.bits(), dt));
        axis_turn_matrix(&COURSE_LISTS.turn_matrix);
        square_matrix(&COURSE_LISTS.turn_matrix);
        multiply_matrices(
            &s.matrix,
            &COURSE_LISTS.turn_matrix,
            &MATRICES.turned_matrix,
        );
        copy(&MATRICES.turned_matrix, &s.matrix);
    }
    for k in [1, 0, 2] {
        let d = mul2_to_int(s.motion[k].bits(), dt, k65536) as u32;
        DRAW.scratch.set(d);
        s.place[k].set(s.place[k].get().wrapping_add(d));
    }
    s.motion[1].set_bits(neg_product_plus(
        PHYSICS.gravity.bits(),
        dt,
        s.motion[1].bits(),
    ));
    GROUND_WORK.ground_x.set(s.place[0].get());
    GROUND_WORK.ground_z.set(s.place[2].get());
    COURSE.ground_face_last.set(s.car_word.get());
    crate::physics::ground_probe_store::GROUND_PROBE
        .settle_mark
        .set(0x80);
    crate::physics::find_ground::find_ground()?;
    let face: u32 = COURSE.ground_slot.get();
    if face == u32::MAX {
        copy(&RACE_WORK.slot_place, &s.place);
        for cell in s.motion.iter().chain(&s.spin) {
            cell.set_bits(0);
        }
        return Ok(());
    }
    s.car_word.set(face);
    let height: u32 = GROUND_WORK.ground_height.get();
    if (s.place[1].get().wrapping_sub(height) as i32) >= 0 {
        return Ok(());
    }
    s.place[1].set(height);
    let half = NUMBERS.half.bits();
    s.motion[0].set_bits(x87::mul(s.motion[0].bits(), half));
    s.motion[1].set_bits(neg_mul(s.motion[1].bits(), half));
    s.motion[2].set_bits(x87::mul(s.motion[2].bits(), half));
    for cell in &s.spin {
        cell.set_bits(neg_mul(cell.bits(), half));
    }
    if abs_compare(s.motion[1].bits(), NUMBERS.one.bits()) & 0x100 != 0 {
        copy(&RACE_WORK.part_matrix_flat[..12], &s.matrix);
        let n = s.part.get() % 20;
        if (n.wrapping_sub(5) as i32) >= 0 && (n.wrapping_sub(9) as i32) < 0 {
            copy(&RACE_WORK.part_matrix_still[..12], &s.matrix);
            return Ok(());
        }
    }
    DRAW.scratch.set(abs_of(s.motion[1].bits()));
    Ok(())
}
