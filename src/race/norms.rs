//! The stage load's "store norms" step: each face's two normals (of its first three points and of
//! its last three) worked out over the loaded course by `face_normals`, which returns them, and
//! stored in the faces by `store_norms`.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::numbers_store::Numbers;
use crate::race::owner::Race;
use crate::race::scratch_store::FrameScratch;

/// The vector `v` made one long by its 80-bit length (`math::x87::normalise_in_place`); a length
/// of 0 makes it 0; -0.0 parts become 0. Gives the vector's bits and the length's bits.
pub fn normalised(numbers: &Numbers, v: [u32; 3]) -> ([u32; 3], u32) {
    let mut v = v;
    let (length, zero) = x87::normalise_in_place(&mut v, numbers.zero);
    for part in &mut v {
        if zero || *part == 0x8000_0000 {
            *part = 0;
        }
    }
    (v, length.to_bits())
}

/// Makes `work_vector` one long (`normalised`); the length goes into `scratch`.
pub fn normalise_work_vector(
    numbers: &Numbers,
    scratch: &mut FrameScratch,
    scene: &crate::scene::owner::Scene,
) {
    let (v, length) = normalised(numbers, scratch.work_vector);
    scratch.work_vector = v;
    scene.draw_work_mut().scratch = length;
}

/// A triangle's normal worked out from its points.
struct TriangleNormal {
    /// The normalised normal's bits.
    normal: [u32; 3],
    /// The normal's length before it was made one long, bits.
    length: u32,
}

/// The normal of the triangle of scene points `a`, `b`, `c`: the cross product of the differences
/// b - a and c - a (whole numbers, each part as `math::x87::int_product_difference`), normalised.
fn triangle_normal(race: &Race, [a, b, c]: [u32; 3]) -> Result<TriangleNormal, BadInput> {
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
    let cross = [
        x87::int_product_difference(d[5], d[2], d[3], d[4]),
        x87::int_product_difference(d[1], d[4], d[5], d[0]),
        x87::int_product_difference(d[3], d[0], d[1], d[2]),
    ]
    .map(f32::to_bits);
    let (normal, length) = normalised(&race.course.numbers, cross);
    Ok(TriangleNormal { normal, length })
}

/// One face's results.
#[derive(PartialEq, Eq, Debug)]
pub struct FaceNormals {
    pub face: u32,
    /// The face's switches word with the repeated-points bit set or cleared.
    pub switches: u16,
    /// The vectors stored for the two triangles (x, y, z bits; the earlier face's when a
    /// triangle's points are past the scene's points).
    pub normals: [[u32; 3]; 2],
}

/// The faces' normals and what the work leaves in shared scratch.
#[derive(PartialEq, Eq, Debug)]
pub struct Normals {
    /// The faces that take normals, in face order.
    pub faces: Vec<FaceNormals>,
    /// The work vector at the end.
    pub work_vector: [u32; 3],
    /// The last length the work left in the scene's scratch word, if any.
    pub scratch: Option<u32>,
}

/// Every face's normals, starting from the work vector `work_vector`: a triangle whose points are
/// not all below the scene's point count keeps the vector the last one left. Reads the loaded
/// course and writes nothing. Refused: no faces, 0x100000 or more, or a read past a table.
fn face_normals(
    race: &Race,
    scratch: &crate::scene::scratch_store::DrawingScratch,
    work_vector: [u32; 3],
) -> Result<Normals, Refused> {
    use rc2k_core::race::faces::{
        NORMAL_TRIANGLES, normal_made, repeated_points, takes_normals, with_repeated,
    };
    let count: u32 = race.course.course.borrow().course_face_count;
    rc2k_core::race::faces::check_face_count(count)?;
    let points: u32 = race.course.course.borrow().scene_point_count;
    // The fourth corner the last drawn face left is -1 for a face of three points.
    let fourth_corner_set = crate::race::face_work::last_fourth_corner(scratch) != u32::MAX;
    let mut out = Normals {
        faces: Vec::new(),
        work_vector,
        scratch: None,
    };
    for face in 0..count {
        let f = crate::race::course_places::face_number(race.course.reads(), face)?;
        if !takes_normals(f.kind()) {
            continue;
        }
        let corners = [0, 1, 2, 3].map(|n| f.point(n));
        let repeated = repeated_points(corners, || fourth_corner_set);
        let mut normals = [[0; 3]; 2];
        for (t, (triangle, _)) in NORMAL_TRIANGLES.iter().enumerate() {
            let [a, b, c] = triangle.map(|k| corners[k]);
            if normal_made([a, b, c], points) {
                let made = triangle_normal(race, [a, b, c])?;
                out.work_vector = made.normal;
                out.scratch = Some(made.length);
            }
            normals[t] = out.work_vector;
        }
        out.faces.push(FaceNormals {
            face,
            switches: with_repeated(f.switches(), repeated),
            normals,
        });
    }
    Ok(out)
}

/// Works out the normals (`face_normals`) and stores them in the faces and
/// the work vector, plane work and scratch word the work leaves.
/// Refuses no faces or 0x100000 or more.
pub(super) fn store_norms(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let count: u32 = race.course.course.borrow().course_face_count;
    rc2k_core::race::faces::check_face_count(count)?;
    let work_vector = race.frame.scratch().work_vector;
    let normals = face_normals(race, &scene.scratch, work_vector)?;
    for made in &normals.faces {
        crate::race::course_places::update_face_number(&race.course, made.face, |f| {
            f.set_switches(made.switches);
        })?;
        for ((_, places), vector) in rc2k_core::race::faces::NORMAL_TRIANGLES
            .iter()
            .zip(made.normals)
        {
            for (&place, value) in places.iter().zip(vector) {
                crate::race::course_places::write_face_field(
                    race.course.reads(),
                    made.face,
                    place,
                    value,
                )?;
            }
        }
    }
    race.frame.scratch_mut().work_vector = normals.work_vector;
    if let Some(length) = normals.scratch {
        scene.draw_work_mut().scratch = length;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A face of kind 0 with its four corner points.
    fn face(corners: [u32; 4]) -> [u8; 64] {
        let mut face = [0u8; 64];
        for (k, corner) in corners.into_iter().enumerate() {
            face[4 + 4 * k..8 + 4 * k].copy_from_slice(&corner.to_le_bytes());
        }
        face
    }

    #[test]
    fn a_triangle_past_the_points_keeps_the_last_normal() {
        // The first triangle lies in the x-z plane; the second names point 3, past the three.
        let faces = [face([0, 1, 2, 3])];
        let points = [[0, 0, 0], [100, 0, 0], [0, 0, 100]];
        let normals = crate::race::best_line::with_course(&faces, &points, |race| {
            face_normals(race, &crate::owners::Owners::the().scene.scratch, [7, 8, 9]).unwrap()
        });
        let up = [0, 0xBF80_0000, 0];
        assert_eq!(normals.faces.len(), 1);
        assert_eq!(normals.faces[0].normals, [up, up]);
        assert_eq!(normals.work_vector, up);
    }

    #[test]
    fn without_a_triangle_the_start_vector_stays() {
        let faces = [face([5, 6, 7, 8])];
        let normals = crate::race::best_line::with_course(&faces, &[[0; 3]], |race| {
            face_normals(race, &crate::owners::Owners::the().scene.scratch, [7, 8, 9]).unwrap()
        });
        assert_eq!(normals.faces[0].normals, [[7, 8, 9]; 2]);
        assert_eq!(normals.scratch, None);
    }
}
