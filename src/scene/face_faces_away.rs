//! `scene::face_faces_away` (0x45205B): whether a face of an object is seen from behind.
//!
//! Takes: the face
//!
//! `g_polygonVertexCount` = the face's number of vertices. Then, for the triangles of
//! the face's first point with each two that follow one another (0x4520A5): the
//! three points' places in the view (their records 0x1000 + (number & 0xFFF), +0x10
//! .. +0x18) to `dword_6396A0` ..; the two sides from the first point to
//! `dword_6396C4` ..; their cross product to the work vector; and the first point's
//! place times that. A product above 0 (or no number) says this triangle faces away;
//! the first one that does not ends it with 0. The x87's arithmetic in `X` (plain Rust since
//! 2026-10-24; the original's instructions are `asm_reference` in the tests).
//! A point's number below 0 is fatal error 0x80600314.
//! One caller, 0x45071B, which turns the answer into the carry flag.
//! Refuses when there is no face or no records.

use crate::errors::{BadInput, FatalError, Refused};
use crate::math::x87_float::{X, compare, ld, st};

// ---- game data ----

const BAD_POINT: FatalError = FatalError {
    code: 0x8060_0314,
    message: "illegal point num in object",
};

// ---- Rust version ----

/// 0x4520A5's work on one triangle, as the original leaves it at `dword_6396A0`: the three
/// points' places, the sides from the first point to the second and to the third, their
/// cross product, and the x87's condition bits after comparing 0 with the first point's place
/// times that product.
struct Triangle {
    /// The places (x, y, z of each point), then the sides by axis: x to the second point, x
    /// to the third, y to the second, ...
    work: [u32; 15],
    cross: [u32; 3],
    status: u16,
}

impl Triangle {
    /// The x87's arithmetic in its order, each value stored as a float where the original
    /// stores it (`fstp dword`) and read back from there.
    fn of(places: [[u32; 3]; 3]) -> Triangle {
        let p = places.map(|point| point.map(ld));
        let side = |axis: usize, to: usize| st(p[to][axis] - p[0][axis]);
        let sides = [
            side(0, 1),
            side(0, 2),
            side(1, 1),
            side(1, 2),
            side(2, 1),
            side(2, 2),
        ];
        let [ax, bx, ay, by, az, bz] = sides.map(ld);
        // The side to the third point times the side to the second.
        let cross = [
            st(bz * ay - by * az),
            st(bx * az - bz * ax),
            st(by * ax - bx * ay),
        ];
        let c = cross.map(ld);
        let product = p[0][0] * c[0] + (p[0][1] * c[1] + p[0][2] * c[2]);
        let mut work = [0; 15];
        work[..9].copy_from_slice(places.as_flattened());
        work[9..].copy_from_slice(&sides);
        Triangle {
            work,
            cross,
            status: compare(X(0.0), product),
        }
    }

    /// The carry of the compare: the product above 0, or no number.
    fn faces_away(&self) -> bool {
        self.status & 0x100 != 0
    }
}

pub fn face_faces_away(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    face: usize,
) -> Result<bool, Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    if face == 0 || !crate::scene::point_records::held() {
        return Err(BadInput("no face or no records").into());
    }
    let count: u32 = crate::scene::object_faces::word(face as u32)?;
    scene.drawing.scene.polygon_vertex_count.set_bits(count);
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
    crate::scene::object_faces::array::<0x10>(last)?;
    let point = |n: u32| crate::scene::object_faces::word(face as u32 + 0x10 + n * 0x10);
    let table = crate::scene::reading::PointRecords::new()?;
    for n in 2..count {
        // The original looks at each triangle's numbers as it comes to it.
        let points = [point(0)?, point(n - 1)?, point(n)?];
        if points.iter().any(|&p| (p as i32) < 0) {
            return Err(BAD_POINT.into());
        }
        let triangle = Triangle::of([
            table.place(points[0])?,
            table.place(points[1])?,
            table.place(points[2])?,
        ]);
        crate::session::write_bits(&ctx.view.object.normal_work, triangle.work);
        crate::session::write_first(&race.frame.scratch.work_vector, &triangle.cross);
        if !triangle.faces_away() {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential};

    /// The original's instructions for `triangle`.
    unsafe fn triangle_ref(w: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{w} + 12]", "fsub dword ptr [{w}]", "fstp dword ptr [{w} + 36]",
                "fld dword ptr [{w} + 24]", "fsub dword ptr [{w}]", "fstp dword ptr [{w} + 40]",
                "fld dword ptr [{w} + 16]", "fsub dword ptr [{w} + 4]", "fstp dword ptr [{w} + 44]",
                "fld dword ptr [{w} + 28]", "fsub dword ptr [{w} + 4]", "fstp dword ptr [{w} + 48]",
                "fld dword ptr [{w} + 20]", "fsub dword ptr [{w} + 8]", "fstp dword ptr [{w} + 52]",
                "fld dword ptr [{w} + 32]", "fsub dword ptr [{w} + 8]", "fstp dword ptr [{w} + 56]",
                "fld dword ptr [{w} + 56]", "fld dword ptr [{w} + 44]",
                ".byte 0xDE, 0xC9", // fmulp st(1), st
                "fld dword ptr [{w} + 48]", "fld dword ptr [{w} + 52]",
                ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9", // fsubp st(1), st
                "fstp dword ptr [{w} + 60]",
                "fld dword ptr [{w} + 40]", "fld dword ptr [{w} + 52]",
                ".byte 0xDE, 0xC9",
                "fld dword ptr [{w} + 56]", "fld dword ptr [{w} + 36]",
                ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9",
                "fstp dword ptr [{w} + 64]",
                "fld dword ptr [{w} + 48]", "fld dword ptr [{w} + 36]",
                ".byte 0xDE, 0xC9",
                "fld dword ptr [{w} + 40]", "fld dword ptr [{w} + 44]",
                ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9",
                "fstp dword ptr [{w} + 68]",
                "fld dword ptr [{w}]", "fld dword ptr [{w} + 60]",
                ".byte 0xDE, 0xC9",
                "fld dword ptr [{w} + 4]", "fld dword ptr [{w} + 64]",
                ".byte 0xDE, 0xC9",
                "fld dword ptr [{w} + 8]", "fld dword ptr [{w} + 68]",
                ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xC1", // faddp st(1), st
                ".byte 0xDE, 0xC1",
                ".byte 0xD9, 0xEE", // fldz
                ".byte 0xDE, 0xD9", // fcompp
                "fnstsw word ptr [{w} + 72]",
                w = in(reg) w,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    #[test]
    fn triangle_matches_the_assembly() {
        // 19 dwords: nine floats, ten written; the status keeps only its condition bits
        let kinds = {
            let mut k = [Kind::Float; 19];
            k[18] = Kind::Exact;
            k
        };
        differential::<19>(
            "face_faces_away triangle",
            8000,
            11,
            kinds,
            |rng, m| {
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
            |base| unsafe {
                triangle_ref(base);
                let s = base.add(72).cast::<u32>();
                s.write_unaligned(s.read_unaligned() & 0x4500);
            },
            |base| unsafe {
                let w = base.cast::<[u32; 19]>();
                let mut block = w.read_unaligned();
                let places =
                    std::array::from_fn(|k| [block[3 * k], block[3 * k + 1], block[3 * k + 2]]);
                let t = Triangle::of(places);
                block[..15].copy_from_slice(&t.work);
                block[15..18].copy_from_slice(&t.cross);
                block[18] = u32::from(t.status) & 0x4500;
                w.write_unaligned(block);
            },
        );
    }
}
