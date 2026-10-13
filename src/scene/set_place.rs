//! `scene::set_place` (0x4271AE): three whole numbers as the place of the matrix
//! `flt_5EA04C`.
//!
//! In:  eax, ebx, ecx = x, y, z
//! Out: every register preserved
//!
//! Each as a float (`fild`, `fstp`) to the matrix's fourth row (`flt_5EA07C` ..); the
//! work variable `dword_62400C` is left holding z.
//! 20 callers.
//! Self-test: 24 places on the game's matrix, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::matrix::{self, PLACED_RVA, matrix};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_place", rva(0x4271AE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { matrix::set_place([r.eax, r.ebx, r.ecx]) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: [Region; 2] = [(game_addr(PLACED_RVA + 0x30) as u32, 12), at(CHECKED_RVA)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions: [Region; 2] = [matrix(PLACED_RVA), at(CHECKED_RVA)];
    let saved = capture(&regions);
    let values = [
        0u32,
        1,
        0xFFFF_FFFF,
        0x7FFF_FFFF,
        0x8000_0000,
        0x0100_0001,
        0xFEFF_FFFF,
        0x0012_3456,
    ];
    for n in 0..24 {
        let place = [
            values[n % 8],
            values[(n * 3 + 1) % 8],
            values[(n * 5 + 2) % 8],
        ];
        check_state(
            &mut t,
            &format!("place {place:x?}"),
            call_original,
            Regs {
                eax: place[0],
                ebx: place[1],
                ecx: place[2],
                ..Regs::SENTINEL
            },
            &|| restore(&regions, &vec![0xAA; saved.len()]),
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
