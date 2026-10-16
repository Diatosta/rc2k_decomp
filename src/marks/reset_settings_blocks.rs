//! `marks::reset_settings_blocks` (0x4C9A6D): in `init_marks`' step 3, the eight blocks
//! of `g_marksSettings` reset.
//!
//! In:  nothing
//! Out: ebp the settings, esi the first block, eax, ebx, ecx its three copied floats
//!
//! See `marks/blocks.rs`. One caller, `sub_4C3B01`. Self-test: the original and the Rust
//! version on settings of the test's, the game's pointer put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::blocks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "marks::reset_settings_blocks",
    rva(0x4C9A6D),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = (0x60 + SETTINGS_BLOCKS * SETTINGS_BLOCK) as usize;

fn regions() -> Vec<Region> {
    let settings: u32 = unsafe { crate::framework::game_read(SETTINGS_RVA) };
    vec![(settings + 0x60, LEN - 0x60)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { reset_settings_blocks(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = [at(SETTINGS_RVA)];
    let saved = capture(&inputs);
    let settings = vec![0xAAu8; LEN];
    let base = settings.as_ptr() as u32;
    unsafe { game_write(SETTINGS_RVA, base) };
    let regions = [(base, LEN)];
    check_state(
        &mut t,
        "settings of the test's",
        call_original,
        Regs::SENTINEL,
        &|| unsafe { std::ptr::write_bytes(base as *mut u8, 0xAA, LEN) },
        &regions,
        &rust,
    );
    restore(&inputs, &saved);
    drop(settings);
    t.finish()
}
