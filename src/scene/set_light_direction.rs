//! `scene::set_light_direction` (0x427326): the direction of the light in the view.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The turning matrices about x and y for the light's two angles (`dword_635098` and
//! `dword_635094`, each times 256 and negated); the first times the second to
//! `flt_5EA10C`; that times the view's turning `dword_5EA24C` to `flt_5EA14C`; and the
//! point (0, 0, 1) through that (`scene::rotate_point`, inlined) is `g_lightDirection`.
//! Three callers.
//! Self-test: 40 settings by lot on the game's matrices, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::matrix::{self, composed_inputs, composed_regions, set_composed};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::set_light_direction",
    rva(0x427326),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { matrix::set_light_direction() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &composed_regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let (regions, inputs) = (composed_regions(), composed_inputs());
    let saved = (capture(&regions), capture(&inputs));
    let mut lot = 0x00C0_FFEEu32;
    let mut next = |below: u32| {
        lot = lot.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (lot >> 8) % below
    };
    for case in 0..40u32 {
        let angles = [next(0x100), next(0x100), next(0x2_0000), next(0x2_0000)];
        let place = [
            next(0x40_0000).wrapping_sub(0x20_0000),
            next(0x40_0000).wrapping_sub(0x20_0000),
            next(0x400_0000),
        ];
        let size = [16u32, 1, 0, 0x100, 0xFFFF_FFF0, 37][next(6) as usize];
        let seed = next(0x00FF_FFFF);
        check_state(
            &mut t,
            &format!("case {case}: angles {angles:x?}, place {place:x?}, size {size:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { set_composed(&mut { seed }, angles, place, size) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
