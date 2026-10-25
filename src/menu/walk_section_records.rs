//! `menu::walk_section_records` (0x471F0D): the current section's records walked from
//! `index`: a record tagged 0 gets the computer drivers' times (`menu::calc_cpu_times`)
//! and `dword_755944` 0, one tagged 1 the drivers' condition reset
//! (`menu::reset_driver_condition`) and `dword_755944` 1; past each of those and of any
//! other tag but 3 and 4, `index` and `dword_755950` up by one and the players' totals
//! reset (`menu::reset_player_totals`). Tag 4 ends the walk, tag 3 too with
//! `dword_755928` 1; then the drivers' condition reset.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A0E60`, `sub_4AA4D7`. The steps are their Rust versions. Refuses what they refuse
//! for any record of the walk, and a walk past 64 records, before anything.
use crate::errors::Refused;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::standings::*;
use rc2k_core::menu::section_walk::Step;

/// The record at `index` of the current section.
fn record_at(index: u32) -> u32 {
    let table = crate::records::stages_store::STAGE_TABLES.championship();
    let section: u32 = { SECTION_RVA.get() };
    crate::records::find_record::find_record_in(table, index as i32, section)
}

/// The records the walk visits from `index`, with their steps, checked.
fn walk() -> Result<Vec<(u32, Step)>, Refused> {
    current_event()?;
    counted_players()?;
    entries(crate::menu::drivers_store::DRIVERS.first(), DRIVER)?;
    let index: u32 = { INDEX_RVA.get() };
    // Converted: `rc2k_core::menu::section_walk::walk`.
    rc2k_core::menu::section_walk::walk(
        index,
        |i| get(record_at(i)),
        |i| crate::menu::calc_cpu_times::check(record_at(i)).map(|_| ()),
    )
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn walk_section_records() -> Result<(), Refused> {
    use crate::menu::{reset_driver_condition as rdc, reset_player_totals as rpt};
    let records = walk()?;
    unsafe {
        for (_, step) in records {
            if !step.goes_on() {
                if step == Step::Ended {
                    EVENT_STATE.championship_end.set(1u32);
                }
                break;
            }
            match step {
                Step::CpuTimes => {
                    crate::menu::calc_cpu_times::calc_cpu_times()?;
                }
                Step::Condition => {
                    rdc::reset_driver_condition()?;
                }
                _ => {}
            }
            if let Some(first) = step.first() {
                EVENT_STATE.service_944.set(first);
            }
            INDEX_RVA.set(INDEX_RVA.get().wrapping_add(1));
            EVENT_STATE
                .stages_done
                .set(EVENT_STATE.stages_done.get().wrapping_add(1));
            rpt::reset_player_totals()?;
        }
        rdc::reset_driver_condition()?;
    }
    Ok(())
}
