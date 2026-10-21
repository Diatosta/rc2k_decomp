//! `marks::blend_points` (0x4BA5E2): three points of whole numbers (eax, ebx, ecx) blended
//! by the weights of entry `dl` of the spline file (`g_marksSplineFile`, 12 bytes each:
//! c's weight, then b's, then a's): each of x, y, z a ka + (b kb + c kc), left unrounded in
//! st(2) .. st(0) (x deepest).
//!
//! In:  eax, ebx, ecx the points, dl the entry
//! Out: st(0) z, st(1) y, st(2) x; edx the entry times 12; the rest kept
//!
//! Thunk by `out3_hook!`. Caller: `sub_4BA646`. Self-test: points of the test's and
//! entries by lot (weights the game's file's, or put by the test when the file is empty),
//! compared on the three values and the registers.
//!
//! Converted 2026-10-21: the hook's Rust version is `blend_points_plain`, through
//! `rc2k_core::marks::particles::blend` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, rva};
use crate::marks::splines::FILE_RVA;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::vector3::{Triple80, out3_hook};
use crate::screens::harness::Left;
use crate::service::floats::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static OUT: Triple80 = Triple80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::blend_points", rva(0x4BA5E2), thunk, &ORIGINAL).self_test(self_test);

out3_hook!(
    thunk,
    call_original,
    original_with_out3,
    dispatch = dispatch,
    original = ORIGINAL,
    triple = OUT
);

/// The Rust version (see the header): the three values (x, y, z) and the registers.
pub unsafe fn blend_points(entry: &Regs) -> ([[u8; 10]; 3], Regs) {
    let mut g = *entry;
    g.edx = (g.edx & 0xFF) * 12;
    let at = FILE_RVA + g.edx as usize;
    let k: [u32; 3] = unsafe { [game_read(at + 8), game_read(at + 4), game_read(at)] };
    let p = |r: u32, c: u32| unsafe { ((r + 4 * c) as *const i32).read_unaligned() };
    let v = std::array::from_fn(|c| {
        let c = c as u32;
        x87::blend3_80(p(g.eax, c), p(g.ebx, c), p(g.ecx, c), k)
    });
    (v, g)
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::particles::blend`
pub unsafe fn blend_points_plain(entry: &Regs) -> ([[u8; 10]; 3], Regs) {
    let mut g = *entry;
    g.edx = (g.edx & 0xFF) * 12;
    let at = FILE_RVA + g.edx as usize;
    let k: [u32; 3] = unsafe { game_read(at) };
    let p = |r: u32| -> [i32; 3] {
        std::array::from_fn(|c| unsafe { ((r + 4 * c as u32) as *const i32).read_unaligned() })
    };
    let v =
        rc2k_core::marks::particles::blend([p(g.eax), p(g.ebx), p(g.ecx)], k.map(f32::from_bits));
    (v.map(rc2k_core::math::extended::from_f64), g)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let ([x, y, z], g) = unsafe { blend_points_plain(r) };
    OUT.set_outputs([z, y, x]);
    Ok(left_of(&g))
}

extern "C" fn dispatch(r: &mut Regs) {
    let floats = Floats {
        memory: &[(OUT.output_region(), EXTENDED)],
        regs: &[],
    };
    dispatch_state_floats(
        &HOOK,
        r,
        original_with_out3,
        &[OUT.output_region()],
        &floats,
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let points: Vec<i32> = (0..9i32).map(|n| n * 0x1_2345 - 0x4_0000).collect();
    let p = points.as_ptr() as u32;
    let tolerant = Floats {
        memory: &[(OUT.output_region(), EXTENDED)],
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for (entry, edx_high) in [
            (0u32, 0u32),
            (1, 0x1234_5600),
            (0x7F, 0),
            (0xFF, 0xFFFF_FF00),
            (0x40, 0),
        ] {
            check_state_floats(
                &mut t,
                "marks::blend_points",
                &format!("control word {word:#x}, entry {entry:#x}"),
                original_with_out3,
                Regs {
                    eax: p,
                    ebx: p + 12,
                    ecx: p + 24,
                    edx: edx_high | entry,
                    ..Regs::SENTINEL
                },
                &|| OUT.set_outputs([[0xAA; 10]; 3]),
                &[OUT.output_region()],
                held,
                &rust,
            );
        }
    });
    drop(points);
    t.finish()
}
