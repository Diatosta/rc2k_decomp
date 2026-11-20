//! The contact of the probe point that crossed a face's edge (`crossed_probe`, into
//! `probe_index`). The probe's x, z before and now and its face go into `crossed_figures`; the
//! face's four corners' x, z (scene points) into `crossed_corners`. The side of the probe's
//! path each corner lies on (the sign of the 64-bit cross product of the move) picks the edge crossed: p1 -> p2, p2 -> p4, p4 -> p3, else p3
//! -> p1, into `crossed_edge`. Then the normal across the edge (`physics::wall_edge_contact`'s
//! `edge_normal`; the heading also into `crossed_heading`), the speeds and the probe's point (y
//! times `tenth`) are set, with no slide limit; `physics::contact_force` fills contact record
//! 1, then `physics::place_sound_e1`.
//!
//! Refuses (before anything) a probe point past 0x3F (the tables' room), and what `place_sound_e1`
//! refuses (what was written is put back).

use crate::errors::Refused;
use crate::math::x87::mul;
use crate::memory::session::FloatLike;
use crate::physics::contact_force::contact_force;
use crate::physics::contact_store::ContactWork;
use crate::physics::step_families::ContactInputs;
use crate::physics::wall_edge_contact::edge_normal;
use rc2k_core::physics::walls;

/// Three floats copied as bits, in order.
fn copy3(to: &[impl FloatLike], from: &[impl FloatLike]) {
    for k in 0..3 {
        to[k].set(from[k].get());
    }
}

/// Which side of the probe's path the point (x, z) lies: true for not left of it (the
/// 64-bit difference not below 0).
fn side(contact: &ContactWork, x: u32, z: u32) -> bool {
    let (_, difference) = walls::path_side(
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
    difference >= 0
}

/// The steps, not guarded (the probe checked before).
// Core: `rc2k_core::physics::walls::{path_side, crossed_edge}`.
pub fn steps(c: &ContactInputs<'_>) -> Result<(), Refused> {
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
        let point =
            crate::race::course_places::face_field::<u32>(c.course.reads(), face, 4 + 4 * j)?;
        c.contact.crossed_corners[2 * j as usize].set(crate::race::scene_points::point_word(
            c.course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            0,
        )?);
        c.contact.crossed_corners[2 * j as usize + 1].set(crate::race::scene_points::point_word(
            c.course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            2,
        )?);
    }
    let corner = |j: usize| {
        (
            c.contact.crossed_corners[2 * j].get(),
            c.contact.crossed_corners[2 * j + 1].get(),
        )
    };
    // The tests run in this order (each computed, `product` the last one's).
    let (first, second) = walls::crossed_edge(|j| {
        let (x, z) = corner(j);
        side(c.contact, x, z)
    });
    let corner_point =
        |at| crate::race::course_places::face_field::<u32>(c.course.reads(), face, at);
    let (from, to) = (corner_point(first)?, corner_point(second)?);
    edge_normal(
        &c.shared.buffers.reads.get().engine_tables,
        c.contact,
        c.course,
        c.tables,
        from,
        to,
    )?;
    copy3(&c.contact.contact_car_speed, &c.car.velocity);
    copy3(&c.contact.contact_turn_speed, &c.car.spin);
    c.contact.contact_point[0].set(c.contact.probe_turned_x[k].get());
    c.contact.contact_point[1].set(c.contact.probe_turned_y[k].get());
    c.contact.contact_point[2].set(c.contact.probe_turned_z[k].get());
    c.contact.contact_point[1].set(mul(&c.contact.contact_point[1], &c.numbers.tenth));
    c.contact.contact_slide_limit.set(0.0);
    contact_force(c.contact, c.numbers, c.tables, c.car, &c.tables.contacts[1]);
    { crate::physics::place_sound_e1::place_sound_e1(&c.sound) }?;
    Ok(())
}

/// The probe checked: refused past 0x3F.
pub fn check_probe(contact: &ContactWork) -> Result<(), Refused> {
    Ok(walls::check_probe(contact.crossed_probe.get())?)
}
