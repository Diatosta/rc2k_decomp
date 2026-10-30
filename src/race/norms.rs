//! `race::loadlevel_init`'s "store norms" (IDA, 2026-10-16): the course memory laid out
//! (faces of 0x40 bytes, points of 12), and each face's two normals (of its first three
//! points and of its last three) stored in it.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::math::x87;
use crate::physics::ground_store::GROUND_WORK;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::numbers_store::NUMBERS;
use crate::race::scratch_store::FRAME_SCRATCH;
use crate::race::view_store::RACE_VIEW;
use crate::scene::draw_store::DRAW;

/// `race::normalise_work_vector` (0x41FC4D): `g_workVector` made one long by its 80-bit
/// length (`math::x87::normalise_in_place`); a length equal to 0 makes it 0; -0.0 parts
/// become 0. The length rounded into `dword_62400C`.
pub fn normalise_work_vector() {
    let mut v = FRAME_SCRATCH.work_vector.each_ref().map(|c| c.bits());
    let (length, zero) = x87::normalise_in_place(
        v.first_chunk_mut::<3>().expect("four parts"),
        NUMBERS.zero.bits(),
    );
    for (cell, &bits) in FRAME_SCRATCH.work_vector[..3].iter().zip(&v) {
        cell.set_bits(bits);
    }
    DRAW.scratch.set_bits(length);
    for part in &FRAME_SCRATCH.work_vector[..3] {
        if zero || part.bits() == 0x8000_0000 {
            part.set_bits(0u32);
        }
    }
}

/// `race::face_normal` (0x422144): the points `a`, `b`, `c` (eax, ebx, ecx) of
/// `g_scenePoints` kept, the differences b - a and c - a (whole numbers), their cross
/// product into `g_workVector` (each part as `math::x87::int_product_difference`),
/// normalised. Its caller (`store_face_normal`) keeps every register round it.
pub fn face_normal(a: u32, b: u32, c: u32) -> Result<(), BadInput> {
    {
        let point = |n: u32| -> Result<[i32; 3], BadInput> {
            Ok(
                crate::race::scene_points::point(crate::race::scene_points::place(n))?
                    .map(|v| v as i32),
            )
        };
        let (a, b, c) = (point(a)?, point(b)?, point(c)?);
        let d = [
            b[0].wrapping_sub(a[0]),
            c[0].wrapping_sub(a[0]),
            b[1].wrapping_sub(a[1]),
            c[1].wrapping_sub(a[1]),
            b[2].wrapping_sub(a[2]),
            c[2].wrapping_sub(a[2]),
        ];
        let mut work = [0i32; 15];
        work[..3].copy_from_slice(&a);
        work[3..6].copy_from_slice(&b);
        work[6..9].copy_from_slice(&c);
        work[9..].copy_from_slice(&d);
        crate::session::write_bits(&GROUND_WORK.normal_work, work.map(|v| v as u32));
        // c4 = d[0], c8 = d[1], cc = d[2], d0 = d[3], d4 = d[4], d8 = d[5]
        let normal = [
            x87::int_product_difference(d[5], d[2], d[3], d[4]),
            x87::int_product_difference(d[1], d[4], d[5], d[0]),
            x87::int_product_difference(d[3], d[0], d[1], d[2]),
        ];
        crate::session::write_first(&FRAME_SCRATCH.work_vector, &normal);
        normalise_work_vector();
    }
    Ok(())
}

