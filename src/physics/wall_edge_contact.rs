//! The wall met between two faces, the faces before (the wall) and now. The wall's
//! four edges ((p1, p2), (p3, p1), (p4, p3), (p2, p4) of its corner words) are looked for in the
//! other face (`rc2k_core::physics::faces::has_edge`); none shared: `physics::wall_push_back`. The
//! first shared, its ends swapped, goes into `wall_edge_from`, `wall_edge_to`; their scene points'
//! x, z give the edge's heading (arctangent of the differences, x `one_over_two_pi`
//! x 65536, & 0xFFFF) << 16 + 0x40000000; the normal (0, 0, 1) turned by it
//! (`math::rotate_float_point_about_y`) into `contact_normal`; the speeds taken, the point of
//! the met probe (its y x `tenth`), no slide limit; `physics::contact_force` into contact record 4,
//! `physics::place_sound_e1`.
//!
//! Refuses a probe point past 0x3F on the edge path (the tables' room), and what its calls refuse
//! (what was written is put back).

use crate::errors::Refused;
use crate::math::rotate_float_point_about_y::rotate_float_point_about_y;
use crate::math::x87::mul;
use crate::math::x87_float::int;
use crate::math::x87_float::to_int;
use crate::physics::contact_force::contact_force;
use crate::physics::contact_store::ContactWork;
use crate::physics::step_families::ContactInputs;
use crate::physics::vector_bits::bits_of;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use rc2k_core::physics::walls;
use std::cell::RefCell;

/// The four corners (scene point numbers) of face `face`.
fn corners(course: &Course, face: u32) -> Result<[u32; 4], Refused> {
    let corner =
        |k: u32| crate::race::course_places::face_field::<u32>(course.reads(), face, 4 + 4 * k);
    Ok([corner(0)?, corner(1)?, corner(2)?, corner(3)?])
}

/// The normal across the edge from scene point `first` to `second` (also used by
/// `crossed_edge_contact`): the heading (the arctangent of the differences of their x, z, x
/// `one_over_two_pi` x 65536, & 0xFFFF) << 16 + 0x40000000 turns (0, 0, 1)
/// (`math::rotate_float_point_about_y`) into `contact_normal`.
pub(super) fn edge_normal(
    engine_tables: &crate::math::engine_tables::EngineTables,
    contact: &RefCell<ContactWork>,
    course: &Course,
    first: u32,
    second: u32,
) -> Result<(), Refused> {
    let rd = |n: u32, k: u32| {
        crate::race::scene_points::point_word(
            course.reads(),
            &crate::race::scene_points::no_own(),
            n,
            k,
        )
    };
    let (first_x, first_z) = (rd(first, 0)?, rd(first, 2)?);
    let (second_x, second_z) = (rd(second, 0)?, rd(second, 2)?);
    let dx = second_x.wrapping_sub(first_x);
    let dz = second_z.wrapping_sub(first_z);
    let d = [dx, dz];
    let turns = x87_1(&course.numbers, d) & 0xFFFF;
    let heading = (turns << 16).wrapping_add(0x4000_0000);
    let (out, _) =
        rotate_float_point_about_y(engine_tables, 0.0, 0.0, course.numbers.one.get(), heading);
    contact.borrow_mut().contact_normal = out.map(f32::to_bits);
    Ok(())
}

/// The two faces of the wall met: the probe point's face now and before.
#[derive(Clone, Copy)]
pub struct WallFaces {
    pub before: u32,
    pub now: u32,
}

/// The steps, not guarded; `probe` the probe point that met the wall.
// Core: `rc2k_core::physics::walls::shared_edge` (the edge's heading stays x87).
pub fn steps(
    c: &ContactInputs<'_>,
    scene: &crate::scene::owner::Scene,
    faces: WallFaces,
    probe: u32,
) -> Result<(), Refused> {
    let face_corners = corners(c.course, faces.now)?;
    let wall_corners = corners(c.course, faces.before)?;
    let (shared, to, from) = walls::shared_edge(|j| wall_corners[j], face_corners);
    if !shared {
        return crate::physics::wall_push_back::steps(c, scene);
    }
    // Along the edge.
    let n = probe;
    walls::check_probe(n)?;
    edge_normal(
        &c.shared.buffers.reads.get().engine_tables,
        c.contact,
        c.course,
        to,
        from,
    )?;
    let n = n as usize;
    let probes = c.ground.probes.borrow();
    let mut point = [
        probes.probe_turned_x[n].to_bits(),
        probes.probe_turned_y[n].to_bits(),
        probes.probe_turned_z[n].to_bits(),
    ];
    drop(probes);
    point[1] = mul(point[1], c.numbers.tenth).to_bits();
    {
        let mut work = c.contact.borrow_mut();
        work.contact_car_speed = bits_of(&c.car.velocity);
        work.contact_turn_speed = bits_of(&c.car.spin);
        work.contact_point = point;
        work.contact_slide_limit = 0;
    }
    contact_force(c.contact, c.numbers, c.tables, c.car, &c.tables.contacts[4]);
    { crate::physics::place_sound_e1::place_sound_e1(&c.sound) }?;
    Ok(())
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [scratch] = (atan2(int [d + 0], int [d + 4]) * [one_over_two_pi]) * [k65536]`
fn x87_1(numbers: &Numbers, d: [u32; 2]) -> u32 {
    let t0 = int(d[0] as i32);
    let t1 = int(d[1] as i32);
    let t2 = t0.atan2(t1);
    let t3 = numbers.one_over_two_pi.ld();
    let t4 = t2 * t3;
    let t5 = numbers.k65536.ld();
    let t6 = t4 * t5;
    to_int(t6) as u32
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87_mem::view;
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
                let turns = x87_1(&crate::test_support::race_places::numbers(), *view(a[0], 0));
                crate::test_support::own::scene().draw_work_mut().scratch = turns;
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.wall_edge_contact", &blocks(), 1200, 100);
    }
}
