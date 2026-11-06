//! `physics::wall_edge_contact` (0x45DA14, two chunks): the wall met between two faces,
//! `dword_6D3170` (the wall, ebp) and `dword_6D3174` (esi). The wall's four edges
//! ((p1, p2), (p3, p1), (p4, p3), (p2, p4) of its corner words +4 .. +0x10) are looked
//! for in the other face (`physics::face_has_edge`); none shared: `physics::wall_push_back`.
//! The first shared, its ends swapped, into `dword_6D3178`, `dword_6D317C`; their scene
//! points' x, z into `dword_6141AC` .. `6141B8`; the edge's heading (`fpatan` of the
//! differences, x `flt_5C5DD4` x `flt_5C5D98`, & 0xFFFF) << 16 + 0x40000000 into
//! `dword_6141C8`; the normal (0, 0, 1) turned by it (`math::turn_float_point_xz`) into
//! `qword_6D3078` ..; the speeds taken, the point of probe `dword_6D3214` (its y x
//! `flt_5C5DD0`), no slide limit; `physics::contact_force` into contact record 4,
//! `physics::place_sound_e1`.
//!
//! In:  nothing
//! Out: ebp the wall, and the registers as the edge tests and `contact_force` leave them;
//!      no flags read (the caller goes on)
//!
//! Caller: `physics::wall_contacts`. Refuses a probe point past 0x3F on the edge path (the
//! tables' room), and what its calls refuse (what was written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::turn_float_point_xz::turn_float_point_xz;
use crate::math::x87::mul;
use crate::math::x87_float::to_int;
use crate::math::x87_float::{int, ld};
use crate::physics::car_store::CAR;
use crate::physics::contact_force::contact_force;
use crate::physics::flow::copy_cells;
use crate::physics::tables_store::PHYSICS;
use crate::scene::draw_store::DRAW;
use rc2k_core::physics::walls;

/// The four corners (scene point numbers) of face `face`.
fn corners(face: u32) -> Result<[u32; 4], Refused> {
    let corner =
        |k: u32| crate::race::course_places::face_field::<u32>(face, 4 + 4 * k, "a face's corner");
    Ok([corner(0)?, corner(1)?, corner(2)?, corner(3)?])
}

/// The normal across the edge from scene point `first` to `second` (0x45DAAA ..
/// 0x45DB70, and `crossed_edge_contact`'s): their x, z into `dword_6141AC`, `6141B0`
/// (first) and `6141B4`, `6141B8`; the heading (`fpatan` of the differences, x
/// `flt_5C5DD4` x `flt_5C5D98`, & 0xFFFF, through `dword_62400C`) << 16 + 0x40000000
/// into `dword_6141C8`; (0, 0, 1) turned by it (`math::turn_float_point_xz`) into
/// `qword_6D3078` ... Returns what that leaves in eax, ebx, ecx, edx.
pub fn edge_normal(first: u32, second: u32) -> Result<[u32; 4], Refused> {
    let rd = crate::race::scene_points::point_word;
    PHYSICS.edge_points[0].set(rd(first, 0)?);
    PHYSICS.edge_points[1].set(rd(first, 2)?);
    PHYSICS.edge_points[2].set(rd(second, 0)?);
    PHYSICS.edge_points[3].set(rd(second, 2)?);
    let dx = PHYSICS.edge_points[2]
        .get()
        .wrapping_sub(PHYSICS.edge_points[0].get());
    let dz = PHYSICS.edge_points[3]
        .get()
        .wrapping_sub(PHYSICS.edge_points[1].get());
    let d = [dx, dz];
    x87_1(d);
    DRAW.scratch.set(DRAW.scratch.get() & 0xFFFF);
    let heading = (DRAW.scratch.get() << 16).wrapping_add(0x4000_0000);
    PHYSICS.wall_heading.set(heading);
    let out = {
        turn_float_point_xz(
            crate::race::owner::race().course.numbers.zero.bits(),
            crate::race::owner::race().course.numbers.zero.bits(),
            crate::race::owner::race().course.numbers.one.bits(),
            heading,
        )
    };
    crate::race::owner::race().physics.contact.contact_normal[0].set_bits(out[0]);
    crate::race::owner::race().physics.contact.contact_normal[1].set_bits(out[1]);
    crate::race::owner::race().physics.contact.contact_normal[2].set_bits(out[2]);
    Ok(out)
}

