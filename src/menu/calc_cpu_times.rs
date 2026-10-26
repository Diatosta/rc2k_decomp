//! `menu::calc_cpu_times` (0x470950): the computer drivers' times for the current record
//! ("CALCCPUTIMES:STARTED" / "FINISHED" logged around it). The record's four times (+0x18
//! .. +0x24) into `dword_75AAE8` .. and scaled (`menu::scale_stage_times`); its track pack
//! entry's +0xC into `dword_75AAF8` and its hundredfold / 0x96A into `dword_75AAFC`. Then
//! for each class c (0 .. `dword_7558F8`) and each driver of that class (+8), in order:
//! half the class's time (`dword_75AAE8`[c]) / its driver count (`dword_75AB00`[c]) into
//! `hhmm` (the gap); `dword_7559EC` a random number under 3000, plus (when `dword_755944`
//! is not 1) one under 1000 times `dword_755948` (then `dword_755944` 0 and
//! `dword_755948` up by one; when it is 1, `dword_755948` 0); a driver whose +0x24 is 1
//! gets that plus the class's time plus the gaps so far into its +0xC and added to its
//! +0x10; the gap added for each driver of the class.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_471F0D`, `sub_473126`. The steps are their Rust versions (the random
//! numbers `math::random_below`'s). Fatal error 0x80701010 as the original's checks (a
//! hundredfold of 0, a half time of 0: the original tests the dividend), before anything;
//! refuses a record past the table, a class count of 0 or past 0x100, a class's driver
//! count of 0 (the original's `div` traps), a drivers table without its end, before
//! anything. Shadow mode runs the original alone (it logs). Self-test:
//! the original and the Rust version on the game's tables, the random state and the
//! counters by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;

use crate::menu::scale_stage_times::{TIMES_RVA, held_section, scaled};

const TEXTS_RVA: [usize; 2] = [rva(0x4DE232), rva(0x4DE247)]; // aCalccputimesSt, aCalccputimesFi
const LENGTH_RVA: [usize; 2] = [rva(0x75AAF8), rva(0x75AAFC)]; // dword_75AAF8, dword_75AAFC
const COUNTS_RVA: usize = rva(0x75AB00); // dword_75AB00: drivers per class
const CLASSES_RVA: usize = rva(0x7558F8); // dword_7558F8
const GAP_RVA: usize = rva(0x7559F0); // hhmm
const SPREAD_RVA: usize = rva(0x7559EC); // dword_7559EC
const FIRST_RVA: usize = rva(0x755944); // dword_755944
const STEP_RVA: usize = rva(0x755948); // dword_755948
const DIVIDE_BY_ZERO: FatalError = FatalError {
    code: 0x80701010,
    message: "",
};
/// The current record (see `standings::current_event`'s checks).
pub fn record() -> Result<u32, BadInput> {
    current_event()?;
    let table = game_addr(RECORDS_RVA) as u32;
    let (section, index): (u32, u32) = unsafe { (game_read(SECTION_RVA), game_read(INDEX_RVA)) };
    Ok(crate::records::find_record::find_record_in(
        table,
        index as i32,
        section,
    ))
}

/// What the Rust version refuses for the record `rec` (or its fatal error), before
/// anything: the class count, the drivers and the pack entry's length.
pub fn check(rec: u32) -> Result<(u32, Vec<u32>, u32), Refused> {
    let classes: u32 = unsafe { game_read(CLASSES_RVA) };
    if classes == 0 || classes > 0x100 {
        return Err(BadInput("a class count of 0 or past 0x100").into());
    }
    let drivers = entries(game_addr(DRIVERS_RVA) as u32, DRIVER)?;
    let times = [
        get(rec + 0x18),
        get(rec + 0x1C),
        get(rec + 0x20),
        get(rec + 0x24),
    ];
    let (times, _) = scaled(times, held_section());
    let pack = unsafe { crate::records::trakpak_entry::record_entry(rec as *const u8) }?;
    let length = get(pack + 0xC);
    if length.wrapping_mul(100) == 0 {
        return Err(DIVIDE_BY_ZERO.into());
    }
    // The class's time as the loop reads it: `dword_75AAE8`[c], past the four times
    // `dword_75AAF8` / `dword_75AAFC` as written just before.
    let class_time = |c: u32| -> u32 {
        match c {
            0..4 => times[c as usize],
            4 => length,
            5 => length.wrapping_mul(100) / 0x96A,
            _ => unsafe { game_read(TIMES_RVA + 4 * c as usize) },
        }
    };
    for c in 0..classes {
        if !drivers.iter().any(|&d| get(d + 8) == c) {
            continue;
        }
        if class_time(c) >> 1 == 0 {
            return Err(DIVIDE_BY_ZERO.into());
        }
        if unsafe { game_read::<u32>(COUNTS_RVA + 4 * c as usize) } == 0 {
            return Err(BadInput("a class's driver count of 0: the original divides by it").into());
        }
    }
    Ok((classes, drivers, length))
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn calc_cpu_times(entry: &Regs) -> Result<Regs, Refused> {
    use crate::logfile::log_write::log_write;
    let rec = record()?;
    let (classes, drivers, length) = check(rec)?;
    unsafe {
        log_write(game_addr(TEXTS_RVA[0]) as *const u8, true, 1);
        for (k, t) in [0x18, 0x1C, 0x20, 0x24].into_iter().enumerate() {
            game_write(TIMES_RVA + 4 * k, get(rec + t));
        }
        crate::menu::scale_stage_times::scale_stage_times(&Regs { edi: rec, ..*entry })?;
        game_write(LENGTH_RVA[0], length);
        game_write(LENGTH_RVA[1], length.wrapping_mul(100) / 0x96A);
        // Converted: `rc2k_core::menu::event::{Spread, cpu_gap, cpu_time}`.
        use rc2k_core::menu::event::{Spread, cpu_gap, cpu_time};
        for c in 0..classes {
            let mut gaps = 0u32;
            for &d in &drivers {
                if get(d + 8) != c {
                    continue;
                }
                let time: u32 = game_read(TIMES_RVA + 4 * c as usize);
                let count: u32 = game_read(COUNTS_RVA + 4 * c as usize);
                let gap = cpu_gap(time, count);
                game_write(GAP_RVA, gap);
                let mut spread = Spread {
                    first: game_read(FIRST_RVA),
                    step: game_read(STEP_RVA),
                };
                let own = spread.next(|limit| crate::math::random_below::random_below(limit))?;
                game_write(SPREAD_RVA, own);
                game_write(FIRST_RVA, spread.first);
                game_write(STEP_RVA, spread.step);
                if get(d + 0x24) == 1 {
                    let own = cpu_time(own, gaps, time);
                    put(d + 0xC, own);
                    put(d + 0x10, get(d + 0x10).wrapping_add(own));
                }
                gaps = gaps.wrapping_add(gap);
            }
        }
        log_write(game_addr(TEXTS_RVA[1]) as *const u8, true, 1);
    }
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { calc_cpu_times(r) }?))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, &rust)
}
