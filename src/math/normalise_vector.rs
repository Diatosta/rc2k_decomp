//! `math::normalise_vector` (0x45C3FE): the work vector made one long.
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

use crate::errors::BadInput;
use crate::math::x87_float::{ONE, compare, ld, st};
use crate::race::owner::Race;
use crate::scene::owner::scene;

// ---- game data ----

// Still the image's: the work vector and the SSE switch (the race's), the root tables
// (`scene::fill_engine_tables` makes them), `dword_62400C` (scratch many functions share).
// The squares (`xmmword_5C5F70`) and the length (`g_workVectorLength`, 0x6D18C8) are no
// longer stored (2026-10-24, phase 5): no function the game reaches reads them (IDA: the
// length's one reader, 0x4517F3, is unreached).
/// `flt_5C5D90`: 0.0.
const NOTHING: u32 = 0;

// ---- Rust version ----

/// The squares' sum as a float (`fld v0; fmul st, st; fld v1; fmul st, st; fld v2; fmul
/// st, st; faddp; faddp; fstp`): `v0 v0 + (v1 v1 + v2 v2)` at double precision, stored
/// as a float, rounded as the control word says.
pub(crate) fn squares_sum(v: [u32; 3]) -> u32 {
    let (a, b, c) = (
        ld(v[0]) * ld(v[0]),
        ld(v[1]) * ld(v[1]),
        ld(v[2]) * ld(v[2]),
    );
    st(a + (b + c))
}

/// Each component times 1 over the length, the length being the product of the two
/// roots kept unrounded (`fld root; fmul exponent_root; fld1; fdivrp`; then `fld v_k;
/// fmul st, st(1); fstp`). Gives the three components as floats and the inverse as a
/// float (the last `fstp`).
pub(crate) fn scaled_by_inverse(v: [u32; 3], root: u32, exponent_root: u32) -> ([u32; 3], u32) {
    let inverse = ONE / (ld(root) * ld(exponent_root));
    (v.map(|k| st(ld(k) * inverse)), st(inverse))
}

/// Returns what the function leaves in eax, if it changes it.
pub fn normalise_vector(race: &Race) -> Result<Option<u32>, BadInput> {
    normalise(race, scene().draw_switches.use_sse.bits() != 0)
}

/// The original's SSE path: each component times the approximate reciprocal root of the
/// squares' sum, in single precision. The fourth component is scaled too, as `mulps` does.
fn sse_normalise(v: [u32; 4]) -> [u32; 4] {
    let v = v.map(f32::from_bits);
    let sum = (v[0] * v[0] + v[1] * v[1]) + v[2] * v[2];
    let root = crate::arch::rsqrtss(sum);
    v.map(|k| (root * k).to_bits())
}

/// Exactly as the original (the x87 at its precision, the SSE instructions): what the
/// Rust callers in modules not yet converted use.
pub fn normalise(race: &Race, sse: bool) -> Result<Option<u32>, BadInput> {
    let v: [u32; 4] = crate::session::read_bits(&race.frame.scratch.work_vector);
    if sse {
        crate::session::write_bits(&race.frame.scratch.work_vector, sse_normalise(v));
        return Ok(None);
    }
    let sum = squares_sum([v[0], v[1], v[2]]);
    let root: u32 = race.course.lists.root_table[((sum >> 15) & 0xFF) as usize].bits();
    let exponent_root: u32 =
        race.course.lists.exponent_root_table[((sum & 0x7F80_0000) >> 23) as usize].bits();
    // The product of two floats is exact in a double, as it is in the x87.
    let length = ld(root) * ld(exponent_root);
    if compare(length, ld(NOTHING)) & 0x4000 != 0 {
        return Err(BadInput("a vector of no length"));
    }
    let (scaled, inverse) = scaled_by_inverse([v[0], v[1], v[2]], root, exponent_root);
    let unsigned_zero = |bits: u32| if bits == 0x8000_0000 { 0 } else { bits };
    crate::session::write_first(&race.frame.scratch.work_vector, &scaled.map(unsigned_zero));
    scene().drawing.scene.scratch.set_bits(inverse);
    Ok(Some(sum))
}

#[cfg(test)]
mod tests {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn the_sse_path_equals_the_instructions() {
        fn by_instructions(w: &mut [u32; 9]) {
            // SAFETY: reads and writes only `w`'s 36 bytes and the three xmm registers named.
            unsafe {
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
            }
        }
        let mut x = 0x9E37_79B9u32;
        for _ in 0..200_000 {
            let mut next = || {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                x
            };
            let scale = |r: u32| (r as i32 as f32 / 2.0f32.powi(31) * 500.0).to_bits();
            let v = [scale(next()), scale(next()), scale(next()), next()];
            let mut w = [v[0], v[1], v[2], v[3], 0, 0, 0, 0, 0];
            by_instructions(&mut w);
            assert_eq!(super::sse_normalise(v), [w[0], w[1], w[2], w[3]], "{v:x?}");
        }
    }

    #[test]
    fn nothing_is_the_games() {
        let d = crate::moved::image_bytes(0x5C5D90, 4);
        assert_eq!(u32::from_le_bytes(d.try_into().unwrap()), super::NOTHING);
    }
}
