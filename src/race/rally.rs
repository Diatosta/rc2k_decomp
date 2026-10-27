//! `single_rally`'s race (`sub_4843EC`, IDA 2026-10-17) and its rally's times
//! (`race::copy_rally_times`).

use crate::errors::{BadInput, Refused};
use crate::memory::blocks::At;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_flow::StageTable;
use crate::menu::event_store::EVENT_STATE;
use crate::pointers::GamePtr;

fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a rally times record")
}

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
/// `dword_7558C8`'s table `off_735512`, the rally `dword_7558D0`, the stage
/// `dword_7558CC` x 0xA4, its +0x3C the stage in `g_stageBestTimesAlt` when
/// `dword_7558C8` is 1, else in `dword_7556E8`), and where it puts them (`dword_7556F4`).
/// None for a null pointer or a table not taken.
pub fn rally_times_places() -> Result<Option<(&'static StageTable, usize, At)>, BadInput> {
    {
        let kind = EVENT_STATE.stage_group.get();
        let rallies = crate::records::stages_store::STAGE_TABLES.group_list(kind);
        if rallies == 0 {
            return Ok(None);
        }
        let stages = get(rallies.wrapping_add(4 * EVENT_STATE.stage_sub.get()))?;
        if stages == 0 {
            return Ok(None);
        }
        let stage = stages.wrapping_add(EVENT_STATE.stage_number.get().wrapping_mul(0xA4));
        let n = get(stage + 0x3C)? as usize;
        let table = if kind == 1 {
            &crate::saves::best_times::STAGE_TIMES_ALT
        } else {
            &crate::saves::best_times::TIME_TRIAL_TIMES
        };
        let (true, Some(to)) = (table.is_taken(), CHAMP_VIEW.summary.get()) else {
            return Ok(None);
        };
        Ok(Some((table, n, to)))
    }
}

/// `race::copy_rally_times` (0x48C997): the stage's ten best times into the race's
/// records (0x50 each, `dword_7556F4`): +0xC, +4, +8, +0 from the key, nationality, car
/// and name id, and the sixteen checkpoints at +0x10; each logged (the count, "GB_Timed",
/// the sixteen "CTs", "GB_Shaun", the name, the sixteen again, rules). Every register
/// kept. Refuses, before anything, a null pointer on the way (the original reads through
/// it) and a stage past its table. The logs read only eax (their text or number).
pub fn copy_rally_times() -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, write_line::log_text};
    let Some((table, stage, mut edi)) = rally_times_places()? else {
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
    let at = |p: At, off: u32| p.add(off).read::<u32>("a rally times record");
    for (esi, ecx) in ten.iter().zip((1..=10u32).rev()) {
        let from = |off: u32| u32::from_le_bytes(esi[off as usize..][..4].try_into().unwrap());
        // Converted: `rc2k_core::race::results::race_time`.
        let time = rc2k_core::race::results::race_time(from);
        for (k, &v) in time.iter().enumerate() {
            edi.add(4 * k as u32).write(v, "the race's times")?;
        }
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
        int(at(edi, 0xC)?, false);
        // The record's text: its bytes from its start, read as the original's log reads.
        text(&edi.cstring("the race's times")?, true);
        text(word(9), false);
        for k in 0..16u32 {
            int(at(edi, 0x10 + 4 * k)?, k == 7 || k == 15);
        }
        for n in 10..14 {
            text(word(n), true);
        }
        edi = edi.add(0x50);
    }
    Ok(())
}
