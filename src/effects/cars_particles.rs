//! `effects::cars_particles`: the effects' frame for each car, when the settings' +0x10 is 1. For
//! each of the settings' +0x5C cars (from the last; each 0x4C8 bytes from the settings' +0x60) it
//! sets the emitting car, the matrix's adjugate (+0x94, `scene::adjugate_matrix`), the motion
//! history (+0x58 .. +0x7C), the mean of the last four motions (+0x4C ..) with its length (+0x88,
//! rounded) and a rate from it (+0x90), the side factors (from the up vector and the vector at
//! +0x28, `x87::normalised3`, `x87::crossed_turned`) and two more values from the looks table by
//! the car's +4. While +0x30 counts down the mean and its length are 0. Then it runs
//! `effects::emit_car_particles` (unless +0x30 counts down, by one), `effects::emit_exhaust_smoke`,
//! `emit_twin_particle_first` and `emit_twin_particle_second`.
//!
//! Refuses null settings, more than 8 cars, a car's null matrix or vector, a car model (+4) of 34
//! or more, and the emitters' refusals; on a refusal nothing stays written (all it may write is put
//! back).

use crate::effects::emit_car_particles::emit_car_particles;
use crate::effects::emitters::*;
use crate::effects::owner::Effects;
use crate::effects::{emit_exhaust_smoke, emit_twin_particle_first, emit_twin_particle_second};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::memory::session::{read_bits, write_bits};
use crate::race::effects_settings_store::car_handle;
use crate::scene::adjugate_matrix::adjugate_matrix;

const MOST_CARS: u32 = 8;
// The looks by the car's model are `SplineHead.looks` (33 pairs of dwords, 8 bytes
// apart). The cars have 34 models: model 32's second dword is the spline faces' count,
// model 33 (the Clio's) reads that and the float after it (a slip kept as is):
// `SplineHead::look` reads past the table's end.
const MODELS: u32 = 34;

/// The cars' numbers, last first, worked out without writing: none when the settings' +0x10
/// is not 1.
fn cars(effects: &crate::effects::owner::Effects) -> Result<Vec<usize>, Refused> {
    let marks = effects.settings()?;
    Ok({
        if marks.other_marks_shown.get() != 1 {
            return Ok(Vec::new());
        }
        let count = marks.cars_copy.get() as i32;
        (0..count.clamp(0, MOST_CARS as i32 + 1) as u32)
            .rev()
            .map(|n| n as usize)
            .collect()
    })
}

/// Why the frame refuses, if it does (its own checks), worked out without writing.
fn refusal(
    effects: &crate::effects::owner::Effects,
    settings: u32,
) -> Result<Option<&'static str>, Refused> {
    let marks = effects.settings()?;
    Ok({
        if settings == 0 {
            return Ok(Some("null marks settings"));
        }
        if marks.other_marks_shown.get() == 1 && marks.cars_copy.get() as i32 > MOST_CARS as i32 {
            return Ok(Some("more than 8 cars"));
        }
        for n in cars(effects)? {
            let car = &marks.cars[n];
            if !effects.has_whole(car) || !effects.has_parts(car) {
                return Ok(Some("a car's null matrix or vector"));
            }
            if car.model.get() >= MODELS {
                return Ok(Some("a car model past the cars' 34"));
            }
        }
        None
    })
}

pub fn cars_particles(effects: &Effects) -> Result<(), Refused> {
    let paths = &effects.paths;
    let (particles, ring) = (&effects.particles, &effects.ring);
    let settings: u32 = effects.setup.settings.is_set.get();
    if let Some(why) = refusal(effects, settings)? {
        return Err(BadInput(why).into());
    }
    let count = cars(effects)?.len() as u32;
    for n in (0..count).rev() {
        effects
            .kinds
            .emitter
            .tag
            .set(bh::to_int(bh::int(n as i32) * bh::int(effects.kinds.emitter.tag_k.get())) as u32);
        let car = &effects.settings()?.cars[n as usize];
        effects.kinds.emitter.car.set(car_handle(n as usize));
        let whole = car.whole(effects.car_blocks()?)?;
        adjugate_matrix(whole, &car.matrix);
        for (to, from) in [(3, 2), (2, 1), (1, 0)] {
            for k in 0..3 {
                car.steps[to][k].set(car.steps[from][k].get());
            }
        }
        for k in 0..3 {
            car.steps[0][k].set(x87::sub(
                matrix_float(whole, 0x30 + 4 * k as u32)?,
                &car.last_place[k],
            ));
        }
        let mean_k = effects.setup.floats.quarter.get();
        for k in 0..3 {
            car.motion[k].set(x87::sum4_times(
                &car.steps[0][k],
                &car.steps[1][k],
                &car.steps[2][k],
                &car.steps[3][k],
                mean_k,
            ));
        }
        for (k, cell) in car.last_place.iter().enumerate().take(3) {
            cell.set(matrix_float(whole, 0x30 + 4 * k as u32)?);
        }
        let [k1, k2, k3] = read_bits(&effects.kinds.emitter.rate_k);
        let one = effects.setup.floats.one.get();
        let (length, rate) = x87::length_and_rate(car.motion.each_ref(), k1, k2, one, k3);
        car.pace.set(length as u32);
        car.rate.set(rate as u32);
        if car.reset_wait.get() != 0 {
            for cell in &car.motion {
                cell.set(0.0);
            }
            car.pace.set(0);
        }
        let parts = car.parts(effects.car_blocks()?)?;
        let up = read_bits(&effects.kinds.emitter.up);
        let across = [
            x87::product_difference(
                up[1],
                matrix_float(parts, 8)?,
                up[2],
                matrix_float(parts, 4)?,
            ),
            x87::product_difference(
                up[2],
                matrix_float(parts, 0)?,
                up[0],
                matrix_float(parts, 8)?,
            ),
            x87::product_difference(
                up[0],
                matrix_float(parts, 4)?,
                up[1],
                matrix_float(parts, 0)?,
            ),
        ];
        let across = x87::normalised3(across.map(f32::to_bits), one);
        let across = across.map(f32::to_bits);
        write_bits(&effects.kinds.emitter.across, across);
        let b = x87::crossed_turned(
            across,
            car.parts(effects.car_blocks()?)?,
            car.whole(effects.car_blocks()?)?,
        );
        write_bits(&effects.kinds.emitter.side_b, b);
        let a = [
            x87::product_difference(b[1], across[2], b[2], across[1]),
            x87::product_difference(b[2], across[0], b[0], across[2]),
            x87::product_difference(b[0], across[1], b[1], across[0]),
        ];
        write_bits(&effects.kinds.emitter.side_a, a.map(f32::to_bits));
        let model = car.model.get();
        effects.kinds.emitter.looks[0].set(paths.spline_head.look(2 * model as usize));
        effects.kinds.emitter.looks[1].set(paths.spline_head.look(2 * model as usize + 2));
        if car.reset_wait.get() != 0 {
            car.reset_wait.set(car.reset_wait.get().wrapping_sub(1));
        } else {
            emit_car_particles(effects, car)?;
        }
        emit_exhaust_smoke::emit_exhaust_smoke(paths, particles, ring, car, effects)?;
        emit_twin_particle_first::emit_twin_particle_first(effects, car)?;
        emit_twin_particle_second::emit_twin_particle_second(effects, car)?;
    }
    Ok(())
}
