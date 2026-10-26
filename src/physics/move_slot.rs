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
use crate::math::x87;
use crate::math::x87_run::{abs_compare, abs_of, mul2_to_int, neg_mul, neg_product_plus};
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::physics::ground::Ground;
use crate::physics::ground_store::GROUND_WORK;
use crate::physics::tables_store::PHYSICS;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::matrices_store::MATRICES;
use crate::race::numbers_store::NUMBERS;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix::multiply_matrices;
use crate::scene::matrix3::rounded;
use crate::scene::view_matrix::{square_matrix, unit_vector};

pub const SLOT_LEN: u32 = 0x78;

fn at(p: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(p, "physics::move_slot's operand")
}

fn put(p: u32, v: u32) {
    crate::pointers::write_or_end::<u32>(p, v, "physics::move_slot's operand")
}

fn copy(from: u32, to: u32, n: u32) {
    for k in 0..n {
        put(to + 4 * k, at(from + 4 * k));
    }
}

/// `find_ground`'s refusal at the place the slot at `s` will move to (worked out without
/// writing: the point is put back).
pub fn slot_refusal(s: u32) -> Option<&'static str> {
    let dt = PHYSICS.step_time.bits();
    let k65536 = NUMBERS.k65536.bits();
    let moved = |c: u32, v: u32| at(s + c).wrapping_add(mul2_to_int(at(s + v), dt, k65536) as u32);
    let (x, z) = (moved(4, 0x50), moved(0xC, 0x58));
    let g = Ground::game();
    let keep = (GROUND_WORK.ground_x.get(), GROUND_WORK.ground_z.get());
    GROUND_WORK.ground_x.set(x);
    GROUND_WORK.ground_z.set(z);
    let why = crate::physics::ground::ground_refusal(&g);
    GROUND_WORK.ground_x.set(keep.0);
    GROUND_WORK.ground_z.set(keep.1);
    why
}

/// The Rust core: the slot at `s` (the registers it leaves are dropped by its caller's
/// `popa`).
pub unsafe fn move_slot_at(s: u32) -> Result<(), Refused> {
    unsafe {
        if let Some(why) = slot_refusal(s) {
            return Err(crate::errors::BadInput(why).into());
        }
        let dt = PHYSICS.step_time.bits();
        let k65536 = NUMBERS.k65536.bits();
        RACE_WORK.slot_place[0].set(at(s + 4));
        RACE_WORK.slot_place[1].set(at(s + 8));
        RACE_WORK.slot_place[2].set(at(s + 0xC));
        put(
            s + 0x74,
            rounded(crate::physics::vector_length::vector_length(s + 0x68)),
        );
        if at(s + 0x74) != 0 {
            unit_vector(
                (s + 0x74) as usize,
                (COURSE_LISTS.spin_axis_x.addr()) as usize,
            );
            COURSE_LISTS.spin_angle.set_bits(x87::mul(at(s + 0x74), dt));
            axis_turn_matrix(COURSE_LISTS.turn_matrix[0].addr());
            square_matrix((COURSE_LISTS.turn_matrix[0].addr()) as usize);
            multiply_matrices(
                (s + 0x10) as usize,
                (COURSE_LISTS.turn_matrix[0].addr()) as usize,
                (MATRICES.turned_matrix[0].addr()) as usize,
            );
            copy(MATRICES.turned_matrix[0].addr(), s + 0x10, 16);
        }
        for (v, c) in [(0x54, 8), (0x50, 4), (0x58, 0xC)] {
            let d = mul2_to_int(at(s + v), dt, k65536) as u32;
            DRAW.scratch.set(d);
            put(s + c, at(s + c).wrapping_add(d));
        }
        put(
            s + 0x54,
            neg_product_plus(PHYSICS.gravity.bits(), dt, at(s + 0x54)),
        );
        GROUND_WORK.ground_x.set(at(s + 4));
        GROUND_WORK.ground_z.set(at(s + 0xC));
        COURSE.ground_face_last.set(at(s + 0x60));
        crate::physics::ground_probe_store::GROUND_PROBE
            .settle_mark
            .set(0x80);
        crate::physics::find_ground::find_ground()?;
        let face: u32 = COURSE.ground_slot.get();
        if face == u32::MAX {
            copy(RACE_WORK.slot_place[0].addr(), s + 4, 3);
            for o in [0x50, 0x54, 0x58, 0x68, 0x6C, 0x70] {
                put(s + o, 0);
            }
            return Ok(());
        }
        put(s + 0x60, face);
        let height: u32 = GROUND_WORK.ground_height.get();
        if (at(s + 8).wrapping_sub(height) as i32) >= 0 {
            return Ok(());
        }
        put(s + 8, height);
        let half = NUMBERS.half.bits();
        put(s + 0x50, x87::mul(at(s + 0x50), half));
        put(s + 0x54, neg_mul(at(s + 0x54), half));
        put(s + 0x58, x87::mul(at(s + 0x58), half));
        for o in [0x68, 0x6C, 0x70] {
            put(s + o, neg_mul(at(s + o), half));
        }
        if abs_compare(at(s + 0x54), NUMBERS.one.bits()) & 0x100 != 0 {
            copy(RACE_WORK.part_matrix_flat[0].addr(), s + 0x10, 12);
            let n = at(s) % 20;
            if (n.wrapping_sub(5) as i32) >= 0 && (n.wrapping_sub(9) as i32) < 0 {
                copy(RACE_WORK.part_matrix_still[0].addr(), s + 0x10, 12);
                return Ok(());
            }
        }
        DRAW.scratch.set(abs_of(at(s + 0x54)));
        Ok(())
    }
}
