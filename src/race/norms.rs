//! `race::loadlevel_init`'s "store norms" (IDA, 2026-10-16): the course memory laid out
//! (faces of 0x40 bytes, points of 12), and each face's two normals (of its first three
//! points and of its last three) stored in it.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;

pub const WORK_VECTOR_RVA: usize = rva(0x5C5F60); // g_workVector: three floats
pub const LENGTH_RVA: usize = rva(0x62400C); // dword_62400C
const ZERO_RVA: usize = rva(0x5C5D90); // flt_5C5D90, 0.0
pub const POINTS_RVA: usize = rva(0x5CCCC0); // g_scenePoints
pub const POINT_COUNT_RVA: usize = rva(0x5CCD20); // g_scenePointCount
pub const FACES_RVA: usize = rva(0x5CCCBC); // dword_5CCCBC: the faces
pub const FACE_COUNT_RVA: usize = rva(0x5CCD1C); // dword_5CCD1C
pub const QUAD_RVA: usize = rva(0x63D9F8); // dword_63D9F8: -1 for faces of three points
/// `face_normal`'s work: the three points (9 dwords) and the six differences.
pub const NORMAL_WORK_RVA: usize = rva(0x6396A0);
const COURSE_MEMORY_RVA: usize = rva(0x5C6F94); // g_courseMemory
const LAYOUT_RVA: usize = rva(0x5CCCB8); // dword_5CCCB8 .. : the layout's five
const FIRST_COUNT_RVA: usize = rva(0x5CCD18); // dword_5CCD18: 0x20-byte records first

/// `race::normalise_work_vector` (0x41FC4D): `g_workVector` made one long by its 80-bit
/// length (`math::x87::normalise_in_place`); a length equal to 0 makes it 0; -0.0 parts
/// become 0. The length rounded into `dword_62400C`.
pub unsafe fn normalise_work_vector() {
    unsafe {
        let v = game_addr(WORK_VECTOR_RVA) as *mut u32;
        let (length, zero) = x87::normalise_in_place(v, game_read(ZERO_RVA));
        game_write(LENGTH_RVA, length);
        for n in 0..3 {
            let part = v.add(n);
            if zero || part.read_unaligned() == 0x8000_0000 {
                part.write_unaligned(0);
            }
        }
    }
}

/// `race::face_normal` (0x422144): the points `a`, `b`, `c` (eax, ebx, ecx) of
/// `g_scenePoints` kept, the differences b - a and c - a (whole numbers), their cross
/// product into `g_workVector` (each part as `math::x87::int_product_difference`),
/// normalised. Its caller (`store_face_normal`) keeps every register round it.
pub unsafe fn face_normal(a: u32, b: u32, c: u32) {
    unsafe {
        let points: u32 = game_read(POINTS_RVA);
        let point = |n: u32| -> [i32; 3] {
            ((points.wrapping_add(n.wrapping_mul(12))) as *const [i32; 3]).read_unaligned()
        };
        let (a, b, c) = (point(a), point(b), point(c));
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
        game_write(NORMAL_WORK_RVA, work);
        // c4 = d[0], c8 = d[1], cc = d[2], d0 = d[3], d4 = d[4], d8 = d[5]
        let normal = [
            x87::int_product_difference(d[5], d[2], d[3], d[4]),
            x87::int_product_difference(d[1], d[4], d[5], d[0]),
            x87::int_product_difference(d[3], d[0], d[1], d[2]),
        ];
        game_write(WORK_VECTOR_RVA, normal);
        normalise_work_vector();
    }
}

/// `race::store_face_normal` (0x421F72): face ebx. Nothing for a face whose first byte is
/// 0x10 or more; else bit 0x200 of its word +2 set when two of its points (+4, +8, +0xC,
/// and +0x10 unless `dword_63D9F8` is -1) are the same in their low twelve bits, cleared
/// otherwise; then, when all three are below `g_scenePointCount`, the normal of +4, +8,
/// +0xC (else `g_workVector` as it was) into +0x18, +0x30, +0x38, and of +0x10, +0xC, +8
/// into +0x1C, +0x34, +0x3C. Every register preserved (`face` gives the callee).
pub unsafe fn store_face_normal(face_number: u32, normal: &mut dyn FnMut(u32, u32, u32)) {
    unsafe {
        // Converted: `rc2k_core::race::faces::{takes_normals, repeated_points,
        // with_repeated, normal_made}`.
        use rc2k_core::race::faces::{
            NORMAL_TRIANGLES, normal_made, repeated_points, takes_normals, with_repeated,
        };
        let faces: u32 = game_read(FACES_RVA);
        let face = faces.wrapping_add(face_number << 6);
        let dword = |o: u32| ((face + o) as *const u32).read_unaligned();
        if !takes_normals(*(face as *const u8)) {
            return;
        }
        let corners = [4, 8, 0xC, 0x10].map(dword);
        let repeated = repeated_points(corners, || game_read::<u32>(QUAD_RVA) != u32::MAX);
        let flags = (face + 2) as *mut u16;
        flags.write_unaligned(with_repeated(flags.read_unaligned(), repeated));
        let count: u32 = game_read(POINT_COUNT_RVA);
        for (points, places) in NORMAL_TRIANGLES {
            let [a, b, c] = points.map(|k| corners[k]);
            if normal_made([a, b, c], count) {
                normal(a, b, c);
            }
            let work: [u32; 3] = game_read(WORK_VECTOR_RVA);
            for (place, value) in places.into_iter().zip(work) {
                ((face + place) as *mut u32).write_unaligned(value);
            }
        }
    }
}

/// `race::lay_out_course` (0x421BE3): from `g_courseMemory` + 0x100, `dword_5CCD18`
/// records of 0x20, the faces (`dword_5CCCBC`, `dword_5CCD1C` of 0x40), the points
/// (`g_scenePoints`, `g_scenePointCount` of 12), and what follows (`dword_5CCCC4`, and its
/// offset in `dword_5CCCC8`); the first at `dword_5CCCB8`. Every register preserved.
pub unsafe fn lay_out_course() {
    unsafe {
        // Converted: `rc2k_core::race::faces::CourseLayout`.
        let layout = rc2k_core::race::faces::CourseLayout::of(
            game_read(COURSE_MEMORY_RVA),
            game_read(FIRST_COUNT_RVA),
            game_read(FACE_COUNT_RVA),
            game_read(POINT_COUNT_RVA),
        );
        game_write(LAYOUT_RVA, layout.dwords());
    }
}

/// `race::store_norms` (0x421F02): `lay_out_course`, then `store_face_normal` for every
/// face. Refuses no faces (the original counts round) or 0x100000 or more.
pub unsafe fn store_norms(normal: &mut dyn FnMut(u32, u32, u32)) -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::race::faces::check_face_count`.
        let count: u32 = game_read(FACE_COUNT_RVA);
        rc2k_core::race::faces::check_face_count(count)?;
        lay_out_course();
        for face in 0..count {
            store_face_normal(face, normal);
        }
    }
    Ok(())
}

/// In the game: `face_normal` in Rust.
pub fn game_normal(a: u32, b: u32, c: u32) {
    unsafe { face_normal(a, b, c) };
}

// ---- for the self-tests ----
