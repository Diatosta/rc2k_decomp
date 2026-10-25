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
//! Callers: `sub_471F0D`, `sub_473126`. The steps are their Rust versions (the random numbers
//! `math::random_below`'s). Fatal error 0x80701010 as the original's checks (a hundredfold of 0, a
//! half time of 0: the original tests the dividend), before anything; refuses a record past the
//! table, a class count of 0 or past 0x100, a class's driver count of 0 (the original's `div`
//! traps), a drivers table without its end, before anything.
use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::standings::*;
use crate::menu::state_store::MENU_STATE;

use crate::menu::scale_stage_times::{held_section, scaled};

const TEXTS_RVA: [&std::ffi::CStr; 2] = [c"CALCCPUTIMES:STARTED", c"CALCCPUTIMES:FINISHED"]; // aCalccputimesSt, aCalccputimesFi

/// The class time `c` as the original reads it, `dword_75AAE8`[c]: with no bound, so that a
/// class past the four reads on into the variables that follow.
unsafe fn class_time_at(c: u32) -> u32 {
    unsafe {
        ((MENU_STATE.menus.class_times[0].addr() + 4 * c as usize) as *const u32).read_unaligned()
    }
}

/// The drivers of class `c`, `dword_75AB00`[c], read the same way.
unsafe fn class_count_at(c: u32) -> u32 {
    unsafe {
        ((MENU_STATE.menus.class_counts[0].addr() + 4 * c as usize) as *const u32).read_unaligned()
    }
}
const DIVIDE_BY_ZERO: FatalError = FatalError {
    code: 0x80701010,
    message: "",
};
/// The current record (see `standings::current_event`'s checks).
pub fn record() -> Result<u32, BadInput> {
    current_event()?;
    let table = crate::records::stages_store::STAGE_TABLES.championship();
    let (section, index): (u32, u32) = { (SECTION_RVA.get(), INDEX_RVA.get()) };
    Ok(crate::records::find_record::find_record_in(
        table,
        index as i32,
        section,
    ))
}

/// What the Rust version refuses for the record `rec` (or its fatal error), before
/// anything: the class count, the drivers and the pack entry's length.
pub fn check(rec: u32) -> Result<(u32, Vec<u32>, u32), Refused> {
    let classes: u32 = { EVENT_STATE.classes.get() };
    if classes == 0 || classes > 0x100 {
        return Err(BadInput("a class count of 0 or past 0x100").into());
    }
    let drivers = entries(crate::menu::drivers_store::DRIVERS.first(), DRIVER)?;
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
            _ => unsafe { class_time_at(c) },
        }
    };
    for c in 0..classes {
        if !drivers.iter().any(|&d| get(d + 8) == c) {
            continue;
        }
        if class_time(c) >> 1 == 0 {
            return Err(DIVIDE_BY_ZERO.into());
        }
        if unsafe { class_count_at(c) } == 0 {
            return Err(BadInput("a class's driver count of 0: the original divides by it").into());
        }
    }
    Ok((classes, drivers, length))
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn calc_cpu_times() -> Result<(), Refused> {
    use crate::logfile::log_write::log_write;
    let rec = record()?;
    let (classes, drivers, length) = check(rec)?;
    unsafe {
        log_write(TEXTS_RVA[0].as_ptr() as usize as *const u8, true, 1);
        for (k, t) in [0x18, 0x1C, 0x20, 0x24].into_iter().enumerate() {
            MENU_STATE.menus.class_times[k].set(get(rec + t));
        }
        crate::menu::scale_stage_times::scale_stage_times(rec)?;
        MENU_STATE.menus.track_length[0].set(length);
        MENU_STATE.menus.track_length[1].set(length.wrapping_mul(100) / 0x96A);
        // Converted: `rc2k_core::menu::event::{Spread, cpu_gap, cpu_time}`.
        use rc2k_core::menu::event::{Spread, cpu_gap, cpu_time};
        for c in 0..classes {
            let mut gaps = 0u32;
            for &d in &drivers {
                if get(d + 8) != c {
                    continue;
                }
                let time = class_time_at(c);
                let count = class_count_at(c);
                let gap = cpu_gap(time, count);
                DRAW_WORK.hhmm.set(gap);
                let mut spread = Spread {
                    first: EVENT_STATE.service_944.get(),
                    step: EVENT_STATE.dword_755948.get(),
                };
                let own = spread.next(|limit| crate::math::random_below::random_below(limit))?;
                DRAW_WORK.figure.set(own);
                EVENT_STATE.service_944.set(spread.first);
                EVENT_STATE.dword_755948.set(spread.step);
                if get(d + 0x24) == 1 {
                    let own = cpu_time(own, gaps, time);
                    put(d + 0xC, own);
                    put(d + 0x10, get(d + 0x10).wrapping_add(own));
                }
                gaps = gaps.wrapping_add(gap);
            }
        }
        log_write(TEXTS_RVA[1].as_ptr() as usize as *const u8, true, 1);
    }
    Ok(())
}
