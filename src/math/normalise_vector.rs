//! `math::normalise_vector` (0x45C3FE): the work vector made one long.
//!
//! In:  nothing
//! Out: without SSE eax = the squares' sum as a float's bits (the original pops it
//!      from the slot it kept eax in); everything else preserved
//!
//! The vector is the three floats at `xmmword_5C5F60`; its length is left in
//! `flt_6D18C8`. Two ways, by the switch `dword_621A58` (a key's toggle and the
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
//! Self-test: the bodies and the entry copied as one piece, with the switch as a test
//! variable, both ways.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, GameHook, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm,
    rva,
};
use crate::math::x87_run::{DOUBLE, TEST_CONTROL_WORDS, with_control_word};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::floats::{
    Agreement, FLOAT, Floats, Tolerance, close, compare_memory, dispatch_state_floats, note_drift,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::normalise_vector", rva(0x45C3FE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const VECTOR_RVA: usize = rva(0x5C5F60); // xmmword_5C5F60
const SQUARES_RVA: usize = rva(0x5C5F70); // xmmword_5C5F70
pub const LENGTH_RVA: usize = rva(0x6D18C8); // flt_6D18C8
const SSE_RVA: usize = rva(0x621A58); // dword_621A58
const ROOTS_RVA: usize = rva(0x5C70F8); // flt_5C70F8
const EXPONENT_ROOTS_RVA: usize = rva(0x5C74F8); // flt_5C74F8
const NOTHING_RVA: usize = rva(0x5C5D90); // flt_5C5D90
const CHECKED_RVA: usize = rva(0x62400C); // dword_62400C

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
            game_write(SQUARES_RVA, [w[4], w[5], w[6], w[7]]);
            game_write(LENGTH_RVA, w[8]);
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
        let nothing = f32::from_bits(game_read(NOTHING_RVA)) as f64;
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
        game_write(LENGTH_RVA, w[5]);
        let unsigned_zero = |bits: u32| if bits == 0x8000_0000 { 0 } else { bits };
        game_write(VECTOR_RVA, [w[0], w[1], w[2]].map(unsigned_zero));
        game_write(CHECKED_RVA, w[6]);
        Ok(Some(sum))
    }
}

/// The converted version (`rc2k_core::math::geometry`, plain floats): the hook's.
pub unsafe fn normalise_plain(sse: bool) -> Result<Option<u32>, BadInput> {
    use rc2k_core::math::geometry;
    // Converted: `rc2k_core::math::geometry::normalise` and `normalise_four`.
    unsafe {
        let v = game_read::<[u32; 4]>(VECTOR_RVA).map(f32::from_bits);
        if sse {
            let n = geometry::normalise_four(v);
            game_write(SQUARES_RVA, n.squares.map(f32::to_bits));
            game_write(LENGTH_RVA, n.length.to_bits());
            game_write(VECTOR_RVA, n.vector.map(f32::to_bits));
            return Ok(None);
        }
        let roots = game_read::<[u32; 256]>(ROOTS_RVA).map(f32::from_bits);
        let exponent_roots = game_read::<[u32; 256]>(EXPONENT_ROOTS_RVA).map(f32::from_bits);
        let nothing = f32::from_bits(game_read(NOTHING_RVA));
        let n = geometry::normalise([v[0], v[1], v[2]], &roots, &exponent_roots, nothing)
            .ok_or(BadInput("a vector of no length"))?;
        game_write(LENGTH_RVA, n.length.to_bits());
        game_write(VECTOR_RVA, n.vector.map(f32::to_bits));
        game_write(CHECKED_RVA, n.reciprocal.to_bits());
        Ok(Some(n.sum.to_bits()))
    }
}

// ---- hook ----

