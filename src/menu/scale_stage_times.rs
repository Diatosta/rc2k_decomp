//! `menu::scale_stage_times` (0x46354E): the stage's four record times (`dword_75AAE8` ..
//! `dword_75AAF4`) scaled for the section (`section` held to 0 .. 5): the last three by
//! `flt_60463C`[section], the first by `flt_604654`[section] (`fild`, `fmul`, `fistp`; the
//! two factors kept in `flt_71C3D8` / `flt_71C3DC`); then logged ("stage times info", the
//! section, the record's +0x40, "a7x" and the first factor, "a8x" and the second, "times
//! a8-a5" and the four times).
//!
//! In:  edi the record
//! Out: eax the section held (no `pusha`); the rest kept
//!
//! Caller: `menu::calc_cpu_times`. Logs through the `logfile::` Rust versions (level 1).
use crate::errors::Refused;
use crate::menu::owner::Screens;
use crate::menu::standings::*;
use crate::race::tables_store::RACE_TABLES;
use crate::race::work_store::RACE_WORK;
use crate::records::stages_store::RecordRef;
use crate::session::{read_all, write_all};

const TEXTS_RVA: [&std::ffi::CStr; 4] = [c"stage times info", c"a7x", c"a8x", c"times a8-a5"];
/// The section held to 0 .. 5.
pub fn held_section() -> u32 {
    rc2k_core::menu::stage_scale::held_section(SECTION_RVA.get() as i32)
}

/// The four times as scaled (see the header), and the two factors.
pub fn scaled(times: [u32; 4], section: u32) -> ([u32; 4], [u32; 2]) {
    // flt_60463C, flt_604654: 6 each
    let s = section as usize;
    let f: [u32; 2] = [
        RACE_TABLES.stage_time_factors_last[s].bits(),
        RACE_TABLES.stage_time_factors_first[s].bits(),
    ];
    // Converted: `rc2k_core::menu::stage_scale` (exact at the game's precisions for times
    // below 2^29; its header).
    let out = rc2k_core::menu::stage_scale::scale_times(times, f.map(f32::from_bits));
    (out, f)
}

/// `record` the stage's record (edi in the original; its +0x40 is logged). The original
/// leaves eax the section, which its one caller (`calc_cpu_times`) does not read.
pub fn scale_stage_times(screens: &Screens, record: RecordRef) -> Result<(), Refused> {
    use crate::logfile::{log_float::log_float, log_int::log_int};
    let section = held_section();
    let times: [u32; 4] = read_all(&screens.state.menus.class_times);
    let (times, f) = scaled(times, section);
    RACE_WORK.stage_time_scales[0].set_bits(f[0]);
    RACE_WORK.stage_time_scales[1].set_bits(f[1]);
    write_all(&screens.state.menus.class_times, times);
    let text = |n: usize| &TEXTS_RVA[n];
    crate::logfile::log_write::log_nul(text(0), false, 1);
    log_int(section as i32, false, 1);
    log_int(record.record()?.number() as i32, false, 1);
    crate::logfile::log_write::log_nul(text(1), false, 1);
    log_float(f32::from_bits(f[0]), 2, false, 1);
    crate::logfile::log_write::log_nul(text(2), false, 1);
    log_float(f32::from_bits(f[1]), 2, false, 1);
    crate::logfile::log_write::log_nul(text(3), false, 1);
    for (k, &t) in times.iter().enumerate() {
        log_int(t as i32, k == 3, 1);
    }
    Ok(())
}
