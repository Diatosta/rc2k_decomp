//! `physics::count_stop_timers` (0x436978): three counters of the work car (`g_car`
//! +0xA0, +0xA4, +0xA8: `unk_71A078` .. `unk_71A080`) in turn. One that is not 0: at 300
//! `unk_71A088` (+0xB0) is set (-1 for the first two, 2 for the third) and it returns;
//! else while +0x1C4 (`unk_71A19C`) is below `flt_5C5D38` (1.0; unordered counts as
//! below) it goes up, and on reaching 300 (unsigned) the same. A reading: the car has
//! stood still too long after a start or a finish.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the callers go on with calls)
//!
//! Caller: `sub_436790` (four calls).
//! Self-test: a copy of the original (its five variables the test's) against the core:
//! counters 0, 1, 299, 300, speeds below and above.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use std::cmp::Ordering;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::count_stop_timers",
    rva(0x436978),
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

/// The variables (IDA addresses): three counters, the speed, the result.
const VARIABLES: [usize; 5] = [0x71A078, 0x71A07C, 0x71A080, 0x71A19C, 0x71A088];

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The core on the variables' addresses.
pub unsafe fn count_stop_timers_at(v: &[u32]) {
    unsafe {
        let one: u32 = game_read(rva(0x5C5D38));
        let slow = !matches!(
            f32::from_bits(at(v[3])).partial_cmp(&f32::from_bits(one)),
            Some(Ordering::Greater | Ordering::Equal)
        );
        for (k, result) in [(0, u32::MAX), (1, u32::MAX), (2, 2)] {
            let c = at(v[k]);
            if c == 0 {
                continue;
            }
            if c == 300 {
                put(v[4], result);
                return;
            }
            if !slow {
                continue;
            }
            put(v[k], c.wrapping_add(1));
            if c.wrapping_add(1) >= 300 {
                put(v[4], result);
                return;
            }
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
pub unsafe fn count_stop_timers() {
    unsafe { count_stop_timers_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { count_stop_timers() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(&HOOK, r, call_original, &[(v[0], 12), (v[4], 4)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 5];
    let base = cells.as_mut_ptr() as u32;
    let v: Vec<u32> = (0..5).map(|k| base + 4 * k).collect();
    let vars: Vec<(usize, u32)> = VARIABLES.iter().copied().zip(v.iter().copied()).collect();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x436978,
            len: 0xD3,
            entry: &[0x83, 0x3D, 0x78, 0xA0, 0x71, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &vars,
        18,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 20)];
    let counts = [0u32, 1, 299, 300, 0xFFFF_FFFF];
    for a in counts {
        for b in counts {
            for c in [0u32, 299, 300] {
                for speed in [0.5f32.to_bits(), 1.0f32.to_bits(), 0x7FC0_0000] {
                    let fill =
                        || unsafe { (base as *mut [u32; 5]).write([a, b, c, speed, 0x5A5A]) };
                    fill();
                    let mut ro = Regs::SENTINEL;
                    unsafe { call_game(&mut ro, copy.addr()) };
                    let theirs = capture(&all);
                    fill();
                    unsafe { count_stop_timers_at(&v) };
                    let mine = capture(&all);
                    let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
                    t.check(regs_ok && theirs == mine, || {
                        format!("counters {a} {b} {c}, speed {speed:#x}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}")
                    });
                }
            }
        }
    }
    drop((copy, cells));
    t.finish()
}
