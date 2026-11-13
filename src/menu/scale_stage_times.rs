//! `menu::scale_stage_times`: the stage's four record times (the four `class_times`) scaled for the
//! section (`section` held to 0 .. 5): the last three by the section's entry of one factor table,
//! the first by the section's entry of another (the two factors kept in the x87 work cells); then
//! logged ("stage times info", the section, the record's +0x40, "a7x" and the first factor, "a8x"
//! and the second, "times a8-a5" and the four times).
//!
//! Caller: `menu::calc_cpu_times`. Logs through `logfile::` (level 1).
use crate::errors::Refused;
use crate::menu::owner::Screens;
use crate::race::owner::Race;
use crate::records::stages_store::RecordRef;
use crate::session::{read_all, write_all};

const TEXTS_RVA: [&std::ffi::CStr; 4] = [c"stage times info", c"a7x", c"a8x", c"times a8-a5"];
/// The section held to 0 .. 5.
pub fn held_section(screens: &Screens) -> u32 {
    rc2k_core::menu::stage_scale::held_section(screens.event.section.get() as i32)
}

/// The four times as scaled (see the header), and the two factors.
pub fn scaled(race: &Race, times: [u32; 4], section: u32) -> ([u32; 4], [f32; 2]) {
    // The two factor tables: six entries each.
    let s = section as usize;
    let f: [f32; 2] = [
        race.shared.tables().stage_time_factors_last[s].get(),
        race.shared.tables().stage_time_factors_first[s].get(),
    ];
    // The logic is in `rc2k_core::menu::stage_scale` (exact at the game's precisions for times
    // below 2^29; its header).
    let out = rc2k_core::menu::stage_scale::scale_times(times, f);
    (out, f)
}

/// `record` is the stage's record (its +0x40 is logged).
pub fn scale_stage_times(
    screens: &Screens,
    stages: &crate::records::stages_store::StageTables,
    record: RecordRef,
    race: &crate::race::owner::Race,
) -> Result<(), Refused> {
    use crate::logfile::{log_float::log_float, log_int::log_int};
    let section = held_section(screens);
    let times: [u32; 4] = read_all(&screens.state.menus.class_times);
    let (times, f) = scaled(race, times, section);
    race.shared.work.get().stage_time_scales[0].set(f[0]);
    race.shared.work.get().stage_time_scales[1].set(f[1]);
    write_all(&screens.state.menus.class_times, times);
    let text = |n: usize| &TEXTS_RVA[n];
    crate::logfile::log_write::log_nul(text(0), false, 1);
    log_int(section as i32, false, 1);
    log_int(record.record(stages)?.number() as i32, false, 1);
    crate::logfile::log_write::log_nul(text(1), false, 1);
    log_float(f[0], 2, false, 1);
    crate::logfile::log_write::log_nul(text(2), false, 1);
    log_float(f[1], 2, false, 1);
    crate::logfile::log_write::log_nul(text(3), false, 1);
    for (k, &t) in times.iter().enumerate() {
        log_int(t as i32, k == 3, 1);
    }
    Ok(())
}
