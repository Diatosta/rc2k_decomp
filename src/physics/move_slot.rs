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
use crate::math::cell_x87::PlainX87;
use crate::math::matrix3::rounded_value;
use crate::math::x87;
use crate::math::x87::{abs_compare, mul, mul2_to_int, negated};
use crate::math::x87_float as bh;
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::physics::step_families::SlotInputs;
use crate::race::work_store::{RaceSlot, RaceWork};
use crate::scene::matrix::{matrix_of, product};
use crate::scene::view_matrix::{square_matrix, unit_vector};

/// The first rows of one of the race work's matrices (floats held in dwords) into a matrix.
fn copy_held_matrix(from: &[u32], to: &mut [f32; 16]) {
    for (t, f) in to.iter_mut().zip(from) {
        *t = f32::from_bits(*f);
    }
}

/// The three dwords the spin's axis is made of: the spin's length and the two after it.
struct SpinAxis([u32; 3]);

impl crate::math::cells::Words for SpinAxis {
    fn bits(&self, n: usize) -> u32 {
        self.0[n]
    }
}

impl crate::math::cells::Cells for SpinAxis {
    fn set_bits(&self, _: usize, _: u32) {
        unreachable!("the spin axis is only read")
    }
}

/// The spin's axis for slot `number`, whose spin length is `spin_length` (the table's copy of the
/// slot is not yet updated): the length and the two dwords read on after it, which are the next
/// slot's `part` and first place number, or after the last slot `slot_count` and `part_fling`.
fn spin_axis(work: &RaceWork, number: usize, spin_length: f32) -> SpinAxis {
    let (second, third) = match work.race_slots.get(number + 1) {
        Some(next) => (next.part, next.place[0]),
        None => (work.slot_count, work.part_fling.to_bits()),
    };
    SpinAxis([spin_length.to_bits(), second, third])
}

/// The slot `s`, number `number` of the table, moved in place. `s` is a copy of the table's slot
/// (the caller puts it back, also after a refusal).
pub fn move_slot(c: &SlotInputs<'_>, s: &mut RaceSlot, number: usize) -> Result<(), Refused> {
    let dt = c.numbers.step_time.get();
    let k65536 = c.numbers.k65536.get();
    c.work.borrow_mut().slot_place = s.place;
    s.spin_length = rounded_value(crate::physics::vector_length::vector_length(&s.spin));
    // Tested as a dword: any bit set (a NaN too) means a spin.
    if s.spin_length.to_bits() != 0 {
        let axis = spin_axis(&c.work.borrow(), number, s.spin_length);
        unit_vector(c.course.reads(), &axis, &c.course.turn_work().spin_axis);
        c.course
            .turn_work()
            .spin_angle
            .set(x87::mul(s.spin_length, dt));
        axis_turn_matrix(c.course, &c.course.turn_work().turn_matrix);
        square_matrix(c.course.reads(), &c.course.turn_work().turn_matrix);
        let slot_matrix = matrix_of(&s.matrix);
        let turn_now = matrix_of(&c.course.turn_work().turn_matrix);
        let turned = {
            let mut kept = c.matrices.borrow_mut();
            kept.turned_matrix = product(&slot_matrix, &turn_now, kept.turned_matrix);
            kept.turned_matrix
        };
        s.matrix = turned;
    }
    for k in [1, 0, 2] {
        let d = mul2_to_int(s.motion[k], dt, k65536) as u32;
        s.place[k] = s.place[k].wrapping_add(d);
    }
    s.motion[1] = f32::from_bits(bh::st(
        -(c.tables.borrow().gravity.ld() * bh::ld(dt)) + bh::ld(s.motion[1].to_bits()),
    ));
    {
        let mut search = c.search.borrow_mut();
        search.x = s.place[0];
        search.z = s.place[2];
        search.last = s.car_word;
    }
    c.probe.borrow_mut().settle_mark = 0x80;
    crate::physics::find_ground::find_ground(c.probe, c.search, c.course)?;
    let face: u32 = c.search.borrow().slot;
    if face == u32::MAX {
        s.place = c.work.borrow().slot_place;
        s.motion = [0.0; 3];
        s.spin = [0.0; 3];
        return Ok(());
    }
    s.car_word = face;
    let height: u32 = c.search.borrow().height;
    if (s.place[1].wrapping_sub(height) as i32) >= 0 {
        return Ok(());
    }
    s.place[1] = height;
    let half = c.numbers.half.get();
    s.motion[0] = x87::mul(s.motion[0], half);
    s.motion[1] = negated(mul(s.motion[1], half));
    s.motion[2] = x87::mul(s.motion[2], half);
    for cell in &mut s.spin {
        *cell = negated(mul(*cell, half));
    }
    if abs_compare(s.motion[1], c.numbers.one) & 0x100 != 0 {
        copy_held_matrix(&c.work.borrow().part_matrix_flat[..12], &mut s.matrix);
        let n = s.part % 20;
        if (n.wrapping_sub(5) as i32) >= 0 && (n.wrapping_sub(9) as i32) < 0 {
            copy_held_matrix(&c.work.borrow().part_matrix_still[..12], &mut s.matrix);
            return Ok(());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two dwords past a slot's spin length are the next slot's first two; after the last
    /// slot they are `slot_count` and `part_fling` (a float), as the image lays them out.
    #[test]
    fn the_axis_reads_on_into_the_next_fields() {
        let mut work = RaceWork::new();
        work.race_slots[3].part = 0x1234_5678;
        work.race_slots[3].place[0] = 0x9ABC_DEF0;
        work.slot_count = 0x1357_9BDF;
        work.part_fling = f32::from_bits(0x4000_0001);
        assert_eq!(
            spin_axis(&work, 2, f32::from_bits(0x0BAD_F00D)).0,
            [0x0BAD_F00D, 0x1234_5678, 0x9ABC_DEF0]
        );
        assert_eq!(
            spin_axis(&work, 7, f32::from_bits(0x0102_0304)).0,
            [0x0102_0304, 0x1357_9BDF, 0x4000_0001]
        );
    }
}
