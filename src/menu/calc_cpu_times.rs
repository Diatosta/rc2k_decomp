//! `menu::calc_cpu_times`: the computer drivers' times for the current record
//! ("CALCCPUTIMES:STARTED" / "FINISHED" logged around it). The record's four times go into
//! `state.menus.class_times` and are scaled (`menu::scale_stage_times`); its track entry's length
//! goes into `track_length[0]` and that x 100 / 0x96A into `track_length[1]`. Then for each class
//! and each driver of it: half the class's time / its driver count is the gap, a random number
//! under 3000 (plus, when `service.spread_first` is not 1, one under 1000 times
//! `service.spread_step`) is added, and a driver whose +0x24 is 1 gets the class's time plus the
//! gaps so far into its +0xC,
//! added to its +0x10.
//!
//! The random numbers are `math::random_below`'s. Fatal error 0x80701010 for a hundredfold of 0
//! or a half time of 0 (before anything); refuses a record past the table, a class count of 0
//! or past 4, a class's driver count of 0, and a drivers table without its end.
use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::drivers_store::{Driver, IN_EVENT};
use crate::menu::owner::{Menus, Screens};
use crate::menu::standings::*;
use crate::race::owner::Race;
use crate::records::stages_store::RecordRef;
use crate::system::owner::System;
use rc2k_core::saves::defaults::CLASS_DRIVER_COUNTS;

use crate::menu::scale_stage_times::{held_section, scaled};

const TEXTS_RVA: [&std::ffi::CStr; 2] = [c"CALCCPUTIMES:STARTED", c"CALCCPUTIMES:FINISHED"];

/// The class time `c`, the class's entry of `class_times`. `event.shown.classes` is only ever the 4
/// of `EventState::new` (no other writer, no saved copy), so a class is one of the four.
fn class_time_at(screens: &Screens, c: u32) -> Result<u32, BadInput> {
    match screens.state.menus.class_times.get(c as usize) {
        Some(time) => Ok(time.get()),
        None => Err(BadInput("a class past the four (class_times)")),
    }
}

/// The drivers of class `c`, the same way.
fn class_count_at(c: u32) -> Result<u32, BadInput> {
    match CLASS_DRIVER_COUNTS.get(c as usize) {
        Some(&count) => Ok(count),
        None => Err(BadInput("a class past the four (class counts)")),
    }
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
    let (section, index): (u32, u32) = (
        screens.event.section.get(),
        screens.event.stage_in_section.get(),
    );
    crate::records::find_record::find_record_in(
        &system.records().stage_tables,
        table,
        index as i32,
        section,
    )
}

/// What is refused for the record `rec` (or its fatal error), before
/// anything: the class count, the drivers and the pack entry's length.
pub fn check<'a>(
    menus: &'a Menus,
    system: &System,
    race: &Race,
    rec: RecordRef,
) -> Result<(u32, &'a [Driver], u32), Refused> {
    let screens = &menus.screens;
    let classes: u32 = screens.event.shown.classes.get();
    if classes == 0 || classes > 4 {
        return Err(BadInput("a class count of 0 or past the four").into());
    }
    let drivers = drivers(menus)?;
    let times = rec.record(&system.records().stage_tables)?.class_times();
    let (times, _) = scaled(race, times, held_section(screens));
    let length = crate::records::trakpak_entry::track_of(&system.records().stage_tables, rec)?
        .length(&system.records().stage_tables)?;
    if length.wrapping_mul(100) == 0 {
        return Err(DIVIDE_BY_ZERO.into());
    }
    let class_time = |c: u32| -> Result<u32, BadInput> {
        match times.get(c as usize) {
            Some(&time) => Ok(time),
            None => Err(BadInput("a class past the four (class_times)")),
        }
    };
    for c in 0..classes {
        if !any_in_class(drivers, c) {
            continue;
        }
        if class_time(c)? >> 1 == 0 {
            return Err(DIVIDE_BY_ZERO.into());
        }
        if class_count_at(c)? == 0 {
            return Err(BadInput("a class's driver count of 0: the original divides by it").into());
        }
    }
    Ok((classes, drivers, length))
}

pub fn calc_cpu_times(menus: &Menus, system: &System, race: &Race) -> Result<(), Refused> {
    let screens = &menus.screens;
    let rec = record(screens, system)?;
    let (classes, drivers, length) = check(menus, system, race, rec)?;
    crate::logfile::log_write::log_nul(&TEXTS_RVA[0], true, 1);
    for (k, t) in rec
        .record(&system.records().stage_tables)?
        .class_times()
        .into_iter()
        .enumerate()
    {
        screens.state.menus.class_times[k].set(t);
    }
    crate::menu::scale_stage_times::scale_stage_times(
        screens,
        &system.records().stage_tables,
        rec,
        race,
    )?;
    screens.state.menus.track_length[0].set(length);
    screens.state.menus.track_length[1].set(length.wrapping_mul(100) / 0x96A);
    use rc2k_core::menu::event::{Spread, cpu_gap, cpu_time};
    for c in 0..classes {
        let mut gaps = 0u32;
        for d in drivers {
            if d.class.get() != c {
                continue;
            }
            let time = class_time_at(screens, c)?;
            let count = class_count_at(c)?;
            let gap = cpu_gap(time, count);
            let mut spread = Spread {
                first: screens.event.service.spread_first.get(),
                step: screens.event.service.spread_step.get(),
            };
            let own = spread.next(crate::math::random_below::random_below)?;
            screens.event.service.spread_first.set(spread.first);
            screens.event.service.spread_step.set(spread.step);
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
