//! `marks::scale_axes` (0x4BFD73): the two axes at `flt_89CF1A` and `flt_89CF26` scaled: by `flt_89CECA` into `flt_89CF3E` / `flt_89CF4A`, by `flt_89CEEE` / `flt_89CEF2` and the per-part factors `flt_8BC1DE` .. into `flt_89CF6E` / `flt_89CF7A`, and the first two times `flt_8BC1F6` .. into `flt_89CF56` / `flt_89CF62`.
//!
//! In:  nothing
//! Out: the registers as its `lea`s leave them; the rest kept
//!
//! Callers: 0x4bf75e, 0x4c90b3. The original's instructions over the game's variables (register
//! forms as their bytes). Self-test: the original and the Rust version on test floats in
//! every variable it reads, all it writes compared; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `scale_axes_plain`, through
//! `rc2k_core::marks::scales::scale_axes` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::scale_axes", rva(0x4BFD73), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The variables it reads (IDA addresses).
const READS: [usize; 27] = [
    0x89CECA, 0x89CEEE, 0x89CEF2, 0x89CF1A, 0x89CF1E, 0x89CF22, 0x89CF26, 0x89CF2A, 0x89CF2E,
    0x89CF3E, 0x89CF42, 0x89CF46, 0x89CF4A, 0x89CF4E, 0x89CF52, 0x8BC1DE, 0x8BC1E2, 0x8BC1E6,
    0x8BC1EA, 0x8BC1EE, 0x8BC1F2, 0x8BC1F6, 0x8BC1FA, 0x8BC1FE, 0x8BC202, 0x8BC206, 0x8BC20A,
];
/// The variables it writes (IDA addresses).
const WRITES: [usize; 18] = [
    0x89CF3E, 0x89CF42, 0x89CF46, 0x89CF4A, 0x89CF4E, 0x89CF52, 0x89CF56, 0x89CF5A, 0x89CF5E,
    0x89CF62, 0x89CF66, 0x89CF6A, 0x89CF6E, 0x89CF72, 0x89CF76, 0x89CF7A, 0x89CF7E, 0x89CF82,
];

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn scale_axes(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x49CF1A]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF3E]",
            "fld dword ptr [{b} + 0x49CF1E]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF42]",
            "fld dword ptr [{b} + 0x49CF22]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF46]",
            "fld dword ptr [{b} + 0x49CF26]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF4A]",
            "fld dword ptr [{b} + 0x49CF2A]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF4E]",
            "fld dword ptr [{b} + 0x49CF2E]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF52]",
            "fld dword ptr [{b} + 0x49CF1A]",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fmul dword ptr [{b} + 0x4BC1DE]",
            "fstp dword ptr [{b} + 0x49CF6E]",
            "fld dword ptr [{b} + 0x49CF1E]",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fmul dword ptr [{b} + 0x4BC1E2]",
            "fstp dword ptr [{b} + 0x49CF72]",
            "fld dword ptr [{b} + 0x49CF22]",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fmul dword ptr [{b} + 0x4BC1E6]",
            "fstp dword ptr [{b} + 0x49CF76]",
            "fld dword ptr [{b} + 0x49CF26]",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fmul dword ptr [{b} + 0x4BC1EA]",
            "fstp dword ptr [{b} + 0x49CF7A]",
            "fld dword ptr [{b} + 0x49CF2A]",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fmul dword ptr [{b} + 0x4BC1EE]",
            "fstp dword ptr [{b} + 0x49CF7E]",
            "fld dword ptr [{b} + 0x49CF2E]",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fmul dword ptr [{b} + 0x4BC1F2]",
            "fstp dword ptr [{b} + 0x49CF82]",
            "fld dword ptr [{b} + 0x49CF3E]",
            "fmul dword ptr [{b} + 0x4BC1F6]",
            "fstp dword ptr [{b} + 0x49CF56]",
            "fld dword ptr [{b} + 0x49CF42]",
            "fmul dword ptr [{b} + 0x4BC1FA]",
            "fstp dword ptr [{b} + 0x49CF5A]",
            "fld dword ptr [{b} + 0x49CF46]",
            "fmul dword ptr [{b} + 0x4BC1FE]",
            "fstp dword ptr [{b} + 0x49CF5E]",
            "fld dword ptr [{b} + 0x49CF4A]",
            "fmul dword ptr [{b} + 0x4BC202]",
            "fstp dword ptr [{b} + 0x49CF62]",
            "fld dword ptr [{b} + 0x49CF4E]",
            "fmul dword ptr [{b} + 0x4BC206]",
            "fstp dword ptr [{b} + 0x49CF66]",
            "fld dword ptr [{b} + 0x49CF52]",
            "fmul dword ptr [{b} + 0x4BC20A]",
            "fstp dword ptr [{b} + 0x49CF6A]",
            b = in(reg) game_addr(0) as u32,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        g.edi = game_addr(rva(0x89CF62)) as u32;
        g.esi = game_addr(rva(0x89CF4A)) as u32;
    }
    g
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::scales::scale_axes`
pub unsafe fn scale_axes_plain(entry: &Regs) -> Regs {
    use rc2k_core::marks::scales::{Axes, scale_axes};
    let f = |a: usize| f32::from_bits(unsafe { game_read(rva(a)) });
    let three = |a: usize| [f(a), f(a + 4), f(a + 8)];
    let made = scale_axes(&Axes {
        axes: [three(0x89CF1A), three(0x89CF26)],
        factor: f(0x89CECA),
        own: [f(0x89CEEE), f(0x89CEF2)],
        parts: [three(0x8BC1DE), three(0x8BC1EA)],
        second_parts: [three(0x8BC1F6), three(0x8BC202)],
    });
    let put = |a: usize, v: [[f32; 3]; 2]| {
        for (n, x) in v.as_flattened().iter().enumerate() {
            unsafe { game_write(rva(a) + 4 * n, x.to_bits()) };
        }
    };
    put(0x89CF3E, made.scaled);
    put(0x89CF6E, made.weighted);
    put(0x89CF56, made.second);
    Regs {
        edi: game_addr(rva(0x89CF62)) as u32,
        esi: game_addr(rva(0x89CF4A)) as u32,
        ..*entry
    }
}

fn regions() -> Vec<Region> {
    WRITES
        .iter()
        .map(|&a| (game_addr(rva(a)) as u32, 4))
        .collect()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { scale_axes_plain(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places(&regions());
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend(READS.iter().map(|&a| (game_addr(rva(a)) as u32, 4)));
    let saved = capture(&vars);
    let tolerant_places = float_places(&regions);
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for case in 0..6u32 {
            check_state_floats(
                &mut t,
                "marks::scale_axes",
                &format!("control word {word:#x}, case {case}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&vars, &saved);
                    for (n, &a) in READS.iter().enumerate() {
                        let x = ((n as u32).wrapping_mul(0x9E37_79B9)
                            ^ case.wrapping_mul(0x85EB_CA6B)) as i32
                            as f32
                            / 3e8;
                        game_write(rva(a), x.to_bits());
                    }
                },
                &regions,
                held,
                &rust,
            );
        }
    });
    restore(&vars, &saved);
    t.finish()
}
