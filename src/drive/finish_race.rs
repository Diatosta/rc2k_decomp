//! `drive::finish_race` (0x459116): the championship's finish routine: the stage
//! (`dword_64BB1C[dword_75B0FC]` into `g_stageNumber`), the cars' kind (`dword_75B104`
//! into `g_raceCarKinds` and the first block's +0x24) and +0xAA8 (`dword_75B100`)
//! logged; one car in mode 3 set up (some thirty settings and block words); the stage
//! loaded (`race::load_stage`); `dword_63DB84` 0; `drive::finish_run`; everything tidied
//! (`race::tidy_everything`); `dword_5C2F88`, `64BB18`, `606744`, `606748` 0; logged.
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read
//!
//! Callers: `drive::finish_race_0`, `sub_442FEF`. Ungated: it loads a stage and plays.
//! Refuses what its calls refuse. Shadow mode runs the original alone.
//! Self-test: skipped (it loads a stage and plays the routine).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::race_parts::log_line;
use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::physics::flow::{g, p};
use rc2k_core::drive::finish_routine::{self, Set, Var};

const B: usize = 0x70F3EC;

fn log_text(text: usize) {
    unsafe { crate::logfile::log_write::log_write(game_addr(rva(text)) as *const u8, false, 1) };
}

fn log_number(v: u32) {
    crate::logfile::log_int::log_int(v as i32, true, 1);
}

/// A variable's IDA address.
fn var(v: Var) -> usize {
    match v {
        Var::D60101C => 0x60101C,
        Var::D621514 => 0x621514,
        Var::D623854 => 0x623854,
        Var::D6235FC => 0x6235FC,
        Var::RaceMode => 0x62389C,
        Var::D6238A4 => 0x6238A4,
        Var::RaceCars => 0x63DB94,
        Var::D63DB90 => 0x63DB90,
        Var::D6238A8 => 0x6238A8,
        Var::D6238AC => 0x6238AC,
        Var::D6238B0 => 0x6238B0,
        Var::D6238B4 => 0x6238B4,
        Var::D6238B8 => 0x6238B8,
        Var::D621890 => 0x621890,
        Var::D6238CC => 0x6238CC,
        Var::D6238D0 => 0x6238D0,
        Var::D64BB18 => 0x64BB18,
        Var::D5C2F88 => 0x5C2F88,
        Var::D606744 => 0x606744,
        Var::D606748 => 0x606748,
        Var::D63DB84 => 0x63DB84,
    }
}

/// The settings for the routine (0x4591B0 .. 0x4592FA).
fn set_up() {
    for (at, v) in finish_routine::set_up() {
        match at {
            Set::Var(x) => p(var(x), v),
            Set::Lead(o) => p(B + o as usize, v),
        }
    }
}

/// The Rust version.
// Converted: `rc2k_core::drive::finish_routine`.
///
/// The caller's ecx, which the original hands through `race::load_stage` and
/// `drive::race_init` to `race::checkpoint_distances`, is never read there (that function
/// is reached only with two checkpoints or more; conversion phase 3, 2026-10-24). The
/// registers this leaves are read by no caller (liveness: esi after the menus' calls has
/// no read point).
pub unsafe fn finish_race() -> Result<(), Refused> {
    p(0x623850, g(0x64BB1C + 4 * g(0x75B0FC) as usize));
    p(0x623858, g(0x75B104));
    p(B + 0x24, g(0x623858));
    p(B + 0xAA8, g(0x75B100));
    log_line(0x4DDA52);
    log_line(0x4DDA69);
    log_text(0x4DDA80);
    log_number(g(0x623850));
    log_text(0x4DDA90);
    log_number(g(0x623858));
    log_text(0x4DDA9F);
    log_number(g(B + 0xAA8));
    set_up();
    unsafe {
        crate::race::stage_loading::load_stage()?;
        p(var(finish_routine::BEFORE_RUN), 0);
        crate::drive::finish_run::finish_run()?;
        // `race::tidy_everything`'s esi "read" is of `finish_run`'s leftover, which has
        // no read point (liveness at 0x459313): a scratch.
        crate::race::race_end::tidy_everything(&crate::race::loadlevel::LoadVars::game())?;
    }
    for x in finish_routine::AFTER {
        p(var(x), 0);
    }
    log_line(0x4DDAB1);
    log_line(0x4DDAC8);
    Ok(())
}