/// The steps, not guarded. The registers the original leaves are read by none of its
/// callers.
// Converted: `rc2k_core::physics::walls::shared_edge` (the edge's heading stays x87).
pub fn steps() -> Result<(), Refused> {
    let face_corners = corners(
        crate::race::owner::race()
            .physics
            .contact
            .wall_face_now
            .get(),
    )?;
    let wall_corners = corners(
        crate::race::owner::race()
            .physics
            .contact
            .wall_face_before
            .get(),
    )?;
    let (shared, to, from) = walls::shared_edge(|j| wall_corners[j], face_corners);
    if !shared {
        return crate::physics::wall_push_back::steps();
    }
    // 0x45DA9E: along the edge.
    let n = crate::race::owner::race().physics.contact.probe_index.get();
    walls::check_probe(n)?;
    crate::race::owner::race()
        .physics
        .contact
        .wall_edge_from
        .set(from);
    crate::race::owner::race()
        .physics
        .contact
        .wall_edge_to
        .set(to);
    edge_normal(to, from)?;
    copy_cells(
        &crate::race::owner::race().physics.contact.contact_car_speed,
        &CAR.velocity,
    );
    copy_cells(
        &crate::race::owner::race()
            .physics
            .contact
            .contact_turn_speed,
        &CAR.spin,
    );
    let n = n as usize;
    crate::race::owner::race().physics.contact.contact_point[0]
        .set_bits(crate::race::owner::race().physics.contact.probe_turned_x[n].bits());
    crate::race::owner::race().physics.contact.contact_point[1]
        .set_bits(crate::race::owner::race().physics.contact.probe_turned_y[n].bits());
    crate::race::owner::race().physics.contact.contact_point[2]
        .set_bits(crate::race::owner::race().physics.contact.probe_turned_z[n].bits());
    crate::race::owner::race().physics.contact.contact_point[1].set_bits(mul(
        crate::race::owner::race().physics.contact.contact_point[1].bits(),
        crate::race::owner::race().course.numbers.tenth.bits(),
    ));
    crate::race::owner::race()
        .physics
        .contact
        .contact_slide_limit
        .set_bits(0);
    contact_force(&PHYSICS.contacts[4]);
    { crate::physics::place_sound_e1::place_sound_e1() }?;
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x22400C] = (atan2(int [d + 0], int [d + 4]) * [rc + 0x1C5DD4]) * [rc + 0x1C5D98]`
fn x87_1(d: [u32; 2]) {
    let t0 = int(d[0] as i32);
    let t1 = int(d[1] as i32);
    let t2 = t0.atan2(t1);
    let t3 = ld(crate::race::owner::race()
        .course
        .numbers
        .one_over_two_pi
        .bits());
    let t4 = t2 * t3;
    let t5 = ld(crate::race::owner::race().course.numbers.k65536.bits());
    let t6 = t4 * t5;
    DRAW.scratch.set_bits(to_int(t6) as u32);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::view;

    // The original's instructions, one function per run.
    reference!(ref_0(d, rc) "fild dword ptr [{d}]", "fild dword ptr [{d} + 4]", ".byte 0xD9, 0xF3", "fmul dword ptr [{rc} + 0x1C5DD4]", "fmul dword ptr [{rc} + 0x1C5D98]", "fistp dword ptr [{rc} + 0x22400C]");

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
            reference: |a| unsafe { ref_0(a[0], a[1]) },
            plain: |a| x87_1(*view(a[0], 0)),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
