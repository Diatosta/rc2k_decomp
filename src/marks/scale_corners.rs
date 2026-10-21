//! `marks::scale_corners` (0x4C6420): the 0x100 corner records' word pairs scaled into
//! `g_marksCornerSlopes`, with each record's slope.
//!
//! In:  nothing
//! Out: eax, ebx the last pair, esi 0, ebp and edi the two sides
//!
//! See `marks/blocks.rs`. Callers: `marks::allocate_links` and `sub_4C5409`. Self-test:
//! the original and the Rust version on the game's tables with corners of the test's (no
//! side 0), put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `scale_corners_plain`, through
//! `rc2k_core::marks::scales::scale_corner` (plain floats; the x87 version stays for
//! `marks::allocate_links`' part). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{EAX, EBX, RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::marks::blocks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, Tolerance, at_test_precisions, check_state_floats, dispatch_state_floats,
};
use crate::service::harness::{capture, restore};
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
    Ok(left_of(&unsafe { scale_corners_plain(r) }))
}

/// The registers holding floats: the last pair.
const FLOAT_REGS: [(u8, Tolerance); 2] = [(EAX, FLOAT), (EBX, FLOAT)];

extern "C" fn dispatch(r: &mut Regs) {
    let places = corner_floats();
    let floats = Floats {
        memory: &places,
        regs: &FLOAT_REGS,
    };
    dispatch_state_floats(&HOOK, r, call_original, &corner_regions(), &floats, &rust);
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
    let places = corner_floats();
    let tolerant = Floats {
        memory: &places,
        regs: &FLOAT_REGS,
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for seed in [1u32, 2, 3] {
            check_state_floats(
                &mut t,
                "marks::scale_corners",
                &format!("control word {word:#x}, corners by lot {seed}"),
                call_original,
                Regs::SENTINEL,
                &|| {
                    for &(a, n) in &regions {
                        unsafe { std::ptr::write_bytes(a as *mut u8, 0xAA, n) };
                    }
                    fill_corners(seed);
                },
                &regions,
                held,
                &rust,
            );
        }
    });
    restore(&corners, &saved_corners);
    restore(&regions, &saved);
    t.finish()
}
