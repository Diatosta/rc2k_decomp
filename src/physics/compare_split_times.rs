//! `physics::compare_split_times` (0x436E11, race mode 3): the car's time (+0x88) against
//! the ten kept for this split (`unk_604134` + 0x28 x the split +0xAC): the place
//! +0xA0C counts down from 11 for each faster one (11 or more becomes -1); the nearest
//! gap behind (x10, the largest below 0) into +0xA14 with its name `dword_60410C[c]` in
//! +0xA1C; the nearest ahead (x10, the smallest) negated into +0xA10, its name in +0xA18;
//! `dword_6043F8` the time; for car 0 or 1 the time kept in `unk_75B02C` / `unk_75B09C`
//! [split].
//!
//! In:  nothing
//! Out: eax, ebx (split x 0x28), ecx, esi, edi as left; no flags read (the caller
//!      returns)
//!
//! Caller: `physics::split_by_mode`.
//! Refuses (before anything) a split past 27 (the kept tables' size not worked out).
//! Self-test: `physics/splits.rs` (`test_splits`), mode 3.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::cells::get_list;
use crate::physics::splits::{FIGURE, SPLIT, TIME, g, guarded, p, test_splits, written};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use rc2k_core::physics::splits::{LAST_COMPARED_SPLIT, compare_split};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::compare_split_times",
    rva(0x436E11),
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

/// The steps on `r`.
// Converted: `rc2k_core::physics::splits::compare_split`.
pub fn compare_split_times_in(r: &mut Regs) -> Result<(), Refused> {
    if g(SPLIT) > LAST_COMPARED_SPLIT {
        return Err(BadInput("a split past 27").into());
    }
    let time = g(TIME);
    p(0x6043F8, time);
    r.ebx = g(SPLIT).wrapping_mul(0x28);
    r.esi = (crate::framework::game_addr(rva(0x604134)) as u32).wrapping_add(r.ebx);
    let kept: [u32; 10] = unsafe { get_list(r.esi) };
    let c = compare_split(time, &kept, |c| g(0x60410C + 4 * c));
    let f = c.figures;
    for (k, v) in [
        f.place,
        f.gap_ahead,
        f.gap_behind,
        f.name_ahead,
        f.name_behind,
    ]
    .into_iter()
    .enumerate()
    {
        p(FIGURE + 4 * k, v);
    }
    r.eax = c.last;
    r.ecx = 10;
    let a = |x: usize| crate::framework::game_addr(rva(x)) as u32;
    r.edi = a(0x75B02C);
    let car = g(0x71B3F8);
    if car != 0 {
        r.edi = a(0x75B09C);
        if car != 1 {
            return Ok(());
        }
    }
    r.ecx = g(SPLIT);
    r.eax = time;
    unsafe { ((r.edi + 4 * r.ecx) as *mut u32).write_unaligned(r.eax) };
    Ok(())
}

/// The Rust version.
pub unsafe fn compare_split_times(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &compare_split_times_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { compare_split_times(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &written(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_splits(&mut t, call_original, &rust, &|c| c.mode == 3);
    t.finish()
}
