//! `drive::animate_lit_faces` (0x45EB06): one step of the stage's moving faces, those in
//! `g_litFaces` (0x6D3220, `g_litFaceCount` of them). For each (face record at
//! `g_courseFaces` + 0x40 n):
//! - object 0x3B (byte +1): a countdown at +0x11; at 0 the face leaves the list;
//! - else its distance from car block 0 (+0x134 / +0x13C: `dword_70F520`, `dword_70F528`;
//!   `fistp` of the root of the squares, through `dword_62400C`) into `dword_6D303C`, and
//!   a pace 1 .. 17 from it (closer is faster) into `dword_6D3040`; then by its object's
//!   kind (`byte_97DF4C[g_objectKinds[object] * 0x80]`):
//!   - 1, 9: +0x11 counts up to 0x18 (then cleared, and the face leaves), +0x13 from
//!     `byte_6D1DFC`;
//!   - 2: +0x13 up by the pace, leaving at 0x30;
//!   - 3, 8: +9 stepped (the pace times 4) towards making +8 + +9 + +0x12 a multiple of
//!     0x80; once it is, as 2 up to 0x40;
//!   - 4: while +0x11 is not 0, the face's offset (+0x30, +0x38) moved by
//!     `math::turn_point_xz` of (0, +0x11 << 8) by (+0x12 + 0x80) << 24, its height
//!     (+0x34) set on the ground under it (`physics::find_ground`, last face
//!     `dword_7101EC`; no ground puts the offset back), +0x11 less 1; at 0 it leaves;
//!   - others: nothing.
//!
//! A face leaves by the list's last taking its place (the count less 1), and the next
//! one is looked at (so the one moved in waits a step). By the look of it the stage's
//! animated objects: lights blinking, things falling (a reading).
//!
//! In:  nothing
//! Out: the registers as the last face leaves them (see `animate_at`); no flags read
//!      (`sub_45E9CA` calls `physics::move_slots` next)
//!
//! Caller: `drive::animate_stage` (0x45E9CA). Refuses (before anything) more than 0x100 lit
//! faces (the list's room) and a face number past `g_courseFaceCount`; `find_ground`'s
//! refusals mid-way (what it wrote is put back first).

use crate::errors::{BadInput, Refused};
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::ground_store::GROUND_WORK;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::course_store::COURSE;
use crate::scene::draw_store::DRAW;
use crate::scene::object_tables_store::OBJECT_TABLES;
use rc2k_core::drive::lit_faces::{
    COUNTDOWN_OBJECT, FaceBytes, RISE_TOP, align, blink, count_down, fall_turn, pace, rise,
};

const MOST: u32 = 0x100;

unsafe fn at(p: u32) -> u32 {
    unsafe { (p as *const u32).read_unaligned() }
}

unsafe fn put(p: u32, v: u32) {
    unsafe { (p as *mut u32).write_unaligned(v) }
}

unsafe fn byte(p: u32) -> u8 {
    unsafe { (p as *const u8).read() }
}

unsafe fn set_byte(p: u32, v: u8) {
    unsafe { (p as *mut u8).write(v) }
}

/// The face's animated bytes.
unsafe fn load(f: u32) -> FaceBytes {
    unsafe {
        FaceBytes {
            b8: byte(f + 8),
            b9: byte(f + 9),
            b11: byte(f + 0x11),
            b12: byte(f + 0x12),
            b13: byte(f + 0x13),
        }
    }
}

unsafe fn store(f: u32, b: &FaceBytes) {
    unsafe {
        for (o, v) in [(9, b.b9), (0x11, b.b11), (0x13, b.b13)] {
            set_byte(f + o, v);
        }
    }
}

/// `g_litFaces`[k] (0x100 of them; the count is checked below).
fn list(k: u32) -> &'static crate::session::Dword {
    &CONTACT_WORK.lit_faces[k as usize]
}

/// The refusals, worked out before anything is written.
fn refusal() -> Option<&'static str> {
    let count: u32 = CONTACT_WORK.lit_face_count.bits();
    if count > MOST {
        return Some("more than 0x100 lit faces (past g_litFaces)");
    }
    let faces: u32 = COURSE.course_face_count.bits();
    for k in 0..count {
        if list(k).get() >= faces {
            return Some("a lit face past g_courseFaceCount");
        }
    }
    None
}

