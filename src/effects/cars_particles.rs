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
//! In:  nothing (`g_marksSettings`)
//! Out: ebp the settings when nothing is done, else as the last car leaves it (see
//!      `cars`); the rest as the last car leaves them
//!
//! Caller: `sub_4C3EEE` (0x4C4006). Refuses null settings, more than 8 cars, a car's null
//! matrix or vector, a car model (+4) of 34 or more, and the emitters' refusals; on a
//! refusal nothing stays written (all it may write is put back).

use crate::effects::emit_car_particles::emit_car_particles;
use crate::effects::emitters::*;
use crate::effects::splines::SPLINE_HEAD;
use crate::effects::tuning::MARKS_FLOATS;
use crate::effects::{emit_particle_13, emit_particle_19, emit_particle_22};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::marks_settings_store::car_handle;
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
    let marks = crate::effects::setup::marks()?;
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
    let marks = crate::effects::setup::marks()?;
    Ok({
        if settings == 0 {
            return Ok(Some("null marks settings"));
        }
        if marks.other_marks_shown.get() == 1 && marks.cars_copy.get() as i32 > MOST_CARS as i32 {
            return Ok(Some("more than 8 cars"));
        }
        for n in cars()? {
            let car = &marks.cars[n];
            if !car.has_block(0x24)? || !car.has_block(0x28)? {
                return Ok(Some("a car's null matrix or vector"));
            }
            if car.word(4)? >= MODELS {
                return Ok(Some("a car model past the cars' 34"));
            }
        }
        None
    })
}

/// The Rust version (see the header).
pub fn cars_particles() -> Result<(), Refused> {
    let settings: u32 = crate::effects::setup::SETTINGS.is_set.get();
    if let Some(why) = refusal(settings)? {
        return Err(BadInput(why).into());
    }
    let count = cars()?.len() as u32;
    for n in (0..count).rev() {
        EMITTER
            .tag
            .set(x87::int_times_int_to_int(n as i32, EMITTER.tag_k.get() as i32) as u32);
        let car = &crate::effects::setup::marks()?.cars[n as usize];
        EMITTER.car.set(car_handle(n as usize));
        let whole = car.whole()?;
        adjugate_matrix(whole, &car.head[0x94 / 4..]);
        for (to, from) in [(0x7C, 0x70), (0x70, 0x64), (0x64, 0x58)] {
            let v = [car.word(from)?, car.word(from + 4)?, car.word(from + 8)?];
            for (k, x) in v.iter().enumerate() {
                car.put(to + 4 * k as u32, *x)?;
            }
        }
        for k in 0..3 {
            let k4 = 4 * k;
            car.put(
                0x58 + k4,
                x87::sub(matrix_cell(whole, 0x30 + k4)?, car.word(0x40 + k4)?),
            )?;
        }
        let mean_k: u32 = MARKS_FLOATS.quarter.bits();
        for k in 0..3 {
            let k4 = 4 * k;
            car.put(
                0x4C + k4,
                x87::sum4_times(
                    car.word(0x58 + k4)?,
                    car.word(0x64 + k4)?,
                    car.word(0x70 + k4)?,
                    car.word(0x7C + k4)?,
                    mean_k,
                ),
            )?;
        }
        car.put(0x40, matrix_cell(whole, 0x30)?)?;
        car.put(0x44, matrix_cell(whole, 0x34)?)?;
        car.put(0x48, matrix_cell(whole, 0x38)?)?;
        let [k1, k2, k3] = read_bits(&EMITTER.rate_k);
        let one: u32 = MARKS_FLOATS.one.bits();
        let (length, rate) = x87::length_and_rate(
            [car.word(0x4C)?, car.word(0x50)?, car.word(0x54)?],
            k1,
            k2,
            one,
            k3,
        );
        car.put(0x88, length as u32)?;
        car.put(0x90, rate as u32)?;
        if car.word(0x30)? != 0 {
            for o in [0x4C, 0x50, 0x54, 0x88] {
                car.put(o, 0)?;
            }
        }
        let parts = car.parts()?;
        let up = read_bits(&EMITTER.up);
        let across = [
            x87::product_difference(up[1], matrix_cell(parts, 8)?, up[2], matrix_cell(parts, 4)?),
            x87::product_difference(up[2], matrix_cell(parts, 0)?, up[0], matrix_cell(parts, 8)?),
            x87::product_difference(up[0], matrix_cell(parts, 4)?, up[1], matrix_cell(parts, 0)?),
        ];
        let across = x87::normalised3(across, one);
        write_bits(&EMITTER.across, across);
        let b = x87::crossed_turned(across, car.parts()?, car.whole()?);
        write_bits(&EMITTER.side_b, b);
        let a = [
            x87::product_difference(b[1], across[2], b[2], across[1]),
            x87::product_difference(b[2], across[0], b[0], across[2]),
            x87::product_difference(b[0], across[1], b[1], across[0]),
        ];
        write_bits(&EMITTER.side_a, a);
        let model = car.word(4)?;
        EMITTER.looks[0].set(SPLINE_HEAD.look(2 * model as usize));
        EMITTER.looks[1].set(SPLINE_HEAD.look(2 * model as usize + 2));
        if car.word(0x30)? != 0 {
            car.put(0x30, car.word(0x30)?.wrapping_sub(1))?;
        } else {
            emit_car_particles(car)?;
        }
        emit_particle_13::emit_particle_13(car)?;
        emit_particle_19::emit_particle_19(car)?;
        emit_particle_22::emit_particle_22(car)?;
    }
    Ok(())
}
