//! `effects::emit_car_particles`: a car's particles for the frame. First the places of its four
//! wheel records (their `place`) are worked out from the object's four `axles` numbers (z, the
//! other z, x and y, each by a factor and offsets),
//! turned and placed by the car's matrix (`scene::rotate_point`), and the first x's size and the
//! two z's sizes are kept, by a factor, rounded. Then `effects::emit_wheel_particles` runs for each
//! wheel (numbered 0 .. 3; before the rear ones the side factors are the matrix's first row and its
//! third negated) with the factors set from the constants, and `effects::emit_body_particles` for
//! the body's record.
//!
//! Refuses a null object or matrix and the emitters' refusals; on a refusal nothing stays written
//! (all it may write is put back).

use crate::effects::emit_body_particles::emit_body_particles;
use crate::effects::emit_wheel_particles::emit_wheel_particles;
use crate::effects::emitters::*;
use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::math::x87_float as bh;

use crate::race::effects_settings_store::CarEffects;
use crate::scene::rotate_point::rotate_point;

/// `o` is the car's object.
pub fn emit_car_particles(effects: &Effects, o: &CarEffects) -> Result<(), Refused> {
    o.whole(effects.car_parts()?)?;
    let k = effects.kinds.emitter.borrow().place_k.map(f32::to_bits);
    let place = |wheel: usize, axis: usize| &o.records[wheel].place[axis];
    let z1 = x87::int_times(o.axles[0].get() as i32, k[0]);
    place(0, 2).set(z1);
    place(1, 2).set(z1);
    let z2 = x87::int_times(o.axles[1].get() as i32, k[1]);
    place(2, 2).set(z2);
    place(3, 2).set(z2);
    let (x, left) = x87::int_times_plus_and_negated(o.axles[2].get() as i32, k[2], k[3]);
    place(1, 0).set(x);
    place(3, 0).set(x);
    place(0, 0).set(left);
    place(2, 0).set(left);
    let y = bh::stf(bh::int(o.axles[3].get() as i32) * bh::ld(k[4]) - bh::ld(k[5]) - bh::ld(k[6]));
    for wheel in [1, 3, 0, 2] {
        place(wheel, 1).set(y);
    }
    effects.kinds.emitter.borrow_mut().looks[0] =
        x87::to_int(o.records[0].place[0].get().abs()) as u32;
    effects.kinds.emitter.borrow_mut().looks[1] = bh::to_int(
        (bh::ld(o.records[0].place[2].get()).abs() + bh::ld(o.records[2].place[2].get()).abs())
            * bh::ld(effects.setup.floats.half),
    ) as u32;
    let m = o.whole(effects.car_parts()?)?;
    for wheel in [0, 1, 2, 3] {
        let at = &o.records[wheel].place;
        let turned = rotate_point(
            m,
            [
                extended(at[0].get()),
                extended(at[1].get()),
                extended(at[2].get()),
            ],
        );
        for (cell, v) in at.iter().zip(turned) {
            cell.set(rounded_value(v));
        }
    }
    for n in 0..4 {
        if n == 2 {
            let side_a = [matrix_cell(m, 0)?, matrix_cell(m, 4)?, matrix_cell(m, 8)?];
            let side_b: [u32; 3] = crate::effects::try_from_fn(|i__| {
                let f = [0x20, 0x24, 0x28][i__];
                Ok(matrix_cell(m, f)? ^ 0x8000_0000)
            })?;
            let mut vars = effects.kinds.emitter.borrow_mut();
            vars.side_a = side_a.map(f32::from_bits);
            vars.side_b = side_b.map(f32::from_bits);
        }
        effects.kinds.emitter.borrow_mut().record = n as u32;
        emit_wheel_particles(effects, &o.records[n], o)?;
    }
    {
        let mut vars = effects.kinds.emitter.borrow_mut();
        let sides: [f32; 6] = std::array::from_fn(|n| x87::loaded(vars.body_k[n]));
        vars.side_a = [sides[0], sides[1], sides[2]];
        vars.side_b = [sides[3], sides[4], sides[5]];
    }
    emit_body_particles(effects, &o.records[4], o)?;
    Ok(())
}
