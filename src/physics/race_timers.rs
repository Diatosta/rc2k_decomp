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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::count_down_start::count_down_start;
use crate::physics::count_start_second::count_start_second;
use crate::physics::count_stop_timers::count_stop_timers;
use crate::physics::mark_car_started::mark_car_started;
use crate::physics::splits::{DONE, MODE, g, guarded, p};
use crate::physics::start_go_notice::start_go_notice;
use crate::physics::take_split::take_split_in;
use crate::physics::take_split_counted::take_split_counted_in;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
