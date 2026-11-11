//! `physics::crossed_edge_contact` (0x45DC3E): the contact of the probe point that
//! crossed a face's edge (`crossed_probe`, into `probe_index`). Its figures from
//! `saved_probe_x`[n]`, `6D2E14[n]` (x, z before), `6D1E14[n]`, `6D2014[n]` (x, z now) and
//! the face `saved_probe_faces`[n]` into `crossed_figures` .. `6D31B0`; the face's four corners'
//! x, z (scene points) into `crossed_corners` .. `6D319C`. Which side of the probe's path
//! each corner lies (the sign of the 64-bit (now - before) cross product; the first
//! product kept in `product`) picks the edge crossed: p1 -> p2, p2 -> p4, p4 -> p3,
//! else p3 -> p1 (corner words +4 .. +0x10), into `crossed_edge`, `6141C0`. The normal
//! across it (`physics::wall_edge_contact`'s `edge_normal`; the heading also into
//! `crossed_heading`), the speeds, the probe's point (y x `tenth`), no slide limit;
//! `physics::contact_force` into contact record 1 (`g_contactRecords` + 0x20),
//! `physics::place_sound_e1`.
//!
//! Caller: `physics::crossed_edge`. Refuses (before anything) a probe point past 0x3F (the
//! tables' room), and what `place_sound_e1` refuses (what was written put back).

use crate::errors::Refused;
use crate::math::x87::mul;
use crate::physics::contact_force::contact_force;
use crate::physics::contact_store::ContactWork;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::step_families::ContactInputs;
use crate::physics::wall_edge_contact::edge_normal;
use crate::session::Float;
use rc2k_core::physics::walls;

/// Three floats copied as bits, in order.
fn copy3(to: &[Float], from: &[Float]) {
    for k in 0..3 {
        to[k].set(from[k].get());
    }
}

/// Which side of the probe's path the point (x, z) lies: true for not left of it (the
/// 64-bit difference not below 0, `jns` / `stc`). `product` the first product.
fn side(contact: &ContactWork, probe: &GroundProbe, x: u32, z: u32) -> bool {
    let (first, difference) = walls::path_side(
        (
            contact.crossed_figures[0].get(),
            contact.crossed_figures[1].get(),
        ),
        (
            contact.crossed_figures[2].get(),
            contact.crossed_figures[3].get(),
        ),
        (x, z),
    );
    probe.product[0].set(first as u32);
    probe.product[1].set((first >> 32) as u32);
    difference >= 0
}

/// The steps, not guarded (the probe checked before).
// Converted: `rc2k_core::physics::walls::{path_side, crossed_edge}`.
pub fn steps(c: &ContactInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let n = c.contact.crossed_probe.get();
    c.contact.probe_index.set(n);
    let k = n as usize;
    let figures = &c.contact.crossed_figures;
    figures[0].set(c.contact.saved_probe_x[k].get());
    figures[1].set(c.contact.saved_probe_z[k].get());
    figures[2].set(c.contact.probe_x[k].get());
    figures[3].set(c.contact.probe_z[k].get());
    figures[4].set(c.contact.saved_probe_faces[k].get());
    let face = c.contact.crossed_figures[4].get();
    for j in 0..4u32 {
        let point = crate::race::course_places::face_field::<u32>(c.course, face, 4 + 4 * j)?;
        c.contact.crossed_corners[2 * j as usize]
            .set(crate::race::scene_points::point_word(c.course, point, 0)?);
        c.contact.crossed_corners[2 * j as usize + 1]
            .set(crate::race::scene_points::point_word(c.course, point, 2)?);
    }
    let corner = |j: usize| {
        (
            c.contact.crossed_corners[2 * j].get(),
            c.contact.crossed_corners[2 * j + 1].get(),
        )
    };
    // The original's order of tests (each computed, `product` the last one's).
    let (first, second) = walls::crossed_edge(|j| {
        let (x, z) = corner(j);
        side(c.contact, c.ground.probe, x, z)
    });
    let corner_point = |at| crate::race::course_places::face_field::<u32>(c.course, face, at);
    let (from, to) = (corner_point(first)?, corner_point(second)?);
    c.tables.crossed_edge[0].set(from);
    c.tables.crossed_edge[1].set(to);
    edge_normal(c.contact, c.course, c.tables, from, to, scene)?;
    c.contact.crossed_heading.set(c.tables.wall_heading.get());
    copy3(&c.contact.contact_car_speed, &c.car.velocity);
    copy3(&c.contact.contact_turn_speed, &c.car.spin);
    c.contact.contact_point[0].set(c.contact.probe_turned_x[k].get());
    c.contact.contact_point[1].set(c.contact.probe_turned_y[k].get());
    c.contact.contact_point[2].set(c.contact.probe_turned_z[k].get());
    c.contact.contact_point[1].set(mul(&c.contact.contact_point[1], &c.numbers.tenth));
    c.contact.contact_slide_limit.set(0.0);
    contact_force(
        c.contact,
        c.numbers,
        c.tables,
        c.car,
        &c.tables.contacts[1],
        scene,
    );
    { crate::physics::place_sound_e1::place_sound_e1(&c.sound) }?;
    Ok(())
}

/// The probe checked: refused past 0x3F.
pub fn check_probe(contact: &ContactWork) -> Result<(), Refused> {
    Ok(walls::check_probe(contact.crossed_probe.get())?)
}
