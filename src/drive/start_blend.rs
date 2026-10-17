//! `drive::start_blend` (0x43773D): the checkpoints the work car is past the lead's
//! start (`dword_71A070` - `dword_70F484`, held to 0 .. 100 by the signs), through
//! `dword_62400C` times `flt_604EC0` into `flt_71A654`; `flt_71A658` = `flt_71A650` +
//! that + `flt_71A64C`, held to `flt_5C5D90` .. `flt_5C5D38` (0 .. 1). By the look of
//! it a share used at a start (a reading).
//!
//! In:  nothing
//! Out: eax the held count; the rest kept; no flags read (`fld` next)
//!
//! Caller: `sub_43A35A` (0x43A486).
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables), 24 cases.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, game_addr, game_read, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{add3, compare_full};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::start_blend", rva(0x43773D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The car's, the lead's, the scratch, the share, the base, the offset, the result.
const VARIABLES: [usize; 7] = [
    0x71A070, 0x70F484, 0x62400C, 0x71A654, 0x71A650, 0x71A64C, 0x71A658,
];
const STEP_RVA: usize = rva(0x604EC0); // flt_604EC0
const LOW_RVA: usize = rva(0x5C5D90); // flt_5C5D90, 0
const HIGH_RVA: usize = rva(0x5C5D38); // flt_5C5D38, 1

unsafe fn get(v: &[u32], k: usize) -> u32 {
    unsafe { (v[k] as *const u32).read_unaligned() }
}

unsafe fn put(v: &[u32], k: usize, x: u32) {
    unsafe { (v[k] as *mut u32).write_unaligned(x) }
}

/// The Rust core on the variables at `v`: the held count.
pub unsafe fn blend_at(v: &[u32]) -> u32 {
    unsafe {
        let mut n = get(v, 0).wrapping_sub(get(v, 1));
        if (n as i32) < 0 {
            n = 0;
        }
        if (n.wrapping_sub(100) as i32) >= 0 {
            n = 100;
        }
        put(v, 2, n);
        put(v, 3, x87::int_times(n as i32, game_read(STEP_RVA)));
        put(v, 6, add3(get(v, 4), get(v, 3), get(v, 5)));
        let (low, high) = (game_read(LOW_RVA), game_read(HIGH_RVA));
        if compare_full(get(v, 6), low) & 0x100 != 0 {
            put(v, 6, x87::loaded(low));
        }
        if compare_full(get(v, 6), high) & 0x100 == 0 {
            put(v, 6, x87::loaded(high));
        }
        n
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn start_blend() -> u32 {
    unsafe { blend_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { start_blend() }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(v[2], 4), (v[3], 4), (v[6], 4)],
        &rust,
    );
}

fn fill(seed: u32, out: &mut [u32]) {
    lot(seed, out);
    out[1] = 1000;
    out[0] = [990u32, 1000, 1050, 1100, 1200, 0x8000_0000][(seed % 6) as usize];
    out[4] = (seed as f32 * 0.21 - 1.0).to_bits();
    out[5] = ((seed / 6) as f32 * 0.3 - 0.4).to_bits();
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x43773D,
        0x93,
        &[0xA1, 0x70, 0xA0, 0x71, 0x00],
        &VARIABLES,
        13,
        24,
        &fill,
        &|v, _| Left::nothing().with(EAX, unsafe { blend_at(v) }),
    )
}
