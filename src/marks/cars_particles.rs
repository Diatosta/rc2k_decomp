//! `marks::cars_particles` (0x4C9AF6): the marks' frame for each car, when the settings'
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
//! `dword_93547A` from `dword_7694AA` by the car's +4; then `marks::emit_car_particles`
//! (unless +0x30 counts down, by one) and `marks::emit_particle_13`, `emit_particle_19`,
//! `emit_particle_22`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: ebp the settings when nothing is done, else as the last car leaves it (see
//!      `cars`); the rest as the last car leaves them
//!
//! Caller: `sub_4C3EEE` (0x4C4006). Refuses null settings, more than 8 cars, a car's null
//! matrix or vector, a car model (+4) of 34 or more, and the emitters' refusals; on a
//! refusal nothing stays written (all it may write is put back).

use crate::errors::{BadInput, Refused};
use crate::marks::emit_car_particles::emit_car_particles;
use crate::marks::emitters::*;
use crate::marks::splines::SPLINE_HEAD;
use crate::marks::tuning::MARKS_FLOATS;
use crate::marks::{emit_particle_13, emit_particle_19, emit_particle_22};
use crate::math::x87;
use crate::scene::adjugate_matrix::adjugate_matrix;
use crate::session::{read_bits, write_bits};

const CAR_LEN: u32 = 0x4C8;
const MOST_CARS: u32 = 8;
// The looks by the car's model are `SPLINE_HEAD.looks` (dword_7694AA, dword_7694B2: 8 bytes
// apart, 33 pairs to 0x7695B2). The cars have 34 models: model 32's second dword is
// `g_marksSplineFaces`, model 33 (the Clio's) reads `g_marksSplineFaces` and `flt_7695BA`,
// as the original does (`notes/bugs.md`): `SplineHead::look` reads past the table's end.
const MODELS: u32 = 34;

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The cars, worked out without writing: none when the settings' +0x10 is not 1.
unsafe fn cars(settings: u32) -> Vec<u32> {
    unsafe {
        if at(settings + 0x10) != 1 {
            return Vec::new();
        }
        let count = at(settings + 0x5C) as i32;
        (0..count.clamp(0, MOST_CARS as i32 + 1) as u32)
            .rev()
            .map(|n| settings + 0x60 + n * CAR_LEN)
            .collect()
    }
}

/// Why the Rust version refuses, if it does (its own checks), worked out without writing.
unsafe fn refusal(settings: u32) -> Option<&'static str> {
    unsafe {
        if settings == 0 {
            return Some("null marks settings");
        }
        if at(settings + 0x10) == 1 && at(settings + 0x5C) as i32 > MOST_CARS as i32 {
            return Some("more than 8 cars");
        }
        for car in cars(settings) {
            if at(car + 0x24) == 0 || at(car + 0x28) == 0 {
                return Some("a car's null matrix or vector");
            }
            if at(car + 4) >= MODELS {
                return Some("a car model past the cars' 34");
            }
        }
        None
    }
}

/// The Rust version (see the header).
pub unsafe fn cars_particles() -> Result<(), Refused> {
    let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
    if let Some(why) = unsafe { refusal(settings) } {
        return Err(BadInput(why).into());
    }
    let count = unsafe { cars(settings) }.len() as u32;
    unsafe {
        for n in (0..count).rev() {
            EMITTER
                .tag
                .set(x87::int_times_int_to_int(n as i32, EMITTER.tag_k.get() as i32) as u32);
            let car = settings + 0x60 + n * CAR_LEN;
            EMITTER.car.set(car);
            let m = at(car + 0x24);
            adjugate_matrix(m, car + 0x94);
            for (to, from) in [(0x7C, 0x70), (0x70, 0x64), (0x64, 0x58)] {
                let v = [at(car + from), at(car + from + 4), at(car + from + 8)];
                for (k, x) in v.iter().enumerate() {
                    put(car + to + 4 * k as u32, *x);
                }
            }
            for k in 0..3 {
                let k4 = 4 * k;
                put(
                    car + 0x58 + k4,
                    x87::sub(at(m + 0x30 + k4), at(car + 0x40 + k4)),
                );
            }
            let mean_k: u32 = MARKS_FLOATS.quarter.bits();
            for k in 0..3 {
                let k4 = 4 * k;
                put(
                    car + 0x4C + k4,
                    x87::sum4_times(
                        at(car + 0x58 + k4),
                        at(car + 0x64 + k4),
                        at(car + 0x70 + k4),
                        at(car + 0x7C + k4),
                        mean_k,
                    ),
                );
            }
            put(car + 0x40, at(m + 0x30));
            put(car + 0x44, at(m + 0x34));
            put(car + 0x48, at(m + 0x38));
            let [k1, k2, k3] = read_bits(&EMITTER.rate_k);
            let one: u32 = MARKS_FLOATS.one.bits();
            let (length, rate) = x87::length_and_rate(
                [at(car + 0x4C), at(car + 0x50), at(car + 0x54)],
                k1,
                k2,
                one,
                k3,
            );
            put(car + 0x88, length as u32);
            put(car + 0x90, rate as u32);
            if at(car + 0x30) != 0 {
                for o in [0x4C, 0x50, 0x54, 0x88] {
                    put(car + o, 0);
                }
            }
            let w = at(car + 0x28);
            let up = read_bits(&EMITTER.up);
            let across = [
                x87::product_difference(up[1], at(w + 8), up[2], at(w + 4)),
                x87::product_difference(up[2], at(w), up[0], at(w + 8)),
                x87::product_difference(up[0], at(w + 4), up[1], at(w)),
            ];
            let across = x87::normalised3(across, one);
            write_bits(&EMITTER.across, across);
            let b = x87::crossed_turned(across, w, m);
            write_bits(&EMITTER.side_b, b);
            let a = [
                x87::product_difference(b[1], across[2], b[2], across[1]),
                x87::product_difference(b[2], across[0], b[0], across[2]),
                x87::product_difference(b[0], across[1], b[1], across[0]),
            ];
            write_bits(&EMITTER.side_a, a);
            let model = at(car + 4);
            EMITTER.looks[0].set(SPLINE_HEAD.look(2 * model as usize));
            EMITTER.looks[1].set(SPLINE_HEAD.look(2 * model as usize + 2));
            if at(car + 0x30) != 0 {
                put(car + 0x30, at(car + 0x30).wrapping_sub(1));
            } else {
                emit_car_particles(car)?;
            }
            emit_particle_13::emit_particle_13(car)?;
            emit_particle_19::emit_particle_19(car)?;
            emit_particle_22::emit_particle_22(car)?;
        }
    }
    Ok(())
}
