//! `single_rally`'s race (`race::single_rally_race`) and its rally's times
//! (`race::copy_rally_times`).

use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::StageTable;

/// " ", "-----", "Count:", "GB_Timed: t:- ", "CTs:-", "-----", "%%%%%", "-----",
/// "GB_Shaun: ", "CTs:", "-----", "%%%%%", "%%%%%", "%%%%%".
const WORDS: [&std::ffi::CStr; 14] = [
    c" ",
    c"--------------------------------------------",
    c"Count:",
    c"GB_Timed: t:- ",
    c"CTs:-",
    c"--------------------------------------------",
    c"%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%",
    c"--------------------------------------------",
    c"GB_Shaun: ",
    c"CTs:",
    c"--------------------------------------------",
    c"%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%",
    c"%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%",
    c"%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%",
];

/// Where `copy_rally_times` takes its ten records from: the rally's times (by
/// `Event.stage_group`'s table `off_735512`, the rally `Event.stage_sub`, the stage
/// `Event.stage_number` x 0xA4, its +0x3C the stage in `g_stageBestTimesAlt` when
/// `Event.stage_group` is 1, else in `ChampView.time_trial_best_times`). None for a null pointer, or a table or
/// the summary they go to (`saves::best_times::SUMMARY`) not taken.
pub fn rally_times_places(
    system: &crate::system::owner::System,
    menus: &crate::menu::owner::Menus,
) -> Result<Option<(&'static StageTable, usize)>, BadInput> {
    let screens = &menus.screens;
    {
        let kind = screens.event.stage_group.get();
        let stages = system
            .records
            .stage_tables
            .group_event_table(kind, screens.event.stage_sub.get())?;
        let stage = crate::records::stages_store::row(stages, screens.event.stage_number.get())?;
        let n = stage.record()?.event_slot() as usize;
        let table = if kind == 1 {
            &crate::saves::best_times::STAGE_TIMES_ALT
        } else {
            &crate::saves::best_times::TIME_TRIAL_TIMES
        };
        let (true, true) = (
            table.is_taken(),
            crate::saves::best_times::SUMMARY.is_taken(),
        ) else {
            return Ok(None);
        };
        Ok(Some((table, n)))
    }
}

/// `race::copy_rally_times` (0x48C997): the stage's ten best times into the summary's first ten
/// rows (`saves::best_times::SUMMARY`): the time, nationality, car and name id from the key,
/// nationality, car and name id, and the sixteen checkpoints; each logged (the count, "GB_Timed",
/// the sixteen "CTs", "GB_Shaun", the name, the sixteen again, rules). Refuses, before anything, a
/// null pointer on the way (the original reads through it) and a stage past its table. The logs
/// read only their text or number.
pub fn copy_rally_times(
    system: &crate::system::owner::System,
    menus: &crate::menu::owner::Menus,
) -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, write_line::log_text};
    let Some((table, stage)) = rally_times_places(system, menus)? else {
        return Err(BadInput("a null pointer to the rally's times").into());
    };
    // The ten as their bytes, as `race_time` reads a record.
    let ten = table
        .with(|t| t.stages.get(stage).copied())?
        .ok_or(BadInput("a stage past its best times table"))?
        .map(|e| {
            let mut b = [0u8; rc2k_core::saves::entries::TIME_LEN];
            e.write(&mut b);
            b
        });
    let text = |t: &[u8], line: bool| log_text(t, line, 1);
    let word = |n: usize| WORDS[n].to_bytes();
    let int = |v: u32, line: bool| log_int(v as i32, line, 1);
    use rc2k_core::saves::rankings::Rival;
    let summary = &crate::saves::best_times::SUMMARY;
    for (row, (esi, ecx)) in ten.iter().zip((1..=10u32).rev()).enumerate() {
        let from = |off: u32| u32::from_le_bytes(esi[off as usize..][..4].try_into().unwrap());
        // Converted: `rc2k_core::race::results::race_time`.
        let time = rc2k_core::race::results::race_time(from);
        let rival = Rival::from_dwords(time);
        // The row's text as the log reads it: from its start to a NUL, on into the rows
        // after (and the block's zeros past the summary).
        let row_text = summary.with(|t| {
            t.rows[row] = rival;
            let bytes = rc2k_core::saves::tables::FileTable::write(t);
            let from = &bytes[row * 0x50..];
            from[..from.iter().position(|&c| c == 0).unwrap_or(from.len())].to_vec()
        })?;
        text(word(0), true);
        text(word(1), true);
        text(word(2), false);
        int(ecx, false);
        text(word(3), false);
        int(from(0x14), false);
        text(word(4), false);
        for k in 0..16u32 {
            int(from(0x5C + 4 * k), k == 7 || k == 15);
        }
        text(word(5), true);
        text(word(6), true);
        text(word(7), true);
        text(word(8), false);
        int(rival.time, false);
        // The record's text: its bytes from its start, read as the original's log reads.
        text(&row_text, true);
        text(word(9), false);
        for k in 0..16u32 {
            int(rival.checkpoints[k as usize], k == 7 || k == 15);
        }
        for n in 10..14 {
            text(word(n), true);
        }
    }
    Ok(())
}
