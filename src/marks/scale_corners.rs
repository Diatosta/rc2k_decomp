//! `marks::scale_corners` (0x4C6420): the 0x100 corner records' word pairs scaled into
//! `g_marksCornerSlopes`, with each record's slope.
//!
//! In:  nothing
//! Out: eax, ebx the last pair, esi 0, ebp and edi the two sides
//!
//! See `marks/blocks.rs`. Callers: `marks::allocate_links` and `sub_4C5409`. Self-test:
//! the original and the Rust version on the game's tables with corners of the test's (no
//! side 0), put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::marks::blocks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::scale_corners", rva(0x4C6420), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { scale_corners(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &corner_regions(), &rust);
}

/// The corner records filled by lot from `seed`: words of every size, the two sides never
/// 0 (+4 != +0, +0xA != +2), some differences past 0x8000.
pub fn fill_corners(seed: u32) {
    let mut s = seed;
    let mut next = || {
        s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (s >> 8) as u16
    };
    let base = game_addr(CORNERS_RVA) as u32;
    for n in 0..CORNERS {
        let w = |o: u32, v: u16| unsafe { ((base + 0x80 * n + o) as *mut u16).write_unaligned(v) };
        for o in (0..0x10).step_by(2) {
            w(o, next());
        }
        let r = |o: u32| unsafe { ((base + 0x80 * n + o) as *const u16).read_unaligned() };
        if r(4) == r(0) {
            w(4, r(0).wrapping_add(1));
        }
        if r(0xA) == r(2) {
            w(0xA, r(2).wrapping_add(3));
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = corner_regions();
    let corners = [(game_addr(CORNERS_RVA) as u32, 0x80 * CORNERS as usize)];
    let (saved, saved_corners) = (capture(&regions), capture(&corners));
    for seed in [1u32, 2, 3] {
        check_state(
            &mut t,
            &format!("corners by lot {seed}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                for &(a, n) in &regions {
                    unsafe { std::ptr::write_bytes(a as *mut u8, 0xAA, n) };
                }
                fill_corners(seed);
            },
            &regions,
            &rust,
        );
    }
    restore(&corners, &saved_corners);
    restore(&regions, &saved);
    t.finish()
}
