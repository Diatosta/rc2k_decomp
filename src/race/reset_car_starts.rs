//! `race::reset_car_starts` (0x444BBC): each car block's +0x14 and +8 0, or 0x320 while
//! `dword_71C3E0` (the replay) is set; +0x18 and +0x1C 0.
//!
//! In:  nothing
//! Out: eax = that value, ecx = 8, esi = the last car block; the rest preserved
//!
//! See `race/race_init.rs`. One caller, `sub_4462B9` (race init). Shadow mode: the Rust
//! version, the blocks put back, the original's, compared. Self-test: the game's eight
//! blocks filled with values of the test's, `dword_71C3E0` 0, 1 and 0x80000000, both
//! versions, put back afterwards.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, ESI, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::race_init::*;
use crate::race::sounds::REPLAY_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::reset_car_starts", rva(0x444BBC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    block_regions(8, 0x18)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { reset_car_starts(r) };
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(ECX, g.ecx)
        .with(ESI, g.esi))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
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
    let regions = regions();
    let mut kept = regions.clone();
    kept.push(at(REPLAY_RVA));
    let saved = capture(&kept);
    for replay in [0u32, 1, 0x8000_0000] {
        check_state(
            &mut t,
            &format!("replay {replay:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0x5A, len);
                }
                game_write(REPLAY_RVA, replay);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
