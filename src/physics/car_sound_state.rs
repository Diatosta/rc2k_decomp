//! Tells the work car's engine sound object (the one for the current car) the car's
//! state: active flag 1; the revs; the same car value in four voice slots; the share (the speed
//! over `sound_speed_scale`) and 1.0 (both 0 with the object's mute flag set) into its four
//! voices; the volume held to 0 ..= 0x100; the gear; the brake and slide words; the car's place
//! less the listener's (the first view block's with `players` 1, else the work car's own) turned
//! by the heard turn (`race::heard_turn::turn`) times 1/65536; the car's velocity turned likewise.
//!
//! The exact x87 version (the runs over the object, plain Rust over `X`) stays for
//! `physics::engine_sound`.
//!
//! Refuses (before anything) a null sound object, or a null view block when it is read.

use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87::{ext_times, ints_diff_80};
use crate::math::x87_float::X;
use crate::physics::car_store::Car;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::scene::matrix::Matrix;
use crate::scene::rotate_point::rotate_point_by;

/// The shares into the voices; the integer steps between are in Rust.
fn shares(numbers: &Numbers, shared: &RaceShared, so: &crate::sound::store::SoundObjectCell) {
    x87_1(numbers, shared);
    if shared.car().all_wheels_off.get() != 0 {
        shared.car().sound_slip_share.set(0.0);
        shared.car().sound_level.set(0.0);
        shared.car().sound_speed.set(0.0);
    }
    x87_2(numbers, shared.car(), so);
}

/// The car the sound is heard from: view 0's block with one player (none if it has no
/// block), else the work car, or the followed car while every car of a replay is heard in one
/// view.
fn listener(shared: &RaceShared) -> Option<&crate::physics::car_store::Car> {
    if shared.view.get().players.get() == 1 {
        shared
            .car_blocks()
            .of(shared.view.get().view_blocks[0].get())
    } else {
        Some(crate::race::heard_cars::listener(shared, shared.car()))
    }
}

/// The exact x87 version (for `physics::engine_sound`).
pub fn car_sound_state(
    numbers: &Numbers,
    view_turn: &Matrix,
    shared: &RaceShared,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let Some(l) = listener(shared) else {
        return Err(BadInput("a null view block").into());
    };
    let so = crate::sound::store::current_object(sound, shared.work.get());
    so.borrow_mut().new_revs = 1;
    so.borrow_mut().revs = shared.car().engine_sound_revs.get();
    let w = shared.car().wheel_surface_a[4].get();
    for voice in &mut so.borrow_mut().voices {
        voice.surface = w;
    }
    shares(numbers, shared, so);
    let n = shared.car().throttle.get() as i32;
    so.borrow_mut().throttle = n.clamp(0, 0x100) as u32;
    so.borrow_mut().gear = shared.car().gear.get() as u32;
    so.borrow_mut().speed = shared.car().mean_drive_speed.get();
    so.borrow_mut().flag_ac =
        shared.car().front_wheels_off.get() & shared.car().rear_wheels_off.get();
    so.borrow_mut().view_flag = shared.car().camera_mode_is_2_or_3.get() ^ 1;
    let d = |c: u32, other: u32| ints_diff_80(c as i32, other as i32);
    let place = [
        d(
            shared.car().position.bits_at(0),
            l.camera_position.bits_at(0),
        ),
        d(
            shared.car().position.bits_at(1),
            l.camera_position.bits_at(1),
        ),
        d(
            shared.car().position.bits_at(2),
            l.camera_position.bits_at(2),
        ),
    ];
    let m = shared.heard_turn.get().turn(view_turn);
    let out = rotate_point_by(&m, place);
    let k = numbers.one_over_65536.get();
    for (i, e) in out.iter().enumerate() {
        so.borrow_mut().place[i] = rounded_value(ext_times(*e, k));
    }
    let out = rotate_point_by(
        &m,
        [
            extended(shared.car().crash_matrix.at(8)),
            extended(shared.car().crash_matrix.at(9)),
            extended(shared.car().crash_matrix.at(10)),
        ],
    );
    for (i, e) in out.iter().enumerate() {
        so.borrow_mut().motion[i] = rounded_value(*e);
    }
    Ok(())
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[sound_slip_share] = [slip_speed] / [sound_speed_scale]`
/// `[sound_level] = 1`
fn x87_1(numbers: &Numbers, shared: &RaceShared) {
    let t0 = shared.car().slip_speed.ld();
    let t1 = numbers.sound_speed_scale.ld();
    let t2 = t0 / t1;
    shared.car().sound_slip_share.st(t2);
    let t3 = X(1.0);
    shared.car().sound_level.st(t3);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_2(numbers: &Numbers, car: &Car, o: &crate::sound::store::SoundObjectCell) {
    let stored = |x: X| f32::from_bits(crate::math::x87_float::st(x));
    let t0 = car.sound_level.ld();
    let t1 = car.sound_slip_share.ld();
    let t2 = numbers.k256.ld();
    let t3 = t1 * t2;
    let t4 = car.sound_speed.ld();
    let (share_b, share_c, share_a) = (stored(t0), stored(t3), stored(t4));
    for voice in &mut o.borrow_mut().voices {
        voice.share_b = share_b;
        voice.share_c = share_c;
        voice.share_a = share_a;
    }
}

#[cfg(test)]
mod tests;
