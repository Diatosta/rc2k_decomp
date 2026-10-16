//! `math::turn_point_xz` (0x41FD12): a point of whole numbers turned in x and z.
//!
//! In:  eax = x, ebx = y, ecx = z, edx = the angle
//! Out: eax = x', ecx = z' (as `math::turn_scratch_xz` makes them, rounded as the x87
//!      rounds), edx as `turn_scratch_xz` leaves it; the rest preserved (ebx too: the
//!      original loads y into it, then its `pop ebx` puts the caller's back)
//!
//! The three go through the scratch point (`dword_5C5F50` x, `dword_5C5F54` y,
//! `dword_5C5F4C` z), left there as whole numbers. 29 callers (the map, the cameras, ..).
//! Self-test: the original and the Rust version on the scratch point, put back afterwards.

use crate::errors::Refused;
use crate::framework::{
    EAX, ECX, EDX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva,
};
use crate::math::turn_scratch_xz::{X_RVA, Y_RVA, Z_RVA, scratch, turn_scratch_xz};
use crate::math::x87;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::turn_point_xz", rva(0x41FD12), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `math::turn_point_xz`: x', y (as it comes back through a float, left only in the
/// scratch point), z' and the entry left in edx.
pub unsafe fn turn_point_xz(x: i32, y: i32, z: i32, angle: u32) -> [u32; 4] {
    unsafe {
        game_write(X_RVA, x87::from_int(x));
        game_write(Y_RVA, x87::from_int(y));
        game_write(Z_RVA, x87::from_int(z));
        let entry = turn_scratch_xz(angle);
        let back = |r: usize| {
            let v = x87::to_int(game_read(r)) as u32;
            game_write(r, v);
            v
        };
        let (x, y, z) = (back(X_RVA), back(Y_RVA), back(Z_RVA));
        [x, y, z, entry]
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [x, _, z, entry] =
        unsafe { turn_point_xz(r.eax as i32, r.ebx as i32, r.ecx as i32, r.edx) };
    Ok(Left::nothing().with(EAX, x).with(ECX, z).with(EDX, entry))
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
    for (x, y, z, angle) in [
        (1000i32, 5, 0, 0u32),
        (-123_456, 7, 98_765, 0x4000_0000),
        (0x7FFF_FFFF, 0x0123_4567, -0x8000_0000, 0x1234_5678),
        (3, -2, 1, 0xFFF8_0000),
        (0x10_0000, 0x0100_0001, -0x20_0000, 0xC000_0000),
    ] {
        check_state(
            &mut t,
            &format!("({x}, {y}, {z}) by {angle:#x}"),
            call_original,
            Regs {
                eax: x as u32,
                ebx: y as u32,
                ecx: z as u32,
                edx: angle,
                ..Regs::SENTINEL
            },
            &|| unsafe { game_write(Z_RVA, [0x1111_1111u32; 3]) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
