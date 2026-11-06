//! `physics::crossed_edge_contact` (0x45DC3E): the contact of the probe point that
//! crossed a face's edge (`dword_6D3B84`, into `dword_6D3214`). Its figures from
//! `dword_6D2C14[n]`, `6D2E14[n]` (x, z before), `6D1E14[n]`, `6D2014[n]` (x, z now) and
//! the face `dword_6D2F14[n]` into `dword_6D31A0` .. `6D31B0`; the face's four corners'
//! x, z (scene points) into `dword_6D3180` .. `6D319C`. Which side of the probe's path
//! each corner lies (the sign of the 64-bit (now - before) cross product; the first
//! product kept in `qword_620460`) picks the edge crossed: p1 -> p2, p2 -> p4, p4 -> p3,
//! else p3 -> p1 (corner words +4 .. +0x10), into `dword_6141BC`, `6141C0`. The normal
//! across it (`physics::wall_edge_contact`'s `edge_normal`; the heading also into
//! `dword_6D31B4`), the speeds, the probe's point (y x `flt_5C5DD0`), no slide limit;
//! `physics::contact_force` into contact record 1 (`g_contactRecords` + 0x20),
//! `physics::place_sound_e1`.
//!
//! In:  nothing
//! Out: ebp the face, and the registers as the steps and `contact_force` leave them; no
//!      flags read (the caller goes on)
//!
//! Caller: `physics::crossed_edge`. Refuses (before anything) a probe point past 0x3F (the
//! tables' room), and what `place_sound_e1` refuses (what was written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87::mul;
use crate::physics::contact_force::contact_force;
use crate::physics::wall_edge_contact::edge_normal;
use crate::race::owner::Race;
use crate::race::owner_physics::Physics;
use crate::session::Float;
use rc2k_core::physics::walls;

/// Three floats copied as bits, in order.
fn copy3(to: &[Float], from: &[Float]) {
    for k in 0..3 {
        to[k].set_bits(from[k].bits());
    }
}

/// Which side of the probe's path the point (x, z) lies: true for not left of it (the
/// 64-bit difference not below 0, `jns` / `stc`). `qword_620460` the first product.
fn side(physics: &Physics, x: u32, z: u32) -> bool {
    let (first, difference) = walls::path_side(
        (
            physics.contact.crossed_figures[0].get(),
            physics.contact.crossed_figures[1].get(),
        ),
        (
            physics.contact.crossed_figures[2].get(),
            physics.contact.crossed_figures[3].get(),
        ),
        (x, z),
    );
    physics.ground_probe.product[0].set(first as u32);
    physics.ground_probe.product[1].set((first >> 32) as u32);
    difference >= 0
}

/// The steps, not guarded (the probe checked before). The registers the original leaves
/// are read by none of its callers.
// Converted: `rc2k_core::physics::walls::{path_side, crossed_edge}`.
pub fn steps(race: &Race) -> Result<(), Refused> {
    let race_owner = race;
    let n = race_owner.physics.contact.crossed_probe.get();
    race_owner.physics.contact.probe_index.set(n);
    let k = n as usize;
    let figures = &race_owner.physics.contact.crossed_figures;
    figures[0].set(race_owner.physics.contact.saved_probe_x[k].get());
    figures[1].set(race_owner.physics.contact.saved_probe_z[k].get());
    figures[2].set(race_owner.physics.contact.probe_x[k].get());
    figures[3].set(race_owner.physics.contact.probe_z[k].get());
    figures[4].set(race_owner.physics.contact.saved_probe_faces[k].get());
    let face = race_owner.physics.contact.crossed_figures[4].get();
    for j in 0..4u32 {
        let point =
            crate::race::course_places::face_field::<u32>(face, 4 + 4 * j, "a face's corner")?;
        race_owner.physics.contact.crossed_corners[2 * j as usize]
            .set(crate::race::scene_points::point_word(point, 0)?);
        race_owner.physics.contact.crossed_corners[2 * j as usize + 1]
            .set(crate::race::scene_points::point_word(point, 2)?);
    }
    let corner = |j: usize| {
        (
            race_owner.physics.contact.crossed_corners[2 * j].get(),
            race_owner.physics.contact.crossed_corners[2 * j + 1].get(),
        )
    };
    // The original's order of tests (each computed, `qword_620460` the last one's).
    let (first, second) = walls::crossed_edge(|j| {
        let (x, z) = corner(j);
        side(&race.physics, x, z)
    });
    let corner_point =
        |at| crate::race::course_places::face_field::<u32>(face, at, "a face's corner");
    let (from, to) = (corner_point(first)?, corner_point(second)?);
    crate::race::owner::race()
        .shared
        .physics_tables
        .crossed_edge[0]
        .set(from);
    crate::race::owner::race()
        .shared
        .physics_tables
        .crossed_edge[1]
        .set(to);
    edge_normal(race, from, to)?;
    race_owner.physics.contact.crossed_heading.set(
        crate::race::owner::race()
            .shared
            .physics_tables
            .wall_heading
            .get(),
    );
    copy3(
        &race_owner.physics.contact.contact_car_speed,
        &crate::race::owner::race().shared.car.velocity,
    );
    copy3(
        &race_owner.physics.contact.contact_turn_speed,
        &crate::race::owner::race().shared.car.spin,
    );
    race_owner.physics.contact.contact_point[0]
        .set_bits(race_owner.physics.contact.probe_turned_x[k].bits());
    race_owner.physics.contact.contact_point[1]
        .set_bits(race_owner.physics.contact.probe_turned_y[k].bits());
    race_owner.physics.contact.contact_point[2]
        .set_bits(race_owner.physics.contact.probe_turned_z[k].bits());
    race_owner.physics.contact.contact_point[1].set_bits(mul(
        race_owner.physics.contact.contact_point[1].bits(),
        race_owner.course.numbers.tenth.bits(),
    ));
    race_owner.physics.contact.contact_slide_limit.set_bits(0);
    contact_force(
        race,
        &crate::race::owner::race().shared.physics_tables.contacts[1],
    );
    { crate::physics::place_sound_e1::place_sound_e1(race) }?;
    Ok(())
}

/// The probe checked: refused past 0x3F.
pub fn check_probe(physics: &Physics) -> Result<(), Refused> {
    Ok(walls::check_probe(physics.contact.crossed_probe.get())?)
}
