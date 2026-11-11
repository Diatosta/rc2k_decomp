//! `physics::keep_motion` (0x440B7C): `dword_71A17C`, `dword_71A180`, `g_carSpeed`
//! (0x71A184) into `dword_61409C`, `dword_6140A0`, `dword_6140A4` (eax pushed around).
//!
//! Caller: `physics::car_physics_step` (0x43B87E), every physics step.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::session::Cell32;

fn core(v: &[&dyn Cell32]) {
    for k in 0..3 {
        v[k + 3].set_bits(v[k].bits());
    }
}

fn game<'a>(car: &'a Car, tables: &'a PhysicsTables) -> Vec<&'a dyn Cell32> {
    vec![
        &car.local_velocity[0] as &dyn Cell32,
        &car.local_velocity[1],
        &car.local_velocity[2],
        &tables.kept_motion[0],
        &tables.kept_motion[1],
        &tables.kept_motion[2],
    ]
}

pub fn keep_motion(car: &Car, tables: &PhysicsTables) {
    core(&game(car, tables));
}
