//! The stage load's "store norms" step: the course memory laid out (faces of 0x40 bytes, points
//! of 12), and each face's two normals (of its first three points and of its last three) stored
//! in it.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::numbers_store::Numbers;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::scratch_store::FrameScratch;

/// Makes `work_vector` one long by its 80-bit length (`math::x87::normalise_in_place`); a length
/// of 0 makes it 0; -0.0 parts become 0. The length, rounded, goes into `scratch`.
pub fn normalise_work_vector(
    numbers: &Numbers,
    scratch: &FrameScratch,
    scene: &crate::scene::owner::Scene,
) {
    let mut v = scratch
        .work_vector
        .each_ref()
        .map(crate::session::PlainDword::get);
    let (length, zero) =
        x87::normalise_in_place(v.first_chunk_mut::<3>().expect("four parts"), &numbers.zero);
    for (cell, &bits) in scratch.work_vector[..3].iter().zip(&v) {
        cell.set(bits);
    }
    scene.drawing.scene.get().scratch.set(length.to_bits());
    for part in &scratch.work_vector[..3] {
        if zero || part.get() == 0x8000_0000 {
            part.set(0u32);
        }
    }
}

/// Takes the points `a`, `b`, `c` of `scene_points`, makes the differences b - a and c - a (whole
/// numbers) and puts their cross product into `work_vector` (each part as
/// `math::x87::int_product_difference`), normalised.
pub fn face_normal(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    a: u32,
    b: u32,
    c: u32,
) -> Result<(), BadInput> {
    {
        let point = |n: u32| -> Result<[i32; 3], BadInput> {
            Ok(crate::race::scene_points::point_number(
                race.course.reads(),
                &crate::race::scene_points::no_own(),
                n,
            )?
            .map(|v| v as i32))
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
        crate::session::write_bits(&race.physics.ground().normal_work, work.map(|v| v as u32));

        let normal = [
            x87::int_product_difference(d[5], d[2], d[3], d[4]),
            x87::int_product_difference(d[1], d[4], d[5], d[0]),
            x87::int_product_difference(d[3], d[0], d[1], d[2]),
        ]
        .map(f32::to_bits);
        crate::session::write_first(&race.frame.scratch().work_vector, &normal);
        normalise_work_vector(race.course.numbers.get(), race.frame.scratch(), scene);
    }
    Ok(())
}

/// Stores a face's normals. Nothing for a face whose first byte is 0x10 or more; else bit 0x200
/// of its word +2 is set when two of its points (+4, +8, +0xC, and +0x10 unless the view
/// scratch's fourth corner is -1) are the same in their low twelve bits, cleared otherwise; then,
/// when all three are below `scene_point_count`, the normal of +4, +8, +0xC (else `work_vector` as
/// it was) goes into +0x18, +0x30, +0x38, and that of +0x10, +0xC, +8 into +0x1C, +0x34, +0x3C.
pub fn store_face_normal(
    race: &Race,
    face_number: u32,
    normal: &mut dyn FnMut(u32, u32, u32) -> Result<(), BadInput>,
) -> Result<(), BadInput> {
    {
        // Logic in `rc2k_core::race::faces::{takes_normals, repeated_points,
        // with_repeated, normal_made}`.
        use rc2k_core::race::faces::{
            NORMAL_TRIANGLES, normal_made, repeated_points, takes_normals, with_repeated,
        };
        let face = face_number;
        let f = crate::race::course_places::face_number(race.course.reads(), face)?;
        if !takes_normals(f.kind()) {
            return Ok(());
        }
        let corners = [0, 1, 2, 3].map(|n| f.point(n));
        let repeated = repeated_points(
            corners,
            || crate::race::face_work::last_fourth_corner(&race.shared) != u32::MAX, /* dword_63D9F8: -1 for faces of three points, the last face drawn's (the original's slip, kept) */
        );
        crate::race::course_places::update_face_number(&race.course, face, |f| {
            f.set_switches(with_repeated(f.switches(), repeated));
        })?;
        let count: u32 = race.course.course.get().scene_point_count.get();
        for (points, places) in NORMAL_TRIANGLES {
            let [a, b, c] = points.map(|k| corners[k]);
            if normal_made([a, b, c], count) {
                normal(a, b, c)?;
            }
            let work: [u32; 3] = crate::session::read_first(&race.frame.scratch().work_vector);
            for (place, value) in places.into_iter().zip(work) {
                crate::race::course_places::write_face_field(
                    race.course.reads(),
                    face,
                    place,
                    value,
                )?;
            }
        }
    }
    Ok(())
}

/// Names the course's parts in the course store: the records (`course_records`), the faces
/// (`course_faces`) and the scene points (`scene_points`).
pub fn lay_out_course(course: &Course) {
    let c = course.course.get();
    // The parts are tables of their own (`race::course_places`, `race::scene_points`): their
    // places are named while they are held, none otherwise.
    let held = crate::race::course_places::held();
    c.course_records.set(if held {
        crate::race::course_places::records_base()
    } else {
        0
    });
    c.course_faces.set(if held {
        crate::race::course_places::faces_base()
    } else {
        0
    });
    c.scene_points.set(if crate::race::scene_points::held() {
        crate::race::scene_points::base()
    } else {
        0
    });
}

/// Runs `lay_out_course`, then `store_face_normal` for every face.
/// Refuses no faces or 0x100000 or more.
pub fn store_norms(
    race: &Race,
    normal: &mut dyn FnMut(u32, u32, u32) -> Result<(), BadInput>,
) -> Result<(), Refused> {
    {
        // Logic in `rc2k_core::race::faces::check_face_count`.
        let count: u32 = race.course.course.get().course_face_count.get();
        rc2k_core::race::faces::check_face_count(count)?;
        lay_out_course(&race.course);
        for face in 0..count {
            store_face_normal(race, face, normal)?;
        }
    }
    Ok(())
}
