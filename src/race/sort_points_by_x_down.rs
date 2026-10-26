//! `race::sort_points_by_x_down` (0x426D5E): a car's points sorted by x, downwards.
//!
//! In:  esi = the first point, edi = the last (12 bytes each)
//! Out: the registers as the sort leaves them (eax, ebx, ecx, edx, ebp, esi; edi kept)
//!
//! See `race/objects.rs` (`sort_points`: the original's steps kept). Called by
//! `race::set_up_car_points` and by itself. Manual patch with a private copy (a `jnb` in
//! its first bytes; its own two calls relocated to the hook). Self-test: point arrays of
//! the test's (2 .. 0x3D points, repeated keys, keys above 0x80000000).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::objects::{BY_X_DOWN, sort_points};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::sort_points_by_x_down",
    rva(0x426D5E),
    thunk,
    &ORIGINAL,
)
.private_copy(0x89)
.relocate_calls(&[0x76, 0x83])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    // Converted: `rc2k_core::race::quick_sort::quick_sort`.
    unsafe { sort_points(BY_X_DOWN, &mut g)? };
    Ok(left_of(&g))
}

fn regions(r: &Regs) -> Vec<Region> {
    if r.esi < r.edi {
        vec![(r.esi, (r.edi - r.esi + 12) as usize)]
    } else {
        Vec::new()
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
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
    let mut points = vec![0u32; 0x40 * 3];
    let at = points.as_mut_ptr() as u32;
    let mut s = 0x1357_2468u32;
    for (n, spread) in [
        (2u32, 0x100u32),
        (3, 4),
        (4, 0x1_0000),
        (7, 3),
        (0x10, 0x9000_0000),
        (0x3D, 0x40),
        (0x20, 1),
        (1, 9),
    ] {
        let start: Vec<u32> = (0..n * 3)
            .map(|_| {
                s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (s >> 3) % spread + if spread > 0x8000_0000 { 0x7000_0000 } else { 0 }
            })
            .collect();
        let before = Regs {
            esi: at,
            edi: at + n * 12 - 12,
            ..Regs::SENTINEL
        };
        let region = [(at, (n * 12) as usize)];
        check_state(
            &mut t,
            &format!("{n} points within {spread:#x}"),
            call_original,
            before,
            &|| unsafe {
                std::ptr::copy_nonoverlapping(start.as_ptr(), at as *mut u32, start.len());
            },
            &region,
            &rust,
        );
    }
    drop(points);
    t.finish()
}
