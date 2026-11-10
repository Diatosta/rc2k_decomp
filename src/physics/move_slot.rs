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
//! Takes: the slot
//!
//! Caller: `physics::move_slots`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_run::{abs_compare, abs_of, mul2_to_int, neg_mul, neg_product_plus};
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::race::owner::Race;
use crate::race::work_store::RaceSlot;
use crate::scene::matrix::multiply_matrices;
use crate::scene::matrix3::rounded;
use crate::scene::view_matrix::{square_matrix, unit_vector};
use crate::session::Cell32;

/// `from`'s first cells into `to`'s.
fn copy<A: Cell32, B: Cell32>(from: &[A], to: &[B]) {
    for (t, f) in to.iter().zip(from) {
        t.set_bits(f.bits());
    }
}

/// The slot `s` (the registers it leaves are dropped by its caller's `popa`).
pub fn move_slot(
    race: &Race,
    s: &RaceSlot,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    let dt = race.shared.physics_tables.step_time.bits();
    let k65536 = race.course.numbers.k65536.bits();
    copy(&s.place, &race.shared.work.slot_place);
    s.spin_length
        .set_bits(rounded(crate::physics::vector_length::vector_length(
            &s.spin,
        )));
    if s.spin_length.bits() != 0 {
        // From the spin's length on: two dwords past the slot, as the original has it (the
        // next slot's first, or what follows the table), so by address.
        let past = crate::pointers::GamePtr::to(&s.spin_length).unbounded();
        unit_vector(&race.course, &past, &race.course.lists.spin_axis);
        race.course
            .lists
            .spin_angle
            .set_bits(x87::mul(s.spin_length.bits(), dt));
        axis_turn_matrix(&race.course, &race.course.lists.turn_matrix);
        square_matrix(&race.course, &race.course.lists.turn_matrix);
        multiply_matrices(
            &s.matrix,
            &race.course.lists.turn_matrix,
            &race.frame.matrices.turned_matrix,
        );
        copy(&race.frame.matrices.turned_matrix, &s.matrix);
    }
    for k in [1, 0, 2] {
        let d = mul2_to_int(s.motion[k].bits(), dt, k65536) as u32;
        drawing.scene.scratch.set(d);
        s.place[k].set(s.place[k].get().wrapping_add(d));
    }
    s.motion[1].set_bits(neg_product_plus(
        race.shared.physics_tables.gravity.bits(),
        dt,
        s.motion[1].bits(),
    ));
    race.physics.ground.ground_x.set(s.place[0].get());
    race.physics.ground.ground_z.set(s.place[2].get());
    race.course.course.ground_face_last.set(s.car_word.get());
    race.physics.ground_probe.settle_mark.set(0x80);
    crate::physics::find_ground::find_ground(race, scene)?;
    let face: u32 = race.course.course.ground_slot.get();
    if face == u32::MAX {
        copy(&race.shared.work.slot_place, &s.place);
        for cell in s.motion.iter().chain(&s.spin) {
            cell.set_bits(0);
        }
        return Ok(());
    }
    s.car_word.set(face);
    let height: u32 = race.physics.ground.ground_height.get();
    if (s.place[1].get().wrapping_sub(height) as i32) >= 0 {
        return Ok(());
    }
    s.place[1].set(height);
    let half = race.course.numbers.half.bits();
    s.motion[0].set_bits(x87::mul(s.motion[0].bits(), half));
    s.motion[1].set_bits(neg_mul(s.motion[1].bits(), half));
    s.motion[2].set_bits(x87::mul(s.motion[2].bits(), half));
    for cell in &s.spin {
        cell.set_bits(neg_mul(cell.bits(), half));
    }
    if abs_compare(s.motion[1].bits(), race.course.numbers.one.bits()) & 0x100 != 0 {
        copy(&race.shared.work.part_matrix_flat[..12], &s.matrix);
        let n = s.part.get() % 20;
        if (n.wrapping_sub(5) as i32) >= 0 && (n.wrapping_sub(9) as i32) < 0 {
            copy(&race.shared.work.part_matrix_still[..12], &s.matrix);
            return Ok(());
        }
    }
    drawing.scene.scratch.set(abs_of(s.motion[1].bits()));
    Ok(())
}
