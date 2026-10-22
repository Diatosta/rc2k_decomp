//! `race::mud_pixel` (0x44A58A): a pixel of `g_surface` taken apart, or 0 outside it.
//!
//! In:  ecx = x, edx = y
//! Out: eax = the components; the rest preserved
//!
//! See `race/mud.rs`. One caller, `race::sample_mud_colours`. Self-test: surfaces of the
//! test's (16 and 32 bits), places inside and outside.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::mud::{TestSurface, has_format, mud_pixel};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::mud_pixel", rva(0x44A58A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { mud_pixel(r.ecx, r.edx) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !has_format() {
        return t.skip("no pixel format");
    }
    for bits in [16u32, 32] {
        let surface = TestSurface::new(bits);
        for (x, y) in [
            (0u32, 0u32),
            (5, 7),
            (63, 47),
            (64, 0),
            (0, 48),
            (u32::MAX, 3),
            (31, 20),
        ] {
            check_state(
                &mut t,
                &format!("{bits} bits, ({x}, {y})"),
                call_original,
                Regs {
                    ecx: x,
                    edx: y,
                    ..Regs::SENTINEL
                },
                &|| {},
                &[],
                &rust,
            );
        }
        drop(surface);
    }
    t.finish()
}
