//! `effects::cars_particles` (0x4C9AF6): the marks' frame for each car, when the settings'
//! +0x10 is 1: for each of the settings' +0x5C cars (from the last; each 0x4C8 bytes from
//! the settings' +0x60), `dword_93547E` its number by `dword_9354DE` (`fimul`),
//! `g_marksCar` it, its matrix's adjugate at +0x94 (`scene::adjugate_matrix`); its motion
//! history moved on (+0x58 .. +0x60 into +0x64, into +0x70, into +0x7C), the new one the
//! matrix's place less the last (+0x40 ..), the mean of the four by `flt_7651BC` at +0x4C
//! .., the place kept; the mean's length (+0x88, `fist`) and a rate from it (+0x90: 1 - 1
//! / (length by `flt_9354E2` plus `flt_9354E6`), by `flt_9354EA`); while +0x30 counts
//! down, the mean and length 0. Then the side factors: `flt_935452` .. the cross product
//! of `flt_935446` .. and the vector at +0x28, made of length 1 (`x87::normalised3`);
//! `flt_93543A` .. that crossed with the vector again, turned by the matrix transposed
//! (`x87::crossed_turned`); `flt_93545E` .. the cross product of the two; `dword_935476` /
//! `dword_93547A` from `dword_7694AA` by the car's +4; then `effects::emit_car_particles`
//! (unless +0x30 counts down, by one) and `effects::emit_particle_13`, `emit_particle_19`,
//! `emit_particle_22`.
//!
//! Works on `g_marksSettings`.
//!
//! Caller: `effects::draw_frame` (0x4C4006). Refuses null settings, more than 8 cars, a car's null
//! matrix or vector, a car model (+4) of 34 or more, and the emitters' refusals; on a
//! refusal nothing stays written (all it may write is put back).

use crate::effects::emit_car_particles::emit_car_particles;
use crate::effects::emitters::*;
use crate::effects::owner::Effects;
use crate::effects::{emit_particle_13, emit_particle_19, emit_particle_22};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::effects_settings_store::car_handle;
use crate::scene::adjugate_matrix::adjugate_matrix;
use crate::session::{read_bits, write_bits};

const MOST_CARS: u32 = 8;
// The looks by the car's model are `SPLINE_HEAD.looks` (dword_7694AA, dword_7694B2: 8 bytes
// apart, 33 pairs to 0x7695B2). The cars have 34 models: model 32's second dword is
// `g_marksSplineFaces`, model 33 (the Clio's) reads `g_marksSplineFaces` and `flt_7695BA`,
// as the original does (`notes/bugs.md`): `SplineHead::look` reads past the table's end.
const MODELS: u32 = 34;

/// The cars' numbers, last first, worked out without writing: none when the settings' +0x10
/// is not 1.
fn cars() -> Result<Vec<usize>, Refused> {
    let marks = crate::effects::setup::effects_settings()?;
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

/// Why the Rust version refuses, if it does (its own checks), worked out without writing.
fn refusal(settings: u32) -> Result<Option<&'static str>, Refused> {
    let marks = crate::effects::setup::effects_settings()?;
    Ok({
        if settings == 0 {
            return Ok(Some("null marks settings"));
        }
        if marks.other_marks_shown.get() == 1 && marks.cars_copy.get() as i32 > MOST_CARS as i32 {
            return Ok(Some("more than 8 cars"));
        }
        for n in cars()? {
            let car = &marks.cars[n];
            if !car.has_whole() || !car.has_parts() {
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
    let (paths, masks) = (&effects.paths, &effects.masks);
    let (particles, ring) = (&effects.particles, &effects.ring);
    let settings: u32 = effects.setup.settings.is_set.get();
    if let Some(why) = refusal(settings)? {
        return Err(BadInput(why).into());
    }
    let count = cars()?.len() as u32;
    for n in (0..count).rev() {
        effects.kinds.emitter.tag.set(x87::int_times_int_to_int(
            n as i32,
            effects.kinds.emitter.tag_k.get() as i32,
        ) as u32);
        let car = &crate::effects::setup::effects_settings()?.cars[n as usize];
        effects.kinds.emitter.car.set(car_handle(n as usize));
        let whole = car.whole()?;
        adjugate_matrix(whole, &car.matrix);
        for (to, from) in [(3, 2), (2, 1), (1, 0)] {
            for k in 0..3 {
                car.steps[to][k].set_bits(car.steps[from][k].bits());
            }
        }
        for k in 0..3 {
            car.steps[0][k].set_bits(x87::sub(
                matrix_cell(whole, 0x30 + 4 * k as u32)?,
                car.last_place[k].bits(),
            ));
        }
        let mean_k: u32 = effects.setup.floats.quarter.bits();
        for k in 0..3 {
            car.motion[k].set_bits(x87::sum4_times(
                car.steps[0][k].bits(),
                car.steps[1][k].bits(),
                car.steps[2][k].bits(),
                car.steps[3][k].bits(),
                mean_k,
            ));
        }
        for k in 0..3 {
            car.last_place[k].set_bits(matrix_cell(whole, 0x30 + 4 * k as u32)?);
        }
        let [k1, k2, k3] = read_bits(&effects.kinds.emitter.rate_k);
        let one: u32 = effects.setup.floats.one.bits();
        let (length, rate) = x87::length_and_rate(
            [
                car.motion[0].bits(),
                car.motion[1].bits(),
                car.motion[2].bits(),
            ],
            k1,
            k2,
            one,
            k3,
        );
        car.pace.set(length as u32);
        car.rate.set(rate as u32);
        if car.reset_wait.get() != 0 {
            for cell in &car.motion {
                cell.set_bits(0);
            }
            car.pace.set(0);
        }
        let parts = car.parts()?;
        let up = read_bits(&effects.kinds.emitter.up);
        let across = [
            x87::product_difference(up[1], matrix_cell(parts, 8)?, up[2], matrix_cell(parts, 4)?),
            x87::product_difference(up[2], matrix_cell(parts, 0)?, up[0], matrix_cell(parts, 8)?),
            x87::product_difference(up[0], matrix_cell(parts, 4)?, up[1], matrix_cell(parts, 0)?),
        ];
        let across = x87::normalised3(across, one);
        write_bits(&effects.kinds.emitter.across, across);
        let b = x87::crossed_turned(across, car.parts()?, car.whole()?);
        write_bits(&effects.kinds.emitter.side_b, b);
        let a = [
            x87::product_difference(b[1], across[2], b[2], across[1]),
            x87::product_difference(b[2], across[0], b[0], across[2]),
            x87::product_difference(b[0], across[1], b[1], across[0]),
        ];
        write_bits(&effects.kinds.emitter.side_a, a);
        let model = car.model.get();
        effects.kinds.emitter.looks[0].set(paths.spline_head.look(2 * model as usize));
        effects.kinds.emitter.looks[1].set(paths.spline_head.look(2 * model as usize + 2));
        if car.reset_wait.get() != 0 {
            car.reset_wait.set(car.reset_wait.get().wrapping_sub(1));
        } else {
            emit_car_particles(effects, car)?;
        }
        emit_particle_13::emit_particle_13(paths, particles, ring, car, effects)?;
        emit_particle_19::emit_particle_19(masks, particles, ring, car)?;
        emit_particle_22::emit_particle_22(masks, particles, ring, car)?;
    }
    Ok(())
}
