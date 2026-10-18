//! `physics::count_start_second` (0x436AC4): `g_car` +0x88 (`unk_71A060`) up by one when
//! +0x26C (`unk_71A244`) is -1 or 0, the start count +0x1C is 0, the timer +0x14 less
//! 0x320 is not below 0 (by its sign) and +0xA8 (`unk_71A080`) is 0. A reading: time
//! counted once the start is behind (`physics::false_start`'s fields).
//!
//! In:  nothing
//! Out: eax the timer less 0x320 when it got that far; the rest kept; no flags read
//!
//! Caller: `sub_436790` (four calls).
//! Self-test: `physics/copy_hooks_test.rs`, 8 fillings over the branches.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::count_start_second",
    rva(0x436AC4),
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

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The variables (IDA addresses).
const VARIABLES: [usize; 5] = [0x71A244, 0x719FF4, 0x719FEC, 0x71A080, 0x71A060];

/// The core on the variables' addresses.
unsafe fn core(v: &[u32]) -> Left {
    unsafe {
        let w = at(v[0]);
        if (w != u32::MAX && w != 0) || at(v[1]) != 0 {
            return Left::nothing();
        }
        let eax = at(v[2]).wrapping_sub(0x320);
        if (eax as i32) >= 0 && at(v[3]) == 0 {
            put(v[4], at(v[4]).wrapping_add(1));
        }
        Left::nothing().with(EAX, eax)
    }
}

/// The Rust version.
pub unsafe fn count_start_second() -> Left {
    unsafe { core(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { count_start_second() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(&HOOK, r, call_original, &[(v[4], 4)], &rust);
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x436AC4,
        0x37,
        &[0x83, 0x3D, 0x44, 0xA2, 0x71, 0x00, 0xFF],
        &VARIABLES,
        6,
        8,
        &|seed, out| {
            lot(seed, out);
            out[0] = [0, u32::MAX, 5, 0][(seed % 4) as usize];
            if seed & 4 == 0 {
                out[1] = 0;
            }
            out[2] = [0x31F, 0x320, 0x8000_0000, 0x400][(seed / 2 % 4) as usize];
            if seed % 3 != 0 {
                out[3] = 0;
            }
        },
        &|v, _| unsafe { core(v) },
    )
}
