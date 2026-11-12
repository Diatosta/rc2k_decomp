//! `physics::move_slot` (0x462485): one step of a loose object's slot (0x78 bytes of
//! `g_raceSlots`; +4 .. +0xC its place, whole numbers; +0x10 its matrix; +0x50 .. +0x58 its
//! speed; +0x68 .. +0x70 its spin; +0x60 its last face):
//! - the place kept (`slot_place`); +0x74 = the spin's length; when not 0, the matrix is turned
//!   by it: the axis (`scene::unit_vector` of +0x74 .. +0x7C, so two dwords past the slot, as
//!   the original has it) and angle (+0x74 times the step `step_time`) go through
//!   `physics::axis_turn_matrix` (`turn_matrix`), `scene::square_matrix`, then
//!   `scene::multiply_matrices` with the matrix into `g_turnedMatrix`, copied back;
//! - the place moved by the speed (times the step and 65536, rounded); y's speed less gravity
//!   (`gravity`) times the step;
//! - the ground under it (`physics::find_ground`, mask 0x80, last face +0x60): none, the place
//!   put back and the speed and spin cleared; else the face kept, and when below the ground
//!   (the sign of the difference) the slot is set on it, the speed halved (y's turned round),
//!   the spin halved and turned round; then, with |y's speed| below 1, the matrix from
//!   `part_matrix_flat` (12 dwords), or from `part_matrix_still` when the slot's part number
//!   mod 20 is 5 .. 8; else the scratch float = |y's speed| (and a call to an empty function).
//!
//! The slots (`race_slots`, eight) are loose objects that move on their own, fall, bounce on the
//! ground and spin; probably debris thrown off in a crash (the code does not say which object).
//!
//! Refuses (before anything) what `find_ground` would refuse where the slot moves to.
//!
//! Takes: the slot
//!
//! Caller: `physics::move_slots`.

use crate::effects::cell_x87::CellX87;
use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87::{abs_compare, abs_of, mul, mul2_to_int, negated};
use crate::math::x87_float as bh;
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::physics::step_families::SlotInputs;
use crate::race::work_store::{RaceSlot, RaceWork};
use crate::scene::matrix::multiply_matrices;
use crate::scene::matrix3::rounded_value;
use crate::scene::view_matrix::{square_matrix, unit_vector};
use crate::session::{Dword, Float};

/// A place (three whole numbers) copied.
fn copy_place(from: &[Dword; 3], to: &[Dword; 3]) {
    for (t, f) in to.iter().zip(from) {
        t.set(f.get());
    }
}

/// A matrix's floats copied.
fn copy_matrix(from: &[Float; 16], to: &[Float; 16]) {
    for (t, f) in to.iter().zip(from) {
        t.set(f.get());
    }
}

/// The first rows of one of the race work's matrices (floats held in dwords) into a matrix.
fn copy_held_matrix(from: &[Dword], to: &[Float; 16]) {
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

/// The slot `s`, number `number` of the table (the registers it leaves are dropped by its caller's `popa`).
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
        // From the spin's length on: two dwords past the slot, as the original has it (the
        // next slot's first two, or what follows the table).
        let first = core::mem::offset_of!(RaceWork, race_slots)
            + number * core::mem::size_of::<RaceSlot>()
            + core::mem::offset_of!(RaceSlot, spin_length);
        let past = |k: usize| crate::race::checkpoints_store::run_on(c.work, first, k).get();
        let axis = SpinAxis([s.spin_length.get().to_bits(), past(1), past(2)]);
        unit_vector(c.course, &axis, &c.course.lists.spin_axis);
        c.course.lists.spin_angle.set(x87::mul(&s.spin_length, dt));
        axis_turn_matrix(c.course, &c.course.lists.turn_matrix);
        square_matrix(c.course, &c.course.lists.turn_matrix);
        multiply_matrices(
            &s.matrix,
            &c.course.lists.turn_matrix,
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
    c.course.course.ground_face_last.set(s.car_word.get());
    c.probe.settle_mark.set(0x80);
    crate::physics::find_ground::find_ground(c.probe, c.ground, c.course, scene)?;
    let face: u32 = c.course.course.ground_slot.get();
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
