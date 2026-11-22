//! Bumps the work car against another (given its block and number): the corner
//! met (`corner_turned`) goes into `contact_point`; contact record 6's
//! force is zeroed and both cars' contact flags set; the relative speed goes into
//! the relative speed (its length goes to the bump sound); the speeds are shared out
//! (`SHARE_A` / `SHARE_B`), the places' difference / 65536 times the push scale becomes
//! the push, times `three`, added to the work car's speed and taken from the other's; both
//! speeds are turned into the cars' frames; finally `physics::place_sound_e1_other`.
//!
//! The straight float runs are plain Rust over `X` in the named helpers below.
//!
//! Refuses what `place_sound_e1_other` refuses (what was
//! written is put back).

use crate::errors::Refused;
use crate::math::cell_x87::CellX87;
use crate::math::x87_float::X;
use crate::memory::plain_floats::PlainFloats;
use crate::memory::session::PlainFloat;
use crate::physics::car_store::Car;
use crate::physics::step_families::ContactInputs;
use crate::race::numbers_store::Numbers;

use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87::loaded;
use crate::math::x87_float::int;
use crate::physics::scale_vector::scale_vector;
use crate::physics::vector_length::vector_length;
use crate::scene::rotate_point::rotate_point;

/// The block's speed turned by its matrix into its car-frame speed.
fn turn_speed(block: &Car) {
    let turned = rotate_point(
        &block.car_inverse_matrix,
        [0, 1, 2].map(|k| extended(block.velocity.at(k))),
    );
    for (k, e) in turned.into_iter().enumerate() {
        block.local_velocity.set_at(k, rounded_value(e));
    }
}

/// How much of each car's speed the other's share takes: the first car keeps `SHARE_B` of its own
/// and takes `SHARE_A` of the other's.
const SHARE_A: f32 = 0.4;
const SHARE_B: f32 = 0.6;
/// The places' difference over 65536 is scaled by this into the push.
const PUSH_SCALE: f32 = 0.5;

fn steps(
    c: &ContactInputs<'_>,
    corner_turned: &[PlainFloat; 3],
    other_car: &Car,
    other_car_number: u32,
) -> Result<(), Refused> {
    // The relative speed and the push: each is made before it is read, within this bump.
    let relative: [PlainFloat; 3] = PlainFloat::array([0.0; 3]);
    let push: [PlainFloat; 3] = PlainFloat::array([0.0; 3]);
    {
        let turned = [0, 1, 2].map(|k| loaded(&corner_turned[k]).to_bits());
        c.contact.borrow_mut().contact_point = turned;
    }
    c.tables.borrow_mut().contacts[6].force = [0.0; 3];
    c.car.contact_flag.set(1);
    other_car.contact_flag.set(1);
    relative_speed(c.car, &relative, &other_car.velocity);
    let bump_speed_length = rounded_value(vector_length(&relative));
    share_out_speeds(
        c.car,
        crate::math::x87_float::ld(SHARE_A.to_bits()),
        crate::math::x87_float::ld(SHARE_B.to_bits()),
        &other_car.velocity,
    );
    // Pushed apart by the places' difference.
    push_apart(
        c.car,
        crate::math::x87_float::ld(PUSH_SCALE.to_bits()),
        &push,
        c.numbers,
        &other_car.position,
    );
    let factor = extended(c.numbers.three.get());
    scale_vector(&push, factor);
    add_push(c.car, &push, &other_car.velocity);
    turn_speed(other_car);
    turn_speed(c.car);
    {
        crate::physics::place_sound_e1_other::place_sound_e1_other(
            &c.sound,
            other_car_number,
            bump_speed_length,
        )
    }?;
    Ok(())
}

/// `corner_turned` the corner met, `other_car` the other car's block, `other_car_number` its number.
pub fn bump_cars(
    c: &ContactInputs<'_>,
    corner_turned: &[PlainFloat; 3],
    other_car: &Car,
    other_car_number: u32,
) -> Result<(), Refused> {
    steps(c, corner_turned, other_car, other_car_number)
}

// ---- the x87 arithmetic ----

/// the work car's speed less `d` (the other car's), on each axis, into the relative speed.
fn relative_speed(car: &Car, relative: &[PlainFloat; 3], d: &PlainFloats<3>) {
    for (k, relative_k) in relative.iter().enumerate() {
        let t0 = car.velocity.ld(k);
        let t1 = d.ld(k);
        let t2 = t0 - t1;
        relative_k.st(t2);
    }
}

/// the two cars' speeds shared out on each axis, with a the work car's speed and b the other's
/// (`s`): `a' = a x share_a + b x share_b`, `b' = a x share_b + b x share_a`.
fn share_out_speeds(car: &Car, share_a: X, share_b: X, s: &PlainFloats<3>) {
    for k in 0..3 {
        let t0 = car.velocity.ld(k);
        let t1 = share_b;
        let t2 = t0 * t1;
        let t3 = s.ld(k);
        let t4 = share_a;
        let t5 = t3 * t4;
        let t6 = t2 + t5;
        let t7 = car.velocity.ld(k);
        let t8 = share_a;
        let t9 = t7 * t8;
        let t10 = s.ld(k);
        let t11 = share_b;
        let t12 = t10 * t11;
        let t13 = t9 + t12;
        car.velocity.st_at(k, t13);
        s.st_at(k, t6);
    }
}

/// the push on each axis: the places' difference (the work car's less `d`'s) over 65536 times the
/// push scale.
fn push_apart(
    car: &Car,
    push_scale: X,
    push: &[PlainFloat; 3],
    numbers: &Numbers,
    d: &PlainFloats<3>,
) {
    for (k, push_k) in push.iter().enumerate() {
        let t0 = int(car.position.bits_at(k) as i32);
        let t1 = int(d.bits_at(k) as i32);
        let t2 = t0 - t1;
        let t3 = numbers.k65536.ld();
        let t4 = t2 / t3;
        let t5 = push_scale;
        let t6 = t4 * t5;
        push_k.st(t6);
    }
}

/// the push added to the work car's speed on each axis, then taken from `d`'s.
fn add_push(car: &Car, push: &[PlainFloat; 3], d: &PlainFloats<3>) {
    for (k, push_k) in push.iter().enumerate() {
        let t0 = car.velocity.ld(k);
        let t1 = push_k.ld();
        let t2 = t0 + t1;
        car.velocity.st_at(k, t2);
    }
    for (k, push_k) in push.iter().enumerate() {
        let t9 = d.ld(k);
        let t10 = push_k.ld();
        let t11 = t9 - t10;
        d.st_at(k, t11);
    }
}

#[cfg(test)]
mod tests;