/// The Rust core. The registers it leaves are read by no caller.
pub unsafe fn animate_at() -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal() {
            return Err(BadInput(why).into());
        }
        if CONTACT_WORK.lit_face_count.bits() == 0 {
            return Ok(());
        }

        animate_faces()
    }
}

// Converted: `rc2k_core::drive::lit_faces` (all but kind 4's turn and ground).
unsafe fn animate_faces() -> Result<(), Refused> {
    unsafe {
        let faces: u32 = COURSE.course_faces.bits();
        let points: u32 = COURSE.scene_points.bits();
        let point = |n: u32| points.wrapping_add(n.wrapping_mul(12));
        let mut k = 0;
        loop {
            let f = faces.wrapping_add(list(k).get() << 6);
            let mut bytes = load(f);
            let leave = 'face: {
                let object = byte(f + 1);
                if object == COUNTDOWN_OBJECT {
                    break 'face count_down(&mut bytes);
                }
                let point_number = at(f + 4);
                let p = point(point_number);
                let dx = at(p).wrapping_sub(CAR_BLOCKS.cars[0].position[0].bits());
                DRAW.scratch.set(dx);
                let dz = at(p + 8).wrapping_sub(CAR_BLOCKS.cars[0].position[2].bits());
                DRAW.scratch.set(dz);
                let d = crate::math::x87::int_hypot(dx as i32, dz as i32) as u32;
                CONTACT_WORK.lit_face_distance.set(d);
                let step = pace(d);
                CONTACT_WORK.lit_face_pace.set(step);
                let kind_of: u32 = OBJECT_TABLES.object_kind(object as u32);
                let kind = ((crate::scene::polygon::materials_addr()
                    + 0x1C
                    + (kind_of.wrapping_mul(0x80)) as usize)
                    as *const u8)
                    .read();
                match kind {
                    1 | 9 => {
                        blink(&mut bytes, |n| {
                            // byte_6D1DFC: the step is below 0x18 (`BLINK_STEPS`).
                            (CONTACT_WORK.blink_levels[n as usize / 4].get() >> (8 * (n % 4))) as u8
                        })
                        .is_none()
                    }
                    2 => rise(&mut bytes, step, RISE_TOP).0,
                    3 | 8 => align(&mut bytes, step).0,
                    4 => {
                        let Some((distance, angle)) = fall_turn(&bytes) else {
                            break 'face true;
                        };
                        let [x, _, z, _] = crate::math::turn_point_xz::turn_point_xz(
                            0,
                            point_number as i32,
                            distance as i32,
                            angle,
                        );
                        CONTACT_WORK.lit_face_saved_30.set(at(f + 0x30));
                        CONTACT_WORK.lit_face_saved_38.set(at(f + 0x38));
                        put(f + 0x30, at(f + 0x30).wrapping_add(x));
                        put(f + 0x38, at(f + 0x38).wrapping_add(z));
                        let p = point(at(f + 4));
                        GROUND_WORK.ground_x.set(at(p).wrapping_add(at(f + 0x30)));
                        GROUND_WORK
                            .ground_z
                            .set(at(p + 8).wrapping_add(at(f + 0x38)));
                        COURSE
                            .ground_face_last
                            .set(CAR_BLOCKS.cars[0].probe_point_face[4].bits());
                        crate::physics::find_ground::find_ground()?;
                        put(
                            f + 0x34,
                            GROUND_WORK.ground_height.get().wrapping_sub(at(p + 4)),
                        );
                        if COURSE.ground_slot.get() == u32::MAX {
                            put(f + 0x30, CONTACT_WORK.lit_face_saved_30.bits());
                            put(f + 0x38, CONTACT_WORK.lit_face_saved_38.bits());
                        }
                        bytes.b11 = bytes.b11.wrapping_sub(1);
                        false
                    }
                    _ => false,
                }
            };
            store(f, &bytes);
            if leave {
                let last = CONTACT_WORK.lit_face_count.bits().wrapping_sub(1);
                list(k).set(list(last).get());
                CONTACT_WORK.lit_face_count.set_bits(last);
            }
            k += 1;
            if k >= CONTACT_WORK.lit_face_count.bits() {
                return Ok(());
            }
        }
    }
}

/// The Rust version.
pub unsafe fn animate_lit_faces() -> Result<(), Refused> {
    unsafe { animate_at() }
}
