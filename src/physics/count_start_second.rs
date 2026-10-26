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
use crate::physics::cells::{get, put_changed};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use rc2k_core::physics::start::CarStart;
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

/// The variables (IDA addresses).
const VARIABLES: [usize; 5] = [0x71A244, 0x719FF4, 0x719FEC, 0x71A080, 0x71A060];

/// The Rust version on the variables' addresses.
// Converted: `rc2k_core::physics::start::CarStart::count_start_second`.
unsafe fn core(v: &[u32]) -> Left {
    unsafe {
        let was = CarStart {
            state: get(v[0]),
            count: get(v[1]),
            timer: get(v[2]),
            finished: get(v[3]),
            time: get(v[4]),
            ..CarStart::default()
        };
        let mut c = was;
        let eax = c.count_start_second();
        put_changed(v[4], was.time, c.time);
        eax.map_or(Left::nothing(), |e| Left::nothing().with(EAX, e))
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
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
