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
use crate::physics::car_store::CAR;
use crate::physics::contact_force::contact_force;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::flow::p;
use crate::physics::tables_store::PHYSICS;
use crate::physics::wall_edge_contact::edge_normal;
use crate::race::course_store::COURSE;
use crate::race::numbers_store::NUMBERS;
use crate::session::Float;
use rc2k_core::physics::walls;

/// Three floats copied as bits, in order.
fn copy3(to: &[Float], from: &[Float]) {
    for k in 0..3 {
        to[k].set_bits(from[k].bits());
    }
}

fn rd(x: u32) -> Result<u32, Refused> {
    Ok(crate::race::course_places::read::<u32>(
        x,
        "a face's or scene point's dword",
    )?)
}

/// Which side of the probe's path the point (x, z) lies: true for not left of it (the
/// 64-bit difference not below 0, `jns` / `stc`). `qword_620460` the first product.
fn side(x: u32, z: u32) -> bool {
    let (first, difference) = walls::path_side(
        (
            CONTACT_WORK.crossed_figures[0].get(),
            CONTACT_WORK.crossed_figures[1].get(),
        ),
        (
            CONTACT_WORK.crossed_figures[2].get(),
            CONTACT_WORK.crossed_figures[3].get(),
        ),
        (x, z),
    );
    crate::physics::ground_probe_store::GROUND_PROBE.product[0].set(first as u32);
    crate::physics::ground_probe_store::GROUND_PROBE.product[1].set((first >> 32) as u32);
    difference >= 0
}

/// The steps, not guarded (the probe checked before). The registers the original leaves
/// are read by none of its callers.
// Converted: `rc2k_core::physics::walls::{path_side, crossed_edge}`.
pub fn steps() -> Result<(), Refused> {
    let n = CONTACT_WORK.crossed_probe.get();
    CONTACT_WORK.probe_index.set(n);
    let k = n as usize;
    let figures = &CONTACT_WORK.crossed_figures;
    figures[0].set(CONTACT_WORK.saved_probe_x[k].get());
    figures[1].set(CONTACT_WORK.saved_probe_z[k].get());
    figures[2].set(CONTACT_WORK.probe_x[k].get());
    figures[3].set(CONTACT_WORK.probe_z[k].get());
    figures[4].set(CONTACT_WORK.saved_probe_faces[k].get());
    let face = (CONTACT_WORK.crossed_figures[4].get() << 6).wrapping_add(COURSE.course_faces.get());
    for j in 0..4u32 {
        let point = rd(face + 4 + 4 * j)?
            .wrapping_mul(12)
            .wrapping_add(COURSE.scene_points.get());
        CONTACT_WORK.crossed_corners[2 * j as usize].set(crate::race::scene_points::word(point)?);
        CONTACT_WORK.crossed_corners[2 * j as usize + 1]
            .set(crate::race::scene_points::word(point + 8)?);
    }
    let corner = |j: usize| {
        (
            CONTACT_WORK.crossed_corners[2 * j].get(),
            CONTACT_WORK.crossed_corners[2 * j + 1].get(),
        )
    };
    // The original's order of tests (each computed, `qword_620460` the last one's).
    let (first, second) = walls::crossed_edge(|j| {
        let (x, z) = corner(j);
        side(x, z)
    });
    let (from, to) = (rd(face + first)?, rd(face + second)?);
    PHYSICS.crossed_edge[0].set(from);
    PHYSICS.crossed_edge[1].set(to);
    edge_normal(from, to)?;
    CONTACT_WORK.crossed_heading.set(PHYSICS.wall_heading.get());
    copy3(&CONTACT_WORK.contact_car_speed, &CAR.velocity);
    copy3(&CONTACT_WORK.contact_turn_speed, &CAR.spin);
    CONTACT_WORK.contact_point[0].set_bits(CONTACT_WORK.probe_turned_x[k].bits());
    CONTACT_WORK.contact_point[1].set_bits(CONTACT_WORK.probe_turned_y[k].bits());
    CONTACT_WORK.contact_point[2].set_bits(CONTACT_WORK.probe_turned_z[k].bits());
    CONTACT_WORK.contact_point[1].set_bits(mul(
        CONTACT_WORK.contact_point[1].bits(),
        NUMBERS.tenth.bits(),
    ));
    CONTACT_WORK.contact_slide_limit.set_bits(0);
    p(&PHYSICS.contact_record, PHYSICS.contacts[1].force[0].addr());
    contact_force();
    { crate::physics::place_sound_e1::place_sound_e1() }?;
    Ok(())
}

/// The probe checked: refused past 0x3F.
pub fn check_probe() -> Result<(), Refused> {
    Ok(walls::check_probe(CONTACT_WORK.crossed_probe.get())?)
}
