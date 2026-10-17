//! `race::seal_stage_time` (0x41F29D): the first car's time (+0x88, held to 0x22550FF;
//! its +0xB0 when that is negative) `xor` `dword_4F66F9` and turned right by its low
//! five bits, into the dword after "RCIB" (0x5BECCC); `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_shell.rs`. One caller, `sub_41E85A` (a `mov` follows). Shadow mode:
//! the Rust version, the dword put back, the original's, compared. Self-test: times, signs
//! and keys of the test's in the game's variables, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::race_shell::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::seal_stage_time", rva(0x41F29D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![(game_addr(SEAL_RVA) as u32, 4)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { seal_stage_time() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

#[allow(unused_variables)] // not every test uses every helper
fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let _ = (
        game_addr(0),
        at(0),
        game_write::<u32> as unsafe fn(usize, u32),
    );
    let regions = regions();
    let kept = [
        (game_addr(SEAL_RVA) as u32, 4),
        at(rva(0x70F49C)),
        at(rva(0x70F474)),
        (game_addr(rva(0x4F66F9)) as u32, 4),
    ];
    let saved = capture(&kept);
    for (absent, time, key) in [
        (0u32, 0x1234u32, 0u32),
        (0, 0x2255_0FFF, 0x1F),
        (1, 0x2255_0100, 0xDEAD_BEEF),
        (0x8000_0000, 7, 0x0000_0105),
        (0xFFFF_FFFF, 0xFFFF_FFFF, 0x55),
    ] {
        check_state(
            &mut t,
            &format!("absent {absent:#x}, time {time:#x}, key {key:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(rva(0x70F49C), absent);
                game_write(rva(0x70F474), time);
                (game_addr(rva(0x4F66F9)) as *mut u32).write_unaligned(key);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
