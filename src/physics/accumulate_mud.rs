//! Mud thrown up by the car's speed: adds to the per-texture mud amounts (`mud_thrown_in`,
//! `mud_splashes_in`) that `race::update_mud_texture` reads and empties.
//!
//! Only cars whose kind is below 0x22 and flagged in `kind_gets_mud` take part. With no wheel off
//! the ground, the drive speed (clamped at 0) times the scale pair is added to the four thrown
//! amounts; the drive speed less the car speed (clamped at 0) times the second pair is added to
//! the four splash amounts.
//!
//! Caller: `physics::car_physics_step`, every physics step.

use crate::math::x87;
use crate::physics::car_store::Car;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::race::kind_store::KindTables;
use crate::race::vars_store::RaceVars;
use crate::session::PlainFloat;
use std::cmp::Ordering;

/// Whether `s` is below zero or unordered.
fn below(s: u32, zero: u32) -> bool {
    !matches!(
        f32::from_bits(s).partial_cmp(&f32::from_bits(zero)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// One half: `speed` (None with the switch set) held, scaled, added to the four at `to`.
fn add_mud(
    speed: Option<u32>,
    f1: &PlainFloat,
    f2: &PlainFloat,
    to: &[PlainFloat; 4],
    scene: &crate::scene::owner::Scene,
) {
    let drawing = &scene.drawing;
    drawing.scene.get().scratch.set(0);
    if let Some(s) = speed {
        drawing.scene.get().scratch.set(s);
        if below(drawing.scene.get().scratch.get(), ZERO) {
            drawing.scene.get().scratch.set(x87::loaded(ZERO).to_bits());
        }
    }
    drawing
        .scene
        .get()
        .scratch
        .set(x87::mul2(drawing.scene.get().scratch.get(), f1, f2).to_bits());
    for cell in to {
        cell.set(x87::add(cell, drawing.scene.get().scratch.get()));
    }
}

/// The exact version, for `physics::car_physics_step`.
pub fn accumulate_mud(
    car: &Car,
    vars: &RaceVars,
    kinds: &KindTables,
    scene: &crate::scene::owner::Scene,
) {
    let kind = car.car_kind.get();
    if kind >= 0x22 || kinds.kind_gets_mud[kind as usize].get() == 0 {
        return;
    }
    let still = car.all_wheels_off.get() != 0;
    let drive = || x87::loaded(&car.drive_speed_a).to_bits();
    let v = &vars;
    add_mud(
        (!still).then(drive),
        &v.mud_scale_a[0],
        &v.mud_scale_b[0],
        &v.mud_thrown_in,
        scene,
    );
    let slip = || x87::sub(&car.drive_speed_a, &car.local_velocity[2]).to_bits();
    add_mud(
        (!still).then(slip),
        &v.mud_scale_a[1],
        &v.mud_scale_b[1],
        &v.mud_splashes_in,
        scene,
    );
}
