//! `marks::pick_hexagon` (0x4BC6F1): one step of the marks' random (`g_marksSeed`, times
//! 0x9D plus 0x47) split into a place (bits 0 .. 10, 11 .. 20) and a kind (bits 22 .. 24)
//! for `marks::spawn_hexagon`.
//!
//! In:  nothing
//! Out: eax, ebx, ecx the place and the kind
//!
//! Caller: `marks::hexagons_spawn` (twice). The call is `spawn_hexagon`'s Rust version.
//! Self-test: the original and the Rust version on the game's items, seeds by lot; put back.
//!
//! Converted 2026-10-21: the hook's Rust version is `pick_hexagon_plain`, through
//! `rc2k_core::marks::items::pick_hexagon`; compared exactly as before (the earlier version
//! stays for the Rust callers in unconverted code).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::setup::SEED_RVA;
use crate::marks::spawn_hexagon::{self, spawn_hexagon};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::pick_hexagon", rva(0x4BC6F1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn pick_hexagon(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let s = game_read::<u32>(SEED_RVA)
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        game_write(SEED_RVA, s);
        g.eax = s & 0x7FF;
        g.ebx = s.rotate_right(11) & 0x3FF;
        g.ecx = s.rotate_right(22) & 7;
        spawn_hexagon(g.eax, g.ebx, g.ecx);
    }
    g
}

/// The converted version: the hook's.
// Converted: `rc2k_core::marks::items::pick_hexagon`
pub unsafe fn pick_hexagon_plain(entry: &Regs) -> Regs {
    unsafe {
        let p = rc2k_core::marks::items::pick_hexagon(game_read(SEED_RVA));
        game_write(SEED_RVA, p.random);
        spawn_hexagon::spawn_hexagon_plain(p.x, p.z, p.kind);
        Regs {
            eax: p.x,
            ebx: p.z,
            ecx: p.kind,
            ..*entry
        }
    }
}

fn regions() -> Vec<crate::service::harness::Region> {
    let mut r = spawn_hexagon::regions();
    r.push(at(SEED_RVA));
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { pick_hexagon_plain(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    for case in 0..5u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &saved);
                game_write(SEED_RVA, case.wrapping_mul(0x9E37_79B9));
                game_write(spawn_hexagon::CURSOR_RVA, case * 0x30 * 7);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
