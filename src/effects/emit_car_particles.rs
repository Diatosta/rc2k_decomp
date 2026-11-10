//! `effects::emit_car_particles` (0x4C9E2E): a car's particles for the frame. First the
//! places of its four wheel records (+0x3C .. of the records at +0xD4, +0x11C, +0x164,
//! +0x1AC): from the object's whole numbers +0xC (z, by `flt_9354EE`; front +0x118, rear
//! +0x160), +0x10 (the other z, by `flt_9354F2`: +0x1A8, +0x1F0), +0x14 (x, by
//! `flt_9354F6` plus `flt_9354FA`: right +0x158, +0x1E8, negated left +0x110, +0x1A0) and
//! +0x18 (y, by `flt_9354FE` less `flt_935502` and `flt_935506`: all four), each turned
//! and placed by the car's matrix (`scene::rotate_point`); `dword_935476` |the first x|
//! and `dword_93547A` the two z's sizes by `flt_765158`, rounded. Then
//! `effects::emit_wheel_particles` for each wheel (`dword_935416` 0 .. 3; before the rear
//! ones the side factors `flt_93545E` .. the matrix's first row, `flt_93543A` .. its third
//! negated), the factors set from `flt_93550A` .., and `effects::emit_body_particles` for
//! the record at +0x1F4.
//!
//! Takes: the car object
//!
//! Caller: `effects::cars_particles` (0x4C9E0C). Refuses a null object or matrix and the emitters'
//! refusals; on a refusal nothing stays written (all it may write is put back).

use crate::effects::emit_body_particles::emit_body_particles;
use crate::effects::emit_wheel_particles::emit_wheel_particles;
use crate::effects::emitters::*;
use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::marks_settings_store::MarkCar;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::session::{read_bits, write_bits};

/// `o` the car's object (the original's esi).
pub fn emit_car_particles(effects: &Effects, o: &'static MarkCar) -> Result<(), Refused> {
    o.whole()?;
    let k = read_bits(&effects.kinds.emitter.place_k);
    let place = |wheel: usize, axis: usize| &o.records[wheel].place[axis];
    let z1 = x87::int_times(o.axles[0].get() as i32, k[0]);
    place(0, 2).set_bits(z1);
    place(1, 2).set_bits(z1);
    let z2 = x87::int_times(o.axles[1].get() as i32, k[1]);
    place(2, 2).set_bits(z2);
    place(3, 2).set_bits(z2);
    let (x, left) = x87::int_times_plus_and_negated(o.axles[2].get() as i32, k[2], k[3]);
    place(1, 0).set_bits(x);
    place(3, 0).set_bits(x);
    place(0, 0).set_bits(left);
    place(2, 0).set_bits(left);
    let y = x87::int_times_less_less(o.axles[3].get() as i32, k[4], k[5], k[6]);
    for wheel in [1, 3, 0, 2] {
        place(wheel, 1).set_bits(y);
    }
    effects.kinds.emitter.looks[0]
        .set(x87::to_int(o.records[0].place[0].bits() & 0x7FFF_FFFF) as u32);
    effects.kinds.emitter.looks[1].set(x87::abs_sum_times_to_int(
        o.records[0].place[2].bits(),
        o.records[2].place[2].bits(),
        effects.setup.floats.half.bits(),
    ) as u32);
    let m = o.whole()?;
    for wheel in [0, 1, 2, 3] {
        let at = &o.records[wheel].place;
        let turned = rotate_point(
            m,
            [
                extended(at[0].bits()),
                extended(at[1].bits()),
                extended(at[2].bits()),
            ],
        );
        for (cell, v) in at.iter().zip(turned) {
            cell.set_bits(rounded(v));
        }
    }
    for n in 0..4 {
        if n == 2 {
            write_bits(
                &effects.kinds.emitter.side_a,
                [matrix_cell(m, 0)?, matrix_cell(m, 4)?, matrix_cell(m, 8)?],
            );
            write_bits(
                &effects.kinds.emitter.side_b,
                crate::effects::try_from_fn(|i__| {
                    let f = [0x20, 0x24, 0x28][i__];
                    Ok(matrix_cell(m, f)? ^ 0x8000_0000)
                })?,
            );
        }
        effects.kinds.emitter.record.set(n as u32);
        emit_wheel_particles(effects, &o.records[n], o)?;
    }
    let sides: [u32; 6] =
        std::array::from_fn(|n| x87::loaded(effects.kinds.emitter.body_k[n].bits()));
    write_bits(
        &effects.kinds.emitter.side_a,
        [sides[0], sides[1], sides[2]],
    );
    write_bits(
        &effects.kinds.emitter.side_b,
        [sides[3], sides[4], sides[5]],
    );
    emit_body_particles(effects, &o.records[4], o)?;
    Ok(())
}
