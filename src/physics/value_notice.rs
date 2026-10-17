//! `physics::value_notice` (0x45B357): notice 0xCE by `dword_71A24C` against 300 (the
//! sign of the difference): at 300 or more, with no notice shown (`dword_71A700` 0) and
//! 0xCE not the last (`dword_71A6FC`), `dword_71A6F4` = 0xCE and `dword_71A704` = 0;
//! below 300 with 0xCE the last, `dword_71A700` and `dword_71A6FC` 0. What the figure is
//! is not known.
//!
//! In:  nothing
//! Out: every register kept; flags not read (`sub_43B635` returns next, its callers
//!      go on with `inc`, calls)
//!
//! Caller: `sub_43B635` (0x43B9AE), its last call, every physics step.
//! Self-test: `physics/copy_hooks_test.rs`, 16 cases.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::value_notice", rva(0x45B357), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The figure, shown, last, id, time (IDA addresses).
const VARIABLES: [usize; 5] = [0x71A24C, 0x71A700, 0x71A6FC, 0x71A6F4, 0x71A704];
const NOTICE: u32 = 0xCE;

unsafe fn core(v: &[u32]) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        if (get(0).wrapping_sub(300) as i32) >= 0 {
            if get(1) == 0 && get(2) != NOTICE {
                put(3, NOTICE);
                put(4, 0);
            }
        } else if get(2) == NOTICE {
            put(1, 0);
            put(2, 0);
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn value_notice() {
    unsafe { core(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { value_notice() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let regions: Vec<_> = v[1..].iter().map(|&a| (a, 4)).collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x45B357,
        0x62,
        &[0x81, 0x3D, 0x4C, 0xA2, 0x71, 0x00, 0x2C, 0x01, 0x00, 0x00],
        &VARIABLES,
        9,
        16,
        &|seed, out| {
            lot(seed, out);
            out[0] = [299u32, 300, 301, 0x8000_012C][(seed % 4) as usize];
            out[1] = (seed / 4) % 2;
            out[2] = if (seed / 8) % 2 == 0 { NOTICE } else { 7 };
        },
        &|v, _| {
            unsafe { core(v) };
            Left::nothing()
        },
    )
}