fn regions() -> [Region; 4] {
    [
        (game_addr(VECTOR_RVA) as u32, 16),
        (game_addr(SQUARES_RVA) as u32, 16),
        at(LENGTH_RVA),
        at(CHECKED_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let sse = unsafe { game_read::<u32>(SSE_RVA) != 0 };
    Ok(match unsafe { normalise_plain(sse) }? {
        Some(eax) => Left::nothing().with(EAX, eax),
        None => Left::nothing(),
    })
}

/// Floats within `FLOAT`; the SSE way's approximations within `APPROXIMATE`.
const APPROXIMATE: Tolerance = Tolerance::Relative(1.0 / 1024.0);

fn floats(sse: bool) -> [(Region, Tolerance); 4] {
    let tolerance = if sse { APPROXIMATE } else { FLOAT };
    regions().map(|region| (region, tolerance))
}

extern "C" fn dispatch(r: &mut Regs) {
    let sse = unsafe { game_read::<u32>(SSE_RVA) != 0 };
    let memory = floats(sse);
    let floats = Floats {
        memory: &memory,
        regs: &[(EAX, FLOAT)],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

// ---- self-test ----

/// The two bodies and the entry after them, one piece from 0x45C2E3.
const PIECE: usize = 0x45C2E3;
const PIECE_LEN: usize = 0x12B;
const ENTRY_AT: usize = 0x45C3FE - PIECE;
/// `cmp dword_621A58, 0`, which the hook's jump has overwritten.
const ENTRY_BYTES: [u8; 7] = [0x83, 0x3D, 0x58, 0x1A, 0x62, 0x00, 0x00];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy =
        unsafe { RedirectedCopy::new(game_addr(rva(PIECE)), PIECE_LEN, &[], &[], &|_| None) };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut switch = 0u32;
    unsafe {
        std::ptr::copy_nonoverlapping(
            ENTRY_BYTES.as_ptr(),
            (copy.addr() + ENTRY_AT) as *mut u8,
            ENTRY_BYTES.len(),
        );
        copy.patch_u32(ENTRY_AT + 2, &raw mut switch as u32);
    }
    let regions = regions();
    let saved = capture(&regions);
    let vectors: [[f32; 4]; 12] = [
        [1.0, 0.0, 0.0, 7.0],
        [3.0, 4.0, 0.0, 1.0],
        [-1.0, -2.0, -3.0, 0.5],
        [0.577, 0.577, 0.577, 0.0],
        [1e-10, 2e-10, -3e-10, 1.0],
        [1e10, -2e10, 3e10, 1.0],
        [0.0, -0.0, 5.0, -2.0],
        [123.456, -0.001, 77.7, 3.0],
        [0.33333334, 0.6666667, 0.6666667, 9.0],
        [16777216.0, 1.0, 1.0, 1.0],
        [-0.0, 0.25, -0.0, 0.0],
        // No length: only the SSE way is given this one.
        [0.0, 0.0, 0.0, 1.0],
    ];
    // The x87 way reads its roots from tables the game fills; without them every length
    // is 0, which is the case the original must not be given.
    let tables: (u32, u32) = unsafe {
        (
            game_read(ROOTS_RVA + 0x80 * 4),
            game_read(EXPONENT_ROOTS_RVA + 0x7F * 4),
        )
    };
    let ways: &[bool] = if tables.0 == 0 || tables.1 == 0 {
        &[true]
    } else {
        &[false, true]
    };
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for &sse in ways {
                // The x87 way at double precision: the same bits.
                let memory = floats(sse);
                let floats = if word == DOUBLE && !sse {
                    Floats::EXACT
                } else {
                    Floats {
                        memory: &memory,
                        regs: &[],
                    }
                };
                for vector in vectors {
                    if !sse && vector[..3].iter().all(|&part| part == 0.0) {
                        continue;
                    }
                    let run = |rust: bool, switch: &mut u32| unsafe {
                        restore(&regions, &vec![0xAA; saved.len()]);
                        game_write(VECTOR_RVA, vector.map(f32::to_bits));
                        *switch = sse as u32;
                        // What is left in eax, or None if another register changed.
                        let done = if rust {
                            normalise_plain(sse)
                                .ok()
                                .map(|eax| eax.unwrap_or(Regs::SENTINEL.eax))
                        } else {
                            let mut ro = Regs::SENTINEL;
                            call_game(&mut ro, copy.addr() + ENTRY_AT);
                            ro.preserved_except(&Regs::SENTINEL, EAX).then_some(ro.eax)
                        };
                        (done, capture(&regions))
                    };
                    let (theirs, mine) = (run(false, &mut switch), run(true, &mut switch));
                    let eax_ok = match (theirs.0, mine.0) {
                        (Some(a), Some(b)) if word == DOUBLE || sse => a == b,
                        (Some(a), Some(b)) => close(a, b, FLOAT),
                        _ => false,
                    };
                    let memory = compare_memory(&regions, &theirs.1, &mine.1, &floats);
                    if let Agreement::Drift(n) = memory {
                        note_drift(HOOK.name(), || {
                            format!("self-test sse {sse}, vector {vector:?}: {n} places")
                        });
                    }
                    t.check(eax_ok && memory != Agreement::Differ, || {
                        format!(
                            "control word {word:#x}, sse {sse}, vector {vector:?}: \
                             original {theirs:x?}; rust {mine:x?}"
                        )
                    });
                }
            }
        });
    }
    restore(&regions, &saved);
    t.finish()
}
