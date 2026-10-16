//! `race::copy_lights_data` (0x438B03): the lights' data into their 0x22 records.
//!
//! In:  nothing
//! Out: esi and edi past them, edx 0x22
//!
//! See `race/stage_prefs.rs`. Self-test: the original and the Rust version on the game's
//! variables with values of the test's, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::stage_prefs::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::copy_lights_data", rva(0x438B03), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![
        (game_addr(LIGHTS_RVA) as u32, LIGHTS_LEN as usize),
        (
            game_addr(LIGHT_RECORDS_RVA) as u32,
            (LIGHT_RECORDS * 0x134) as usize,
        ),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { copy_lights_data(r) }))
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
    let _ = (at as fn(usize) -> Region, game_addr as fn(usize) -> usize);
    for fill in [0x11u8, 0xE7] {
        check_state(
            &mut t,
            &format!("fill {fill:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for (n, b) in std::slice::from_raw_parts_mut(regions[0].0 as *mut u8, regions[0].1)
                    .iter_mut()
                    .enumerate()
                {
                    *b = fill ^ n as u8;
                }
                std::ptr::write_bytes(regions[1].0 as *mut u8, 0xAA, regions[1].1);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
