//! One step of a loose object's slot (0x78 bytes of `race_slots`: place as whole numbers, matrix,
//! speed, spin, last face):
//! - the place is kept (`slot_place`); the spin's length is stored; when not 0, the matrix is
//!   turned by it: the axis (`scene::unit_vector` read two dwords past the slot's spin) and angle
//!   (the length times the step `step_time`) go through `physics::axis_turn_matrix`
//!   (`turn_matrix`), `scene::square_matrix`, then `scene::multiply_matrices` with the matrix into
//!   `turned_matrix`, copied back;
//! - the place is moved by the speed (times the step and 65536, rounded); y's speed less gravity
//!   (`gravity`) times the step;
//! - the ground under it (`physics::find_ground`, mask 0x80, last face from the slot): none, the
//!   place is put back and the speed and spin cleared; otherwise the face is kept, and when below
//!   the ground the slot is set on it, the speed halved (y's turned round), the spin halved and
//!   turned round; then, with |y's speed| below 1, the matrix from `part_matrix_flat` (12 dwords),
//!   or from `part_matrix_still` when the slot's part number mod 20 is 5 .. 8; otherwise the
//!   scratch float = |y's speed|.
//!
//! The slots (`race_slots`, eight) are loose objects that move on their own, fall, bounce on the
//! ground and spin; probably debris thrown off in a crash (the code does not say which object).
//!
//! Refuses (before anything) what `find_ground` would refuse where the slot moves to.

use crate::errors::Refused;
use crate::math::cell_x87::CellX87;
use crate::math::matrix3::rounded_value;
use crate::math::x87;
use crate::math::x87::{abs_compare, abs_of, mul, mul2_to_int, negated};
use crate::math::x87_float as bh;
use crate::memory::session::{FloatLike, PlainDword, PlainFloat};
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::physics::step_families::SlotInputs;
use crate::race::work_store::{RaceSlot, RaceWork};
use crate::scene::matrix::multiply_matrices;
use crate::scene::view_matrix::{square_matrix, unit_vector};

/// A place (three whole numbers) copied.
fn copy_place(from: &[PlainDword; 3], to: &[PlainDword; 3]) {
    for (t, f) in to.iter().zip(from) {
        t.set(f.get());
    }
}

/// A matrix's floats copied.
fn copy_matrix(from: &[impl FloatLike; 16], to: &[impl FloatLike; 16]) {
    for (t, f) in to.iter().zip(from) {
        t.set(f.get());
    }
}

/// The first rows of one of the race work's matrices (floats held in dwords) into a matrix.
fn copy_held_matrix(from: &[PlainDword], to: &[PlainFloat; 16]) {
    for (t, f) in to.iter().zip(from) {
        t.set(f32::from_bits(f.get()));
    }
}

/// The three dwords the spin's axis is made of: the spin's length and the two after it.
struct SpinAxis([u32; 3]);

impl crate::math::cells::Cells for SpinAxis {
    fn bits(&self, n: usize) -> u32 {
        self.0[n]
    }
    fn set_bits(&self, _: usize, _: u32) {
        unreachable!("the spin axis is only read")
    }
}

/// The spin's axis for slot `number`: its spin's length and the two dwords read on after it, which
/// are the next slot's `part` and first place number, or after the last slot `slot_count` and
/// `part_fling`.
fn spin_axis(work: &RaceWork, number: usize) -> SpinAxis {
    let slot = &work.race_slots[number];
    let (second, third) = match work.race_slots.get(number + 1) {
        Some(next) => (next.part.get(), next.place[0].get()),
        None => (work.slot_count.get(), work.part_fling.get().to_bits()),
    };
    SpinAxis([slot.spin_length.get().to_bits(), second, third])
}

