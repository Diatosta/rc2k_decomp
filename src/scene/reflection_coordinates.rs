//! `scene::reflection_coordinates` (0x45198C): where on the reflection's picture a
//! vertex of a shiny surface looks.
//!
//! In:  nothing; `dword_6399A8` = the number of the vertex's normal, `dword_6399AC` =
//!      the number of its point
//! Out: `dword_6399B0`, `dword_6399B4` = the two texture coordinates as whole numbers;
//!      eax, edx and esi as its parts leave them (eax of the second
//!      `math::normalise_vector`, edx = the point's record, esi = the matrix of
//!      `scene::reflect_vector`); everything else preserved
//!
//! The normal's place in the view (record 0x1000 + (number & 0xFFF), +0x10 .. +0x18)
//! goes to `dword_5EA428` ..; the point's, negated, to the work vector, which is made
//! one long and copied to `qword_5EA438` ..: the direction from the point to the eye.
//! `scene::reflect_vector`; the result, with `flt_5C5D38` (1.0) added to its second
//! part, is made one long; then each of its first and third parts, times
//! `flt_5C5E40` (0.5) plus the same, times `flt_5C5D98` (65536) times `flt_5C5E2C`
//! (256), rounded (`fistp`), is a coordinate.
//! Callers: `scene::cached_reflection_coordinates`, by a call and by a jump.
//! Refuses when there are no records and what `math::normalise_vector` refuses.

use crate::errors::BadInput;
use crate::framework::{game_read, game_write, rva};
use crate::math::normalise_vector::{VECTOR_RVA, normalise_vector};
use crate::math::x87;
use crate::scene::project_point::RECORDS_RVA;
use crate::scene::reflect_vector::{NORMAL_RVA, VECTOR_RVA as MIRRORED_RVA, reflect_vector};

// ---- game data ----

pub const NORMAL_NUMBER_RVA: usize = rva(0x6399A8); // dword_6399A8
pub const POINT_NUMBER_RVA: usize = rva(0x6399AC); // dword_6399AC
pub const COORDINATES_RVA: usize = rva(0x6399B0); // dword_6399B0, dword_6399B4
const LIFT_RVA: usize = rva(0x5C5D38); // flt_5C5D38
const HALF_RVA: usize = rva(0x5C5E40); // flt_5C5E40
const SCALE_A_RVA: usize = rva(0x5C5D98); // flt_5C5D98
const SCALE_B_RVA: usize = rva(0x5C5E2C); // flt_5C5E2C

// ---- Rust version ----

/// `fld a; fmul half; fadd half; fmul scale_a; fmul scale_b; fistp`.
fn coordinate(a: u32, half: u32, scales: [u32; 2]) -> i32 {
    let (v, mut out) = ([a, half, scales[0], scales[1]], 0i32);
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{v}]",
            "fmul dword ptr [{v} + 4]",
            "fadd dword ptr [{v} + 4]",
            "fmul dword ptr [{v} + 8]",
            "fmul dword ptr [{v} + 12]",
            "fistp dword ptr [{o}]",
            v = in(reg) v.as_ptr(),
            o = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    out
}

/// `fld a; fchs; fstp`.
fn negated(a: u32) -> u32 {
    let mut out = 0u32;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{a}]",
            "fchs",
            "fstp dword ptr [{o}]",
            a = in(reg) &raw const a,
            o = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    out
}

pub unsafe fn reflection_coordinates() -> Result<(), BadInput> {
    unsafe {
        let records: u32 = game_read(RECORDS_RVA);
        if records == 0 {
            return Err(BadInput("no records for the points"));
        }
        let record = |number: u32| records.wrapping_add(((number & 0xFFF) + 0x1000) << 5);
        let place = |record: u32| ((record as usize + 0x10) as *const [u32; 3]).read_unaligned();
        let normal = record(game_read(NORMAL_NUMBER_RVA));
        game_write(MIRRORED_RVA, place(normal));
        let point = record(game_read::<u32>(POINT_NUMBER_RVA).wrapping_neg());
        game_write(VECTOR_RVA, place(point).map(negated));
        normalise_vector()?;
        let to_eye: [u32; 3] = game_read(VECTOR_RVA);
        game_write(NORMAL_RVA, to_eye);
        reflect_vector();
        let mirrored: [u32; 3] = game_read(MIRRORED_RVA);
        game_write(
            VECTOR_RVA,
            [
                x87::loaded(mirrored[0]),
                x87::add(mirrored[1], game_read(LIFT_RVA)),
                x87::loaded(mirrored[2]),
            ],
        );
        normalise_vector()?;
        let looks: [u32; 3] = game_read(VECTOR_RVA);
        let half: u32 = game_read(HALF_RVA);
        let scales = [game_read(SCALE_A_RVA), game_read(SCALE_B_RVA)];
        game_write(
            COORDINATES_RVA,
            [
                coordinate(looks[0], half, scales),
                coordinate(looks[2], half, scales),
            ],
        );
        Ok(())
    }
}
