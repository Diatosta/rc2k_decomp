//! `math::turn_scratch_xz` (0x41FEF8): the scratch point's x and z (floats at
//! `dword_5C5F50` and `dword_5C5F4C`) turned by an angle.
//!
//! In:  edx = the angle (a dword's turn is 2^32: `g_sines`/`g_cosines` entry (edx +
//!      0x80000) >> 20)
//! Out: edx = the entry; the rest preserved
//!
//! x' = x cos + z sin, z' = z cos - x sin, on the x87 as the original (`math::x87::
//! turn_in_place`). Six callers, `math::turn_point_xz` and its three siblings. Self-test:
//! the original and the Rust version on the scratch point, put back afterwards.

use crate::errors::Refused;
use crate::framework::{EDX, RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::turn_scratch_xz", rva(0x41FEF8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const Z_RVA: usize = rva(0x5C5F4C); // dword_5C5F4C
pub const X_RVA: usize = rva(0x5C5F50); // dword_5C5F50
pub const Y_RVA: usize = rva(0x5C5F54); // dword_5C5F54
const SINES_RVA: usize = rva(0xA23AE4); // g_sines
const COSINES_RVA: usize = rva(0xA24AE4); // g_cosines

/// `math::turn_scratch_xz`: the entry left in edx.
pub unsafe fn turn_scratch_xz(angle: u32) -> u32 {
    let entry = angle.wrapping_add(0x8_0000) >> 20;
    unsafe {
        x87_turn(entry);
    }
    entry
}

unsafe fn x87_turn(entry: u32) {
    let a = |r: usize| game_addr(r + 4 * entry as usize);
    unsafe {
        crate::math::x87::turn_in_place(
            a(SINES_RVA) as *const u32,
            a(COSINES_RVA) as *const u32,
            game_addr(X_RVA) as *mut u32,
            game_addr(Z_RVA) as *mut u32,
        )
    };
}

pub fn scratch() -> Region {
    (game_addr(Z_RVA) as u32, 12)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EDX, unsafe { turn_scratch_xz(r.edx) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[scratch()], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [scratch()];
    let saved = capture(&regions);
    for (x, z, angle) in [
        (1.0f32, 0.0f32, 0u32),
        (123.456, -78.9, 0x4000_0000),
        (-5000.25, 3.5, 0x1234_5678),
        (0.1, 0.7, 0xFFF8_0000),
        (1e9, -1e-9, 0x8007_FFFF),
        (f32::MAX, 2.0, 0x3000_0000),
    ] {
        check_state(
            &mut t,
            &format!("x {x}, z {z}, angle {angle:#x}"),
            call_original,
            Regs {
                edx: angle,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                game_write(Z_RVA, [z.to_bits(), x.to_bits(), 0x5555_5555]);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
