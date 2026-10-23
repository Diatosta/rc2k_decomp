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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EDX, ESI, Regs, game_addr, game_read, game_write, rva};
use crate::math::normalise_vector::{VECTOR_RVA, normalise_vector};
use crate::math::x87;
use crate::scene::project_point::RECORDS_RVA;
use crate::scene::reflect_vector::{
    MATRIX_RVA, NORMAL_RVA, VECTOR_RVA as MIRRORED_RVA, reflect_vector,
};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

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

/// What the function leaves in registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leftovers {
    pub eax: u32,
    pub edx: u32,
    pub esi: u32,
}

pub unsafe fn reflection_coordinates() -> Result<Leftovers, BadInput> {
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
        // Without the x87 way's sum eax is what was last loaded: the third part.
        let eax = normalise_vector()?.unwrap_or(to_eye[2]);
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
        Ok(Leftovers {
            eax,
            edx: point,
            esi: game_addr(MATRIX_RVA) as u32,
        })
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let left = unsafe { reflection_coordinates() }?;
    Ok(Left::nothing()
        .with(EAX, left.eax)
        .with(EDX, left.edx)
        .with(ESI, left.esi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
