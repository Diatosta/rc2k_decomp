//! `marks::seed_noise` (0x4C1697): `init_marks`' step 4, the three noise tables from
//! `g_marksSeed`.
//!
//! In:  nothing
//! Out: eax, ebx, ecx the last entry's three values, esi 0
//!
//! See `marks/setup.rs`. One caller, `marks::init_marks`. Self-test: the original and the
//! Rust version on the game's variables with seeds of the test's, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::marks::setup::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::seed_noise", rva(0x4C1697), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let mut r: Vec<Region> = NOISE_RVAS
        .iter()
        .map(|&t| (game_addr(t) as u32, 4 * NOISE_LEN as usize))
        .collect();
    r.extend([at(SEED_RVA), at(NOISE_COUNT_RVA)]);
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { seed_noise(r) }))
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
    let saved = capture(&regions);
    for seed in [0u32, 1, 0x47, 0x1234_5678, 0x8000_0000, u32::MAX] {
        check_state(
            &mut t,
            &format!("seed {seed:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(a, n) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0xAA, n);
                }
                game_write(SEED_RVA, seed);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
