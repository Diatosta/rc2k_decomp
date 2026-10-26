//! `drive::race_init` (0x4462B9, three chunks): the race's start after the stage is
//! loaded ("RACEINIT : ..." logged before each part, the log indented by 3 round it):
//! the best line (`race::build_best_line`); with fewer than two checkpoints
//! (`g_checkpointCount`, by the sign of the `cmp`) the fatal error 0x80600D1E "Race
//! initialisation failed - no best line" (its third chunk, 0x4D917B), unless
//! `dword_621514` is set (then it returns); the cars' places (`race::start_cars`), the
//! objects (`race::reset_face_lights`), the game mode (`race::set_up_cars`), the messages
//! (`race::reset_car_race_state`), the car parts (`race::clear_race_slots`), the timers
//! (`race::reset_car_starts`, `race::seed_car_randoms`); fifty settling steps (in `pusha`
//! / `popa`: `drive::race_cars_step`; `drive::share_all_cars` with `dword_5C2F94` 1;
//! `marks::frame_marks`, `marks::run_frame`); the slots again; the damage messages
//! (`race::grade_all_cars`); "RACEINIT : end". Its second chunk (0x40763C ..) is the
//! fatal-error reporter's, shared, not entered from here.
//!
//! Not the race's step loop (that runs elsewhere, a step per frame): a run of fifty steps at
//! the stage's start.
//!
//! In:  nothing
//! Out: the registers as the parts leave them (ecx 0x32 after the steps); no flags read
//!
//! Caller: `race::load_stage` (0x44412D).
//! Refuses what its parts refuse; raises 0x80600D1E as the original (section 15).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{FatalError, Refused};
use crate::framework::Regs;
use crate::physics::flow::{a, g, p};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use rc2k_core::drive::race_steps;

const INDENT: usize = 0x4E00A0; // g_logIndent

/// A "RACEINIT : ..." line (level 1, a line break; in `pusha` / `popa`).
fn log(text: usize) {
    unsafe { crate::logfile::log_write::log_write(a(text) as *const u8, true, 1) };
}

/// The log indented round a part (`pushf`; +- 3; `popf`).
fn indented(
    r: &mut Regs,
    part: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    p(INDENT, g(INDENT).wrapping_add(race_steps::LOG_INDENT));
    part(r)?;
    p(INDENT, g(INDENT).wrapping_sub(race_steps::LOG_INDENT));
    Ok(())
}

/// The steps; `r` as the original leaves it.
// Converted: `rc2k_core::drive::race_steps::{no_best_line, NO_BEST_LINE, SETTLING_STEPS,
// LOG_INDENT}`.
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    log(0x4DC558);
    p(INDENT, g(INDENT).wrapping_add(race_steps::LOG_INDENT));
    *r = unsafe {
        crate::race::best_line::build_best_line(r, &mut crate::race::best_line::build_part)
    }?;
    if race_steps::no_best_line(g(0x623640)) {
        if g(0x621514) != 0 {
            return Ok(());
        }
        return Err(FatalError {
            code: race_steps::NO_BEST_LINE,
            message: "Race initialisation failed - no best line",
        }
        .into());
    }
    p(INDENT, g(INDENT).wrapping_sub(race_steps::LOG_INDENT));
    log(0x4DC56F);
    indented(r, &mut |r| {
        *r = unsafe {
            crate::race::car_starts::start_cars(r, &mut crate::race::car_starts::game_part)
        }?;
        Ok(())
    })?;
    log(0x4DC58B);
    indented(r, &mut |r| {
        *r = unsafe { crate::race::course_data::reset_face_lights(r) }?;
        Ok(())
    })?;
    log(0x4DC5AA);
    indented(r, &mut |r| {
        *r = unsafe {
            crate::race::race_cars::set_up_cars(r, &mut crate::race::race_cars::cars_part)
        }?;
        Ok(())
    })?;
    log(0x4DC5C8);
    indented(r, &mut |r| {
        let g = unsafe { crate::race::race_init::reset_car_race_state(r) };
        r.ebx = g.ebx;
        r.edi = g.edi;
        Ok(())
    })?;
    log(0x4DC5E6);
    indented(r, &mut |r| {
        let g = unsafe { crate::race::race_init::clear_race_slots(r) };
        r.edi = g.edi;
        r.ecx = g.ecx;
        Ok(())
    })?;
    log(0x4DC5FF);
    indented(r, &mut |r| {
        for f in [
            crate::race::race_init::reset_car_starts,
            crate::race::race_init::seed_car_randoms,
        ] {
            let g = unsafe { f(r) };
            r.eax = g.eax;
            r.ecx = g.ecx;
            r.esi = g.esi;
        }
        Ok(())
    })?;
    log(0x4DC616);
    p(INDENT, g(INDENT).wrapping_add(race_steps::LOG_INDENT));
    r.ecx = 0;
    while r.ecx != race_steps::SETTLING_STEPS {
        let mut c = *r;
        unsafe { crate::drive::race_cars_step::race_cars_step(&c) }?;
        c = *r;
        p(0x5C2F94, 1);
        unsafe { crate::drive::share_all_cars::share_all_cars() }?.apply(&mut c);
        p(0x5C2F94, 0);
        c = unsafe { crate::marks::frame_marks::frame_marks(&c) }?;
        let _ = unsafe {
            crate::marks::run_frame::run_frame(&c, &mut crate::marks::run_frame::game_part)
        }?;
        r.ecx += 1;
    }
    let s = unsafe { crate::race::race_init::clear_race_slots(r) };
    r.edi = s.edi;
    r.ecx = s.ecx;
    p(INDENT, g(INDENT).wrapping_sub(race_steps::LOG_INDENT));
    log(0x4DC62F);
    indented(r, &mut |r| {
        let g = unsafe { crate::race::car_grades::grade_all_cars(r) };
        r.eax = g.eax;
        r.ebx = g.ebx;
        r.esi = g.esi;
        r.set_carry(false);
        Ok(())
    })?;
    log(0x4DC651);
    Ok(())
}

/// The Rust version.
pub unsafe fn race_init(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    steps(&mut r)?;
    Ok(left_of(&r))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { race_init(r) }?;
    left.apply(r);
    Ok(())
}
