//! `menu::calc_cpu_times` (0x470950): the computer drivers' times for the current record
//! ("CALCCPUTIMES:STARTED" / "FINISHED" logged around it). The record's four times (+0x18
//! .. +0x24) go into `state.menus.class_times` and are scaled (`menu::scale_stage_times`); its
//! track pack entry's +0xC goes into `track_length[0]` and its hundredfold / 0x96A into
//! `track_length[1]`. Then for each class c (0 .. `event.classes`) and each driver of that
//! class (+8), in order: half the class's time / its driver count (`class_counts`[c]) into
//! the gap; the draw work's figure gets a random number under 3000, plus (when
//! `cpu_spread_first` is not 1) one under 1000 times `cpu_spread_step` (then
//! `cpu_spread_first` is cleared and `cpu_spread_step` goes up by one; when it is 1,
//! `cpu_spread_step` is cleared); a driver whose +0x24 is 1 gets that plus the class's time
//! plus the gaps so far into its +0xC, added to its +0x10; the gap added for each driver of
//! the class.
//!
//! The random numbers are `math::random_below`'s. Fatal error 0x80701010 as the original's checks (a hundredfold of 0, a
//! half time of 0: the original tests the dividend), before anything; refuses a record past the
//! table, a class count of 0 or past 0x100, a class's driver count of 0 (the original's `div`
//! traps), a drivers table without its end, before anything.
use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::drivers_store::{Driver, IN_EVENT};
use crate::menu::owner::{Menus, Screens};
use crate::menu::standings::*;
use crate::race::owner::Race;
use crate::records::stages_store::RecordRef;
use crate::system::owner::System;

use crate::menu::scale_stage_times::{held_section, scaled};

const TEXTS_RVA: [&std::ffi::CStr; 2] = [c"CALCCPUTIMES:STARTED", c"CALCCPUTIMES:FINISHED"]; // aCalccputimesSt, aCalccputimesFi

/// The class time `c` as the original reads it, the class's entry of `class_times`: with no bound, so that a
/// class past the four reads on into the variables that follow.
fn class_time_at(screens: &Screens, c: u32) -> Result<u32, BadInput> {
    crate::session::read_on_in(
        &screens.state,
        &screens.state.menus.class_times,
        c as usize,
        "a class time",
    )
}

/// The drivers of class `c`, `class_counts`[c], read the same way.
fn class_count_at(screens: &Screens, c: u32) -> Result<u32, BadInput> {
    crate::session::read_on_in(
        &screens.state,
        &screens.state.menus.class_counts,
        c as usize,
        "a class driver count",
    )
}

const DIVIDE_BY_ZERO: FatalError = FatalError {
    code: 0x80701010,
    message: "",
};
/// Whether any of the drivers is in class `c` (+8), the first found ending the look.
fn any_in_class(drivers: &[Driver], c: u32) -> bool {
    drivers.iter().any(|d| d.class.get() == c)
}

/// The current record (see `standings::current_event`'s checks).
pub fn record(screens: &Screens, system: &System) -> Result<RecordRef, BadInput> {
    current_event(screens, system)?;
    let table = system.records().stage_tables.championship();
    let (section, index): (u32, u32) = (screens.event.section.get(), screens.event.index.get());
    crate::records::find_record::find_record_in(table, index as i32, section)
}

/// What is refused for the record `rec` (or its fatal error), before
/// anything: the class count, the drivers and the pack entry's length.
pub fn check<'a>(
    menus: &'a Menus,
    race: &Race,
    rec: RecordRef,
) -> Result<(u32, &'a [Driver], u32), Refused> {
    let screens = &menus.screens;
    let classes: u32 = screens.event.classes.get();
    if classes == 0 || classes > 0x100 {
        return Err(BadInput("a class count of 0 or past 0x100").into());
    }
    let drivers = drivers(menus)?;
    let times = rec.record()?.class_times();
    let (times, _) = scaled(race, times, held_section(screens));
    let length = crate::records::trakpak_entry::track_of(rec)?.length()?;
    if length.wrapping_mul(100) == 0 {
        return Err(DIVIDE_BY_ZERO.into());
    }
    // The class's time as the loop reads it: `class_times`[c], past the four times
    // `track_length` as written just before.
    let class_time = |c: u32| -> Result<u32, BadInput> {
        match c {
            0..4 => Ok(times[c as usize]),
            4 => Ok(length),
            5 => Ok(length.wrapping_mul(100) / 0x96A),
            _ => class_time_at(screens, c),
        }
    };
    for c in 0..classes {
        if !any_in_class(drivers, c) {
            continue;
        }
        if class_time(c)? >> 1 == 0 {
            return Err(DIVIDE_BY_ZERO.into());
        }
        if class_count_at(screens, c)? == 0 {
            return Err(BadInput("a class's driver count of 0: the original divides by it").into());
        }
    }
    Ok((classes, drivers, length))
}

pub fn calc_cpu_times(menus: &Menus, system: &System, race: &Race) -> Result<(), Refused> {
    let screens = &menus.screens;
    let rec = record(screens, system)?;
    let (classes, drivers, length) = check(menus, race, rec)?;
    crate::logfile::log_write::log_nul(&TEXTS_RVA[0], true, 1);
    for (k, t) in rec.record()?.class_times().into_iter().enumerate() {
        screens.state.menus.class_times[k].set(t);
    }
    crate::menu::scale_stage_times::scale_stage_times(screens, rec, race)?;
    screens.state.menus.track_length[0].set(length);
    screens.state.menus.track_length[1].set(length.wrapping_mul(100) / 0x96A);
    // The logic is in `rc2k_core::menu::event::{Spread, cpu_gap, cpu_time}`.
    use rc2k_core::menu::event::{Spread, cpu_gap, cpu_time};
    for c in 0..classes {
        let mut gaps = 0u32;
        for d in drivers {
            if d.class.get() != c {
                continue;
            }
            let time = class_time_at(screens, c)?;
            let count = class_count_at(screens, c)?;
            let gap = cpu_gap(time, count);
            screens.draw_work.hhmm.set(gap);
            let mut spread = Spread {
                first: screens.event.cpu_spread_first.get(),
                step: screens.event.cpu_spread_step.get(),
            };
            let own = spread.next(crate::math::random_below::random_below)?;
            screens.draw_work.figure.set(own);
            screens.event.cpu_spread_first.set(spread.first);
            screens.event.cpu_spread_step.set(spread.step);
            if d.in_event.get() == IN_EVENT {
                let own = cpu_time(own, gaps, time);
                d.time.set(own);
                d.total.set(d.total.get().wrapping_add(own));
            }
            gaps = gaps.wrapping_add(gap);
        }
    }
    crate::logfile::log_write::log_nul(&TEXTS_RVA[1], true, 1);
    Ok(())
}