/// `race::store_face_normal` (0x421F72): face ebx. Nothing for a face whose first byte is
/// 0x10 or more; else bit 0x200 of its word +2 set when two of its points (+4, +8, +0xC,
/// and +0x10 unless `dword_63D9F8` is -1) are the same in their low twelve bits, cleared
/// otherwise; then, when all three are below `g_scenePointCount`, the normal of +4, +8,
/// +0xC (else `g_workVector` as it was) into +0x18, +0x30, +0x38, and of +0x10, +0xC, +8
/// into +0x1C, +0x34, +0x3C. Every register preserved (`face` gives the callee).
pub fn store_face_normal(
    face_number: u32,
    normal: &mut dyn FnMut(u32, u32, u32) -> Result<(), BadInput>,
) -> Result<(), BadInput> {
    {
        // Converted: `rc2k_core::race::faces::{takes_normals, repeated_points,
        // with_repeated, normal_made}`.
        use rc2k_core::race::faces::{
            NORMAL_TRIANGLES, normal_made, repeated_points, takes_normals, with_repeated,
        };
        let faces: u32 = COURSE.course_faces.bits();
        let face = faces.wrapping_add(face_number << 6);
        let what = "a course face";
        let f = crate::race::course_places::face(face, what)?;
        if !takes_normals(f.kind()) {
            return Ok(());
        }
        let corners = [0, 1, 2, 3].map(|n| f.point(n));
        let repeated = repeated_points(
            corners,
            || RACE_VIEW.corners[3].bits() != u32::MAX, /* dword_63D9F8: -1 for faces of three points */
        );
        crate::race::course_places::update_face(face, what, |f| {
            f.set_switches(with_repeated(f.switches(), repeated))
        })?;
        let count: u32 = COURSE.scene_point_count.bits();
        for (points, places) in NORMAL_TRIANGLES {
            let [a, b, c] = points.map(|k| corners[k]);
            if normal_made([a, b, c], count) {
                normal(a, b, c)?;
            }
            let work: [u32; 3] = crate::session::read_first(&FRAME_SCRATCH.work_vector);
            for (place, value) in places.into_iter().zip(work) {
                crate::race::course_places::write(face + place, value, "a course face normal")?;
            }
        }
    }
    Ok(())
}

/// `race::lay_out_course` (0x421BE3): from `g_courseMemory` + 0x100, `dword_5CCD18`
/// records of 0x20, the faces (`dword_5CCCBC`, `dword_5CCD1C` of 0x40), the points
/// (`g_scenePoints`, `g_scenePointCount` of 12), and what follows (`dword_5CCCC4`, and its
/// offset in `dword_5CCCC8`); the first at `dword_5CCCB8`. Every register preserved.
pub fn lay_out_course() {
    {
        // Converted: `rc2k_core::race::faces::CourseLayout`.
        let layout = rc2k_core::race::faces::CourseLayout::of(
            COURSE_LISTS.course_memory.bits(),
            COURSE.course_record_count.bits(),
            COURSE.course_face_count.bits(),
            COURSE.scene_point_count.bits(),
        );
        let [records, faces, points, after, after_offset] = layout.dwords();
        let c = &COURSE;
        // The faces and records are Rust's too (`race::course_places`).
        if crate::race::course_places::held() {
            c.course_records
                .set(crate::race::course_places::records_base());
            c.course_faces.set(crate::race::course_places::faces_base());
        } else {
            c.course_records.set(records);
            c.course_faces.set(faces);
        }
        // The points are Rust's (`race::scene_points`, moved out by `load_level`): their
        // place, not the address the layout gives.
        let _ = points;
        c.scene_points.set(if crate::race::scene_points::held() {
            crate::race::scene_points::base()
        } else {
            0
        });
        // The census of phase 13 (`pointers::watch` on the records, faces, info and the
        // points' old home) ran 2026-10-27: `notes/course_census.md`. Off since.
        c.course_tail[0].set(after);
        c.course_tail[1].set(after_offset);
    }
}

/// `race::store_norms` (0x421F02): `lay_out_course`, then `store_face_normal` for every
/// face. Refuses no faces (the original counts round) or 0x100000 or more.
pub fn store_norms(
    normal: &mut dyn FnMut(u32, u32, u32) -> Result<(), BadInput>,
) -> Result<(), Refused> {
    {
        // Converted: `rc2k_core::race::faces::check_face_count`.
        let count: u32 = COURSE.course_face_count.bits();
        rc2k_core::race::faces::check_face_count(count)?;
        lay_out_course();
        for face in 0..count {
            store_face_normal(face, normal)?;
        }
    }
    Ok(())
}

/// In the game: `face_normal` in Rust.
pub fn game_normal(a: u32, b: u32, c: u32) -> Result<(), BadInput> {
    face_normal(a, b, c)
}
