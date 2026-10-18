//! `physics::split_by_mode` (0x436B99): the split's figure by the race mode
//! (`dword_62389C`): 2 the car's place (`g_carBlockStore`'s first `place`, 0x70FA40)
//! into +0xA0C; 1 `physics::rank_split_time`; 3 `physics::compare_split_times`; else
//! nothing.
//!
//! In:  nothing
//! Out: as the step leaves them (mode 2: eax the place); no flags read
//!
//! Callers: `physics::take_split`, `physics::take_split_counted`.
//! Refuses what the steps refuse.
//! Self-test: `physics/splits.rs` (`test_splits`).
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::compare_split_times::compare_split_times_in;
use crate::physics::rank_split_time::rank_split_time_in;
use crate::physics::splits::{FIGURE, MODE, g, guarded, p, test_splits, written};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::split_by_mode", rva(0x436B99), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The steps on `r`.
pub fn split_by_mode_in(r: &mut Regs) -> Result<(), Refused> {
    match g(MODE) {
        2 => {
            r.eax = g(0x70FA40);
            p(FIGURE, r.eax);
            Ok(())
        }
        1 => rank_split_time_in(r),
        3 => compare_split_times_in(r),
        _ => Ok(()),
    }
}

/// The Rust version.
pub unsafe fn split_by_mode(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &split_by_mode_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { split_by_mode(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &written(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_splits(&mut t, call_original, &rust, &|_| true);
    t.finish()
}
