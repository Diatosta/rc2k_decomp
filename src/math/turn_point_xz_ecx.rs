//! `math::turn_point_xz_ecx` (0x41FD88): `math::turn_point_xz` with other registers: x in ecx, y in
//! eax, z in ebx (whole numbers), through the scratch point (`fild`/`fstp` in,
//! `math::turn_scratch_xz` by edx, `fld`/`fistp` out); eax is pushed around, so it comes
//! back as it was.
//!
//! In:  ecx x, eax y, ebx z, edx the angle
//! Out: ecx x', ebx z', edx as `turn_scratch_xz` leaves it; the rest kept (esi, ebp
//!      pushed too); no flags read
//!
//! Callers: `sub_4389A8`, `sub_438E5A` (twice).
//! Self-test: the original and the Rust version on the scratch point (put back after),
//! points and angles as `turn_point_xz`'s, under three precisions.

use crate::errors::Refused;
use crate::framework::{EBX, ECX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::turn_point_xz::turn_point_xz;
use crate::math::turn_scratch_xz::scratch;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::turn_point_xz_ecx", rva(0x41FD88), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: x', z' and the entry left in edx.
pub unsafe fn turn_point_xz_ecx(x: i32, y: i32, z: i32, angle: u32) -> [u32; 3] {
    let [x, _, z, entry] = unsafe { turn_point_xz(x, y, z, angle) };
    [x, z, entry]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [x, z, entry] =
        unsafe { turn_point_xz_ecx(r.ecx as i32, r.eax as i32, r.ebx as i32, r.edx) };
    Ok(Left::nothing().with(ECX, x).with(EBX, z).with(EDX, entry))
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
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for (x, y, z, angle) in [
                (1000i32, 5, 0, 0u32),
                (-123_456, 7, 98_765, 0x4000_0000),
                (0x7FFF_FFFF, 0x0123_4567, -0x8000_0000, 0x1234_5678),
                (3, -2, 1, 0xFFF8_0000),
                (0x10_0000, 0x0100_0001, -0x20_0000, 0xC000_0000),
            ] {
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, ({x}, {y}, {z}) by {angle:#x}"),
                    call_original,
                    Regs {
                        ecx: x as u32,
                        eax: y as u32,
                        ebx: z as u32,
                        edx: angle,
                        ..Regs::SENTINEL
                    },
                    &|| {},
                    &regions,
                    &rust,
                );
            }
        });
    }
    restore(&regions, &saved);
    t.finish()
}
