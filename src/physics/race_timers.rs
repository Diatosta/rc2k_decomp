//! `physics::race_timers` (0x436790): the race's timers for the work car, every physics
//! step. `flt_614170` 1.0, or `flt_601028` once finished (+0xA8). Then by the race mode
//! (`dword_62389C`): 2: `physics::count_start_second`, `physics::count_down_start`,
//! `physics::take_split_counted`, `physics::start_go_notice`,
//! `physics::count_stop_timers`, `physics::mark_car_started`; 0, 1, 3:
//! `count_start_second`, `physics::take_split`, `count_stop_timers`, `mark_car_started`;
//! else nothing.
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read (the caller goes on with
//!      calls)
//!
//! Caller: `sub_43B635` (0x43B64E).
//! Refuses what the steps refuse; what was written is put back.
//! Self-test: `physics/splits.rs` (`test_splits`), every mode.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::count_down_start::count_down_start;
use crate::physics::count_start_second::count_start_second;
use crate::physics::count_stop_timers::count_stop_timers;
use crate::physics::mark_car_started::mark_car_started;
use crate::physics::splits::{DONE, MODE, g, guarded, p, test_splits, written};
use crate::physics::start_go_notice::start_go_notice;
use crate::physics::take_split::take_split_in;
use crate::physics::take_split_counted::take_split_counted_in;
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::race_timers", rva(0x436790), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The steps on `r`.
// Converted: `rc2k_core::physics::splits::{timer_rate, timer_run}`.
pub fn race_timers_in(r: &mut Regs) -> Result<(), Refused> {
    use rc2k_core::physics::splits::{TimerRun, timer_rate, timer_run};
    p(0x614170, 1.0f32.to_bits());
    let finished = g(DONE) != 0;
    let factor = if finished { g(0x601028) } else { 0 };
    p(
        0x614170,
        timer_rate(finished, f32::from_bits(factor)).to_bits(),
    );
    let run = timer_run(g(MODE));
    if run == TimerRun::None {
        return Ok(());
    }
    unsafe {
        count_start_second().apply(r);
        if run == TimerRun::Start {
            count_down_start()?.apply(r);
            take_split_counted_in(r)?;
            start_go_notice()?.apply(r);
        } else {
            take_split_in(r)?;
        }
        count_stop_timers();
        mark_car_started().apply(r);
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn race_timers(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &race_timers_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { race_timers(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &written(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_splits(&mut t, call_original, &rust, &|_| true);
    t.finish()
}
