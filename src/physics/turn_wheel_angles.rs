//! `physics::turn_wheel_angles` (0x449EE9): the mud amounts moved on by the speed:
//! the four floats at `mud_thrown_in` / `623D70` are `mud_thrown_in` / `mud_splashes_in` (per car
//! texture, read and emptied by `race::update_mud_texture`), not turning angles; the name is the
//! early reading, kept (a lead for a rename to the mud accumulation). Only when the car's kind (`g_car`
//! +0x24, `car_kind`) is below 0x22 and its entry in `kind_gets_mud` is not 0 (else CF
//! and nothing). Then, with `all_wheels_off` 0, s = `g_carDriveSpeed` (else 0), held at
//! 0 or more (`zero`), times `wheel_turn_a` and `wheel_turn_b`, added to the four
//! floats at `mud_thrown_in`; and s = `g_carDriveSpeed` less `g_carSpeed` (else 0), held
//! the same, times `flt_623DAC` and `flt_623DB4`, added to the four at `mud_splashes_in`.
//! Each s goes through `scratch`.
//!
//! Caller: `physics::car_physics_step` (0x43B91E), every physics step.

use crate::math::x87;
use crate::physics::car_store::Car;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::race::kind_store::KIND_TABLES;
use crate::race::vars_store::RaceVars;
use crate::session::Float;
use std::cmp::Ordering;

/// `fld zero; fld s; fcompp`, C0: s below zero or unordered.
fn below(s: u32, zero: u32) -> bool {
    !matches!(
        f32::from_bits(s).partial_cmp(&f32::from_bits(zero)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// One half: `speed` (None with the switch set) held, scaled, added to the four at `to`.
fn add_turn(
    speed: Option<u32>,
    f1: &Float,
    f2: &Float,
    to: &[Float; 4],
    scene: &crate::scene::owner::Scene,
) {
    let drawing = &scene.drawing;
    drawing.scene.scratch.set(0);
    if let Some(s) = speed {
        drawing.scene.scratch.set(s);
        if below(drawing.scene.scratch.get(), ZERO) {
            drawing.scene.scratch.set(x87::loaded(ZERO));
        }
    }
    drawing
        .scene
        .scratch
        .set(x87::mul2(drawing.scene.scratch.get(), f1, f2));
    for cell in to {
        cell.set_bits(x87::add(cell, drawing.scene.scratch.get()));
    }
}

/// The exact version, for `physics::car_physics_step`.
pub fn turn_wheel_angles(car: &Car, vars: &RaceVars, scene: &crate::scene::owner::Scene) {
    let kind = car.car_kind.get();
    if kind >= 0x22 || KIND_TABLES.kind_gets_mud[kind as usize].get() == 0 {
        return;
    }
    let still = car.all_wheels_off.get() != 0;
    let drive = || x87::loaded(&car.drive_speed_a);
    let v = &vars;
    add_turn(
        (!still).then(drive),
        &v.wheel_turn_a[0],
        &v.wheel_turn_b[0],
        &v.mud_thrown_in,
        scene,
    );
    let slip = || x87::sub(&car.drive_speed_a, &car.local_velocity[2]);
    add_turn(
        (!still).then(slip),
        &v.wheel_turn_a[1],
        &v.wheel_turn_b[1],
        &v.mud_splashes_in,
        scene,
    );
}
