//! The contact of the probe point that crossed a face's edge (`crossed_probe`). The side of the probe's
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
use crate::physics::contact_store::ProbeLists;
use crate::physics::step_families::ContactInputs;
use crate::physics::wall_edge_contact::edge_normal;
use rc2k_core::physics::walls;

/// Three floats copied as bits, in order.
fn copy3(to: &[impl FloatLike], from: &[impl FloatLike]) {
    for k in 0..3 {
        to[k].set(from[k].get());
    }
}

/// The probe's path: its x, z before and now.
struct Path {
    before: (u32, u32),
    now: (u32, u32),
}

/// Which side of the probe's path the point (x, z) lies: true for not left of it (the
/// 64-bit difference not below 0).
fn side(path: &Path, x: u32, z: u32) -> bool {
    let (_, difference) = walls::path_side(path.before, path.now, (x, z));
    difference >= 0
}

/// The steps, not guarded (the probe checked before).
// Core: `rc2k_core::physics::walls::{path_side, crossed_edge}`.
pub fn steps(c: &ContactInputs<'_>) -> Result<(), Refused> {
    let probes = c.ground.probes.borrow().clone();
    let k = probes.crossed_probe as usize;
    let path = Path {
        before: (probes.saved_probe_x[k], probes.saved_probe_z[k]),
        now: (probes.probe_x[k], probes.probe_z[k]),
    };
    let face = probes.saved_probe_faces[k];
    let mut corners = [(0u32, 0u32); 4];
    for (j, corner) in corners.iter_mut().enumerate() {
        let point = crate::race::course_places::face_field::<u32>(
            c.course.reads(),
            face,
            4 + 4 * j as u32,
        )?;
        let x = crate::race::scene_points::point_word(
            c.course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            0,
        )?;
        let z = crate::race::scene_points::point_word(
            c.course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            2,
        )?;
        *corner = (x, z);
    }
    // The tests run in this order (each computed, `product` the last one's).
    let (first, second) = walls::crossed_edge(|j| {
        let (x, z) = corners[j];
        side(&path, x, z)
    });
    let corner_point =
        |at| crate::race::course_places::face_field::<u32>(c.course.reads(), face, at);
    let (from, to) = (corner_point(first)?, corner_point(second)?);
    edge_normal(
        &c.shared.buffers.reads.get().engine_tables,
        c.contact,
        c.course,
        from,
        to,
    )?;
    copy3(&c.contact.contact_car_speed, &c.car.velocity);
    copy3(&c.contact.contact_turn_speed, &c.car.spin);
    c.contact.contact_point[0].set(probes.probe_turned_x[k]);
    c.contact.contact_point[1].set(probes.probe_turned_y[k]);
    c.contact.contact_point[2].set(probes.probe_turned_z[k]);
    c.contact.contact_point[1].set(mul(&c.contact.contact_point[1], &c.numbers.tenth));
    c.contact.contact_slide_limit.set(0.0);
    contact_force(c.contact, c.numbers, c.tables, c.car, &c.tables.contacts[1]);
    { crate::physics::place_sound_e1::place_sound_e1(&c.sound) }?;
    Ok(())
}

/// The probe checked: refused past 0x3F.
pub fn check_probe(probes: &ProbeLists) -> Result<(), Refused> {
    Ok(walls::check_probe(probes.crossed_probe)?)
}
