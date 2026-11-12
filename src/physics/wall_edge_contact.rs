//! `physics::wall_edge_contact` (0x45DA14, two chunks): the wall met between two faces,
//! `wall_face_before` (the wall) and `wall_face_now`. The wall's four edges ((p1, p2), (p3, p1),
//! (p4, p3), (p2, p4) of its corner words +4 .. +0x10) are looked for in the other face
//! (`physics::face_has_edge`); none shared: `physics::wall_push_back`. The first shared, its ends
//! swapped, goes into `wall_edge_from`, `wall_edge_to`; their scene points' x, z into
//! `edge_points`; the edge's heading (arctangent of the differences, x `one_over_two_pi` x 65536,
//! & 0xFFFF) << 16 + 0x40000000 into `wall_heading`; the normal (0, 0, 1) turned by it
//! (`math::rotate_float_point_about_y`) into `contact_normal`; the speeds taken, the point of probe
//! `probe_index` (its y x `tenth`), no slide limit; `physics::contact_force` into contact record
//! 4, `physics::place_sound_e1`.
//!
//! Caller: `physics::wall_contacts`. Refuses a probe point past 0x3F on the edge path (the
//! tables' room), and what its calls refuse (what was written put back).

use crate::effects::cell_x87::CellX87;
use crate::errors::Refused;
use crate::math::rotate_float_point_about_y::rotate_float_point_about_y;
use crate::math::x87::mul;
use crate::math::x87_float::int;
use crate::math::x87_float::to_int;
use crate::physics::contact_force::contact_force;
use crate::physics::contact_store::ContactWork;
use crate::physics::flow::{ZERO_BITS as ZERO, copy_cells};
use crate::physics::step_families::ContactInputs;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use rc2k_core::physics::walls;

/// The four corners (scene point numbers) of face `face`.
fn corners(course: &Course, face: u32) -> Result<[u32; 4], Refused> {
    let corner = |k: u32| crate::race::course_places::face_field::<u32>(course, face, 4 + 4 * k);
    Ok([corner(0)?, corner(1)?, corner(2)?, corner(3)?])
}

/// The normal across the edge from scene point `first` to `second` (also used by
/// `crossed_edge_contact`): their x, z go into `edge_points`; the heading (the arctangent of the
/// differences, x `one_over_two_pi` x 65536, & 0xFFFF) << 16 + 0x40000000 goes into
/// `wall_heading`; (0, 0, 1) turned by it (`math::rotate_float_point_about_y`) goes into
/// `contact_normal`. Returns the leftover register values of the original's code.
pub fn edge_normal(
    contact: &ContactWork,
    course: &Course,
    physics_tables: &PhysicsTables,
    first: u32,
    second: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<[u32; 4], Refused> {
    let drawing = &scene.drawing;
    let rd = |n: u32, k: u32| {
        crate::race::scene_points::point_word(course, &crate::race::scene_points::NO_OWN, n, k)
    };
    physics_tables.edge_points[0].set(rd(first, 0)?);
    physics_tables.edge_points[1].set(rd(first, 2)?);
    physics_tables.edge_points[2].set(rd(second, 0)?);
    physics_tables.edge_points[3].set(rd(second, 2)?);
    let dx = physics_tables.edge_points[2]
        .get()
        .wrapping_sub(physics_tables.edge_points[0].get());
    let dz = physics_tables.edge_points[3]
        .get()
        .wrapping_sub(physics_tables.edge_points[1].get());
    let d = [dx, dz];
    x87_1(&course.numbers, d, scene);
    drawing
        .scene
        .scratch
        .set(drawing.scene.scratch.get() & 0xFFFF);
    let heading = (drawing.scene.scratch.get() << 16).wrapping_add(0x4000_0000);
    physics_tables.wall_heading.set(heading);
    let out = { rotate_float_point_about_y(ZERO, ZERO, course.numbers.one.bits(), heading) };
    contact.contact_normal[0].set_bits(out[0]);
    contact.contact_normal[1].set_bits(out[1]);
    contact.contact_normal[2].set_bits(out[2]);
    Ok(out)
}

/// The steps, not guarded.
// Converted: `rc2k_core::physics::walls::shared_edge` (the edge's heading stays x87).
pub fn steps(c: &ContactInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let face_corners = corners(c.course, c.contact.wall_face_now.get())?;
    let wall_corners = corners(c.course, c.contact.wall_face_before.get())?;
    let (shared, to, from) = walls::shared_edge(|j| wall_corners[j], face_corners);
    if !shared {
        return crate::physics::wall_push_back::steps(c, scene);
    }
    // 0x45DA9E: along the edge.
    let n = c.contact.probe_index.get();
    walls::check_probe(n)?;
    c.contact.wall_edge_from.set(from);
    c.contact.wall_edge_to.set(to);
    edge_normal(c.contact, c.course, c.tables, to, from, scene)?;
    copy_cells(&c.contact.contact_car_speed, &c.car.velocity);
    copy_cells(&c.contact.contact_turn_speed, &c.car.spin);
    let n = n as usize;
    c.contact.contact_point[0].set(c.contact.probe_turned_x[n].get());
    c.contact.contact_point[1].set(c.contact.probe_turned_y[n].get());
    c.contact.contact_point[2].set(c.contact.probe_turned_z[n].get());
    c.contact.contact_point[1].set(mul(&c.contact.contact_point[1], &c.numbers.tenth));
    c.contact.contact_slide_limit.set(0.0);
    contact_force(
        c.contact,
        c.numbers,
        c.tables,
        c.car,
        &c.tables.contacts[4],
        scene,
    );
    { crate::physics::place_sound_e1::place_sound_e1(&c.sound) }?;
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [scratch] = (atan2(int [d + 0], int [d + 4]) * [one_over_two_pi]) * [k65536]`
fn x87_1(numbers: &Numbers, d: [u32; 2], scene: &crate::scene::owner::Scene) {
    let t0 = int(d[0] as i32);
    let t1 = int(d[1] as i32);
    let t2 = t0.atan2(t1);
    let t3 = numbers.one_over_two_pi.ld();
    let t4 = t2 * t3;
    let t5 = numbers.k65536.ld();
    let t6 = t4 * t5;
    scene.drawing.scene.scratch.set_bits(to_int(t6) as u32);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::x87_mem::view;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "wall_edge_contact line 52",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::I), (4, In::I)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F), (0x1C5DD4, In::F)],
                    writes: &[(0x22400C, Out::I)],
                },
            ],
            ulps: 0,
            plain: |a| {
                x87_1(
                    &race().course.numbers,
                    *view(a[0], 0),
                    crate::scene::owner::scene(),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.wall_edge_contact", &blocks(), 1200, 100);
    }
}
