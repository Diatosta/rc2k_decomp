//! `math::normalise_vector` (0x45C3FE): the work vector made one long.
//!
//! In:  nothing
//! Out: without SSE eax = the squares' sum as a float's bits (the original pops it
//!      from the slot it kept eax in); everything else preserved
//!
//! The vector is the three floats at `xmmword_5C5F60`; the original leaves its length in
//! `flt_6D18C8` (not kept here: nothing reached reads it). Two ways, by the switch `dword_621A58` (a key's toggle and the
//! processor's SSE together, set at 0x43B471; 0 as the game starts, so the x87 way is
//! the one in use):
//! - x87: the squares' sum as a float; its square root from two tables (the float's
//!   bits 15 .. 22 pick one of 0x100 at `flt_5C70F8`, its exponent one at
//!   `flt_5C74F8`), the two multiplied; each of the three times 1 over that;
//!   `dword_62400C` = 1 over the length; a component that comes out as minus zero is
//!   made zero. A length that compares equal to `flt_5C5D90` (0.0), or is no number,
//!   ends the original at once with the length left on the x87's stack: refused.
//! - SSE: the four floats squared to `xmmword_5C5F70`, the first three added,
//!   `rsqrtss`, `rcpss` of that to `flt_6D18C8`, and all four (the fourth too) times
//!   the `rsqrtss`. Both instructions are approximations that differ between
//!   processors: the instructions themselves are used.
//!
//! Three callers. In two pieces in the image (the entry lies after the bodies).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{game_read, game_write, rva};

// ---- game data ----

// Still the image's: the work vector and the SSE switch (the race's), the root tables
// (`scene::fill_engine_tables` makes them), `dword_62400C` (scratch many functions share).
pub const VECTOR_RVA: usize = rva(0x5C5F60); // xmmword_5C5F60
const SSE_RVA: usize = rva(0x621A58); // dword_621A58
const ROOTS_RVA: usize = rva(0x5C70F8); // flt_5C70F8
const EXPONENT_ROOTS_RVA: usize = rva(0x5C74F8); // flt_5C74F8
const CHECKED_RVA: usize = rva(0x62400C); // dword_62400C
// The squares (`xmmword_5C5F70`) and the length (`g_workVectorLength`, 0x6D18C8) are no
// longer stored (2026-10-24, phase 5): no function the game reaches reads them (IDA: the
// length's one reader, 0x4517F3, is unreached).
/// `flt_5C5D90`: 0.0.
const NOTHING: u32 = 0;

// ---- Rust version ----

/// Returns what the function leaves in eax, if it changes it.
pub unsafe fn normalise_vector() -> Result<Option<u32>, BadInput> {
    unsafe { normalise(game_read::<u32>(SSE_RVA) != 0) }
}

/// Exactly as the original (the x87 at its precision, the SSE instructions): what the
/// Rust callers in modules not yet converted use.
pub unsafe fn normalise(sse: bool) -> Result<Option<u32>, BadInput> {
    unsafe {
        let v: [u32; 4] = game_read(VECTOR_RVA);
        if sse {
            // The vector, the squares, the length.
            let mut w = [v[0], v[1], v[2], v[3], 0, 0, 0, 0, 0];
            core::arch::asm!(
                "movups xmm0, [{w}]",
                "mulps xmm0, xmm0",
                "movups [{w} + 16], xmm0",
                "addss xmm0, dword ptr [{w} + 20]",
                "addss xmm0, dword ptr [{w} + 24]",
                "rsqrtss xmm0, xmm0",
                "movaps xmm1, xmm0",
                "rcpss xmm1, xmm1",
                "shufps xmm0, xmm0, 0",
                "movss dword ptr [{w} + 32], xmm1",
                "movups xmm2, [{w}]",
                "mulps xmm0, xmm2",
                "movups [{w}], xmm0",
                w = in(reg) w.as_mut_ptr(),
                out("xmm0") _, out("xmm1") _, out("xmm2") _,
                options(nostack),
            );
            game_write(VECTOR_RVA, [w[0], w[1], w[2], w[3]]);
            return Ok(None);
        }
        let mut sum = 0u32;
        core::arch::asm!(
            "fld dword ptr [{v}]",
            ".byte 0xD8, 0xC8", // fmul st, st
            "fld dword ptr [{v} + 4]",
            ".byte 0xD8, 0xC8", // fmul st, st
            "fld dword ptr [{v} + 8]",
            ".byte 0xD8, 0xC8", // fmul st, st
            ".byte 0xDE, 0xC1", // faddp st(1), st
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{s}]",
            v = in(reg) v.as_ptr(),
            s = in(reg) &raw mut sum,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        let root: u32 = game_read(ROOTS_RVA + ((sum >> 15) & 0xFF) as usize * 4);
        let exponent_root: u32 =
            game_read(EXPONENT_ROOTS_RVA + ((sum & 0x7F80_0000) >> 23) as usize * 4);
        // The product of two floats is exact in a double, as it is in the x87.
        let length = f32::from_bits(root) as f64 * f32::from_bits(exponent_root) as f64;
        let nothing = f32::from_bits(NOTHING) as f64;
        if length
            .partial_cmp(&nothing)
            .is_none_or(|order| order.is_eq())
        {
            return Err(BadInput("a vector of no length"));
        }
        // The vector, the two roots, the length, 1 over it.
        let mut w = [v[0], v[1], v[2], root, exponent_root, 0, 0];
        core::arch::asm!(
            "fld dword ptr [{w} + 12]",
            "fmul dword ptr [{w} + 16]",
            "fst dword ptr [{w} + 20]",
            "fld1",
            ".byte 0xDE, 0xF1", // fdivrp st(1), st
            "fld dword ptr [{w}]",
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            "fstp dword ptr [{w}]",
            "fld dword ptr [{w} + 4]",
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            "fstp dword ptr [{w} + 4]",
            "fld dword ptr [{w} + 8]",
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            "fstp dword ptr [{w} + 8]",
            "fstp dword ptr [{w} + 24]",
            w = in(reg) w.as_mut_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        let unsigned_zero = |bits: u32| if bits == 0x8000_0000 { 0 } else { bits };
        game_write(VECTOR_RVA, [w[0], w[1], w[2]].map(unsigned_zero));
        game_write(CHECKED_RVA, w[6]);
        Ok(Some(sum))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn nothing_is_the_games() {
        let d = crate::moved::image_bytes(0x5C5D90, 4);
        assert_eq!(u32::from_le_bytes(d.try_into().unwrap()), super::NOTHING);
    }
}
