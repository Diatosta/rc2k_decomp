//! `race::seed_car_randoms` (0x444BF6): each car block's +0x10 and +0xC one
//! `race::nonzero_random`.
//!
//! In:  nothing
//! Out: eax = the last number, ecx = 8, esi = the last car block; the rest preserved
//!
//! See `race/race_init.rs`. One caller, `sub_4462B9` (race init). Shadow mode: the Rust
//! version, the blocks and the generator's state put back, the original's, compared.
//! Self-test: the game's eight blocks filled with values of the test's, generator states
//! of the test's (0 among them), both versions, put back afterwards.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, ESI, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::math::next_random::STATE_RVA;
use crate::race::race_init::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::seed_car_randoms", rva(0x444BF6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let mut r = block_regions(0xC, 8);
    r.push(at(STATE_RVA));
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { seed_car_randoms(r) };
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(ECX, g.ecx)
        .with(ESI, g.esi))
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
    for state in [0u32, 1, 0x40_0006, 0x2026_1017, 0xDEAD_BEEF] {
        check_state(
            &mut t,
            &format!("state={state:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0x5A, len);
                }
                game_write(STATE_RVA, state);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
