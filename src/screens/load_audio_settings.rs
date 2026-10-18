//! `screens::load_audio_settings` (0x4707E6): the audio options into the sound page's work variables (see `screens/page_settings.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a page's unread list (0x7593A2). Shadow mode: by state. Self-test: the options as they are and test values;
//! all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::screens::page_settings::*;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::load_audio_settings",
    rva(0x4707E6),
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { load_audio_settings() };
    Ok(Left::nothing())
}

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![g(0x755670, 0x14), at(rva(0x755680))]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };

    let regions = regions();
    let mut kept = regions.clone();
    kept.push((game_addr(rva(0x506CA4)) as u32, 0x18));
    let saved = capture(&kept);
    for seed in [0u32, 1, 2] {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                if seed != 0 {
                    game_write(rva(0x506CA4), [seed, seed + 1, seed * 2, 0, 1, seed ^ 3]);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