/// The slot `s`, number `number` of the table.
pub fn move_slot(
    c: &SlotInputs<'_>,
    s: &RaceSlot,
    number: usize,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    let dt = c.tables.step_time.get();
    let k65536 = c.numbers.k65536.get();
    copy_place(&s.place, &c.work.slot_place);
    s.spin_length
        .set(rounded_value(crate::physics::vector_length::vector_length(
            &s.spin,
        )));
    // Tested as a dword: any bit set (a NaN too) means a spin.
    if s.spin_length.get().to_bits() != 0 {
        let axis = spin_axis(c.work, number);
        unit_vector(c.course.reads(), &axis, &c.course.lists.get().spin_axis);
        c.course
            .lists
            .get()
            .spin_angle
            .set(x87::mul(&s.spin_length, dt));
        axis_turn_matrix(c.course, &c.course.lists.get().turn_matrix);
        square_matrix(c.course.reads(), &c.course.lists.get().turn_matrix);
        multiply_matrices(
            &s.matrix,
            &c.course.lists.get().turn_matrix,
            &c.matrices.turned_matrix,
        );
        copy_matrix(&c.matrices.turned_matrix, &s.matrix);
    }
    for k in [1, 0, 2] {
        let d = mul2_to_int(&s.motion[k], dt, k65536) as u32;
        drawing.scene.get().scratch.set(d);
        s.place[k].set(s.place[k].get().wrapping_add(d));
    }
    s.motion[1].st(-(c.tables.gravity.ld() * bh::ld(dt)) + s.motion[1].ld());
    c.ground.ground_x.set(s.place[0].get());
    c.ground.ground_z.set(s.place[2].get());
    c.course.course.get().ground_face_last.set(s.car_word.get());
    c.probe.settle_mark.set(0x80);
    crate::physics::find_ground::find_ground(c.probe, c.ground, c.course)?;
    let face: u32 = c.course.course.get().ground_slot.get();
    if face == u32::MAX {
        copy_place(&c.work.slot_place, &s.place);
        for cell in s.motion.iter().chain(&s.spin) {
            cell.set(0.0);
        }
        return Ok(());
    }
    s.car_word.set(face);
    let height: u32 = c.ground.ground_height.get();
    if (s.place[1].get().wrapping_sub(height) as i32) >= 0 {
        return Ok(());
    }
    s.place[1].set(height);
    let half = c.numbers.half.get();
    s.motion[0].set(x87::mul(&s.motion[0], half));
    s.motion[1].set(negated(mul(&s.motion[1], half)));
    s.motion[2].set(x87::mul(&s.motion[2], half));
    for cell in &s.spin {
        cell.set(negated(mul(cell, half)));
    }
    if abs_compare(&s.motion[1], &c.numbers.one) & 0x100 != 0 {
        copy_held_matrix(&c.work.part_matrix_flat[..12], &s.matrix);
        let n = s.part.get() % 20;
        if (n.wrapping_sub(5) as i32) >= 0 && (n.wrapping_sub(9) as i32) < 0 {
            copy_held_matrix(&c.work.part_matrix_still[..12], &s.matrix);
            return Ok(());
        }
    }
    drawing
        .scene
        .get()
        .scratch
        .set(abs_of(&s.motion[1]).to_bits());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two dwords past a slot's spin length are the next slot's first two; after the last
    /// slot they are `slot_count` and `part_fling` (a float), as the image lays them out.
    #[test]
    fn the_axis_reads_on_into_the_next_fields() {
        let _g = crate::test_support::test_lock();
        let work = crate::test_support::own::race().shared.work.get();
        let saved = (
            work.race_slots[2].spin_length.get(),
            work.race_slots[3].part.get(),
            work.race_slots[3].place[0].get(),
            work.race_slots[7].spin_length.get(),
            work.slot_count.get(),
            work.part_fling.get(),
        );
        work.race_slots[2]
            .spin_length
            .set(f32::from_bits(0x0BAD_F00D));
        work.race_slots[3].part.set(0x1234_5678);
        work.race_slots[3].place[0].set(0x9ABC_DEF0);
        work.race_slots[7]
            .spin_length
            .set(f32::from_bits(0x0102_0304));
        work.slot_count.set(0x1357_9BDF);
        work.part_fling.set(f32::from_bits(0x4000_0001));
        assert_eq!(
            spin_axis(work, 2).0,
            [0x0BAD_F00D, 0x1234_5678, 0x9ABC_DEF0]
        );
        assert_eq!(
            spin_axis(work, 7).0,
            [0x0102_0304, 0x1357_9BDF, 0x4000_0001]
        );
        work.race_slots[2].spin_length.set(saved.0);
        work.race_slots[3].part.set(saved.1);
        work.race_slots[3].place[0].set(saved.2);
        work.race_slots[7].spin_length.set(saved.3);
        work.slot_count.set(saved.4);
        work.part_fling.set(saved.5);
    }
}
