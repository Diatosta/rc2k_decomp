//! `scene::face_faces_away`: whether a face of an object is seen from behind.
//!
//! The polygon's vertex count = the face's number of vertices. Then, for the triangles of the
//! face's first point with each two that follow one another: the three points' places in the view
//! (their records 0x1000 + (number & 0xFFF), the three view floats) kept in the work area; the two
//! sides from the first point; their cross product to the work vector; and the first point's place
//! times that. A product above 0 (or no number) says this triangle faces away; the first one that
//! does not ends it with 0. The x87's arithmetic runs in `X` (plain Rust; the tests compare against
//! recorded vectors in `data/vectors`). A point's number below 0 is fatal error 0x80600314. Refuses
//! when there is no face or no records.

use crate::errors::{BadInput, FatalError, Refused};
use crate::math::x87_float::{Rounds, at_rounding};

// ---- game data ----

const BAD_POINT: FatalError = FatalError {
    code: 0x8060_0314,
    message: "illegal point num in object",
};

/// The work on one triangle: the three points' places, the sides from the first point to the
/// second and to the third, their cross product, and the x87's condition bits after comparing
/// 0 with the first point's place times that product.
struct Triangle {
    /// The places (x, y, z of each point), then the sides by axis: x to the second point, x
    /// to the third, y to the second, ...
    work: [u32; 15],
    cross: [u32; 3],
    status: u16,
}

impl Triangle {
    /// The x87's arithmetic in its order, each value stored as a float where the x87 code
    /// stores it and read back from there.
    fn of(places: [[u32; 3]; 3]) -> Triangle {
        at_rounding!(|r| Triangle::of_with(r, places))
    }

    /// `of` at the rounding `r`.
    fn of_with<R: Rounds>(r: R, places: [[u32; 3]; 3]) -> Triangle {
        let p = places.map(|point| point.map(|v| r.ld(v)));
        let side = |axis: usize, to: usize| r.st(p[to][axis] - p[0][axis]);
        let sides = [
            side(0, 1),
            side(0, 2),
            side(1, 1),
            side(1, 2),
            side(2, 1),
            side(2, 2),
        ];
        let [ax, bx, ay, by, az, bz] = sides.map(|v| r.ld(v));
        // The side to the third point times the side to the second.
        let cross = [
            r.st(bz * ay - by * az),
            r.st(bx * az - bz * ax),
            r.st(by * ax - bx * ay),
        ];
        let c = cross.map(|v| r.ld(v));
        let product = p[0][0] * c[0] + (p[0][1] * c[1] + p[0][2] * c[2]);
        let mut work = [0; 15];
        work[..9].copy_from_slice(places.as_flattened());
        work[9..].copy_from_slice(&sides);
        Triangle {
            work,
            cross,
            status: r.compare(r.of(0.0), product),
        }
    }

    /// The carry of the compare: the product above 0, or no number.
    fn faces_away(&self) -> bool {
        self.status & 0x100 != 0
    }
}

/// Whether the face (of `count` vertices, the dword at its start) is seen from behind. The
/// numbers of its points are read once into `points` when it has 3 or more vertices
/// (`scene::face_sort_key` takes them from there).
pub fn face_faces_away(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    face: usize,
    count: u32,
    points: &mut smallvec::SmallVec<[u32; 16]>,
) -> Result<bool, Refused> {
    if face == 0 || !crate::scene::point_records::held(ctx.reads.course_lists) {
        return Err(BadInput("no face or no records").into());
    }
    points.clear();
    ctx.view.polygon.polygon_vertex_count.set(count);
    if count < 3 {
        return Ok(true);
    }
    // Past the face's last vertex is refused up front, as a read of it all would be.
    let last = (face as u32)
        .checked_add(
            count
                .checked_mul(0x10)
                .ok_or(BadInput("a face of too many vertices"))?,
        )
        .ok_or(BadInput("a face of too many vertices"))?;
    ctx.reads.stage.object_faces.array::<0x10>(last)?;
    // Every vertex is in the faces now (the last one is), so reading them all up front
    // refuses nothing the loop would not.
    let mut vertices: smallvec::SmallVec<[u8; 0x10 * 16]> =
        smallvec::smallvec![0; count as usize * 0x10];
    ctx.reads
        .stage
        .object_faces
        .read_into(face as u32 + 0x10, &mut vertices)?;
    points.extend(
        vertices
            .as_chunks::<0x10>()
            .0
            .iter()
            .map(|v| u32::from_le_bytes(v[..4].try_into().unwrap())),
    );
    let point = |n: u32| Ok::<u32, Refused>(points[n as usize]);
    let table = crate::scene::reading::PointRecords::new(ctx.reads.course_lists, ctx.part)?;
    for n in 2..count {
        // Each triangle's numbers are looked at as it comes to it.
        let points = [point(0)?, point(n - 1)?, point(n)?];
        if points.iter().any(|&p| (p as i32) < 0) {
            return Err(BAD_POINT.into());
        }
        let triangle = Triangle::of([
            table.place(points[0])?,
            table.place(points[1])?,
            table.place(points[2])?,
        ]);
        ctx.view.object.borrow_mut().normal_work = triangle.work;
        crate::memory::session::write_first(&ctx.view.matrices.work_vector, &triangle.cross);
        if !triangle.faces_away() {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, differential};

    #[test]
    fn triangle_matches_the_assembly() {
        // 19 dwords: nine floats, ten written; the status keeps only its condition bits
        let kinds = {
            let mut k = [Kind::Float; 19];
            k[18] = Kind::Exact;
            k
        };
        let test = Differential {
            name: "scene.face_faces_away.triangle",
            cases: 8000,
            seed: 11,
            kinds,
            fill: &|rng, m| {
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().take(9) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
                m[9..].fill(0xAAAA_AAAA);
                m[18] = 0;
                if rng.below(6) == 0 {
                    // a flat triangle: the product is exactly zero
                    let first = [m[0], m[1], m[2]];
                    m[3..6].copy_from_slice(&first);
                }
            },
            plain: &|c| {
                let block: [u32; 19] = std::array::from_fn(|k| c[k].get());
                let places =
                    std::array::from_fn(|k| [block[3 * k], block[3 * k + 1], block[3 * k + 2]]);
                let t = Triangle::of(places);
                for (cell, &d) in c.iter().zip(t.work.iter().chain(&t.cross)) {
                    cell.set(d);
                }
                c[18].set(u32::from(t.status) & 0x4500);
            },
        };
        differential(test);
    }
}
