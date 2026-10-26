//! `single_rally`'s race (`sub_4843EC`, IDA 2026-10-17) and its rally's times
//! (`race::copy_rally_times`).

use crate::errors::{BadInput, Refused};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::pointers::GamePtr;
use crate::race::trap::Trap;

fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a rally times record")
}

fn put(a: u32, v: u32) -> Result<(), BadInput> {
    GamePtr::new(a).write(v, "a rally times record")
}

/// The log calls' places.
pub const WRITE: u32 = 1;
pub const WRITE_NL: u32 = 2;
pub const INT: u32 = 3;
pub const INT_NL: u32 = 4;

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
/// `dword_7558CC` x 0xA4, its +0x3C x 0x618, from `dword_7556E4`, or `dword_7556E8`
/// unless `dword_7558C8` is 1), and puts them (`dword_7556F4`). None for a null pointer.
pub fn rally_times_places() -> Result<Option<(u32, u32)>, BadInput> {
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
        let offset = get(stage + 0x3C)?.wrapping_mul(0x618);
        let base = (if kind == 1 {
            &CHAMP_VIEW.stage_best_times_alt
        } else {
            &CHAMP_VIEW.time_trial_best_times
        })
        .get();
        let to = CHAMP_VIEW.summary.get();
        if base == 0 || to == 0 {
            return Ok(None);
        }
        Ok(Some((offset.wrapping_add(base), to)))
    }
}

/// `race::copy_rally_times` (0x48C997): ten records of the rally's times (0x9C each) into
/// the race's (0x50 each, `dword_7556F4`): +0xC, +4, +8, +0 from +0x14, +8, +0x10, +0xC
/// and 0x40 bytes from +0x5C at +0x10; each logged (the count, "GB_Timed", the sixteen
/// "CTs", "GB_Shaun", the name, the sixteen again, rules). Every register kept. Refuses,
/// before anything, a null pointer on the way (the original reads through it). The logs
/// read only eax (their text or number).
pub fn copy_rally_times() -> Result<(), Refused> {
    {
        let Some((mut esi, mut edi)) = rally_times_places()? else {
            return Err(BadInput("a null pointer to the rally's times").into());
        };
        let mut ecx = 10u32;
        loop {
            // Converted: `rc2k_core::race::results::race_time`.
            let trap = Trap::new();
            let time = rc2k_core::race::results::race_time(|off| {
                trap.read::<u32>(esi + off, "a rally times record")
            });
            trap.finish()?;
            for (k, &v) in time.iter().enumerate() {
                put(edi + 4 * k as u32, v)?;
            }
            let log = |place: u32, value: u32| -> Result<(), Refused> { log_part(place, value) };
            let word = |n: usize| crate::pointers::game_address(WORDS[n].as_ptr());
            log(WRITE_NL, word(0))?;
            log(WRITE_NL, word(1))?;
            log(WRITE, word(2))?;
            log(INT, ecx)?;
            log(WRITE, word(3))?;
            log(INT, get(esi + 0x14)?)?;
            log(WRITE, word(4))?;
            for k in 0..16u32 {
                log(
                    if k == 7 || k == 15 { INT_NL } else { INT },
                    get(esi + 0x5C + 4 * k)?,
                )?;
            }
            log(WRITE_NL, word(5))?;
            log(WRITE_NL, word(6))?;
            log(WRITE_NL, word(7))?;
            log(WRITE, word(8))?;
            log(INT, get(edi + 0xC)?)?;
            log(WRITE_NL, edi)?;
            log(WRITE, word(9))?;
            for k in 0..16u32 {
                log(
                    if k == 7 || k == 15 { INT_NL } else { INT },
                    get(edi + 0x10 + 4 * k)?,
                )?;
            }
            for n in 10..14 {
                log(WRITE_NL, word(n))?;
            }
            esi += 0x9C;
            edi += 0x50;
            ecx -= 1;
            if ecx == 0 {
                break;
            }
        }
    }
    Ok(())
}

/// The log calls in the game.
fn log_part(place: u32, value: u32) -> Result<(), Refused> {
    use crate::logfile::log_int::log_int;
    match place {
        WRITE | WRITE_NL => crate::logfile::log_write::log_at(value, place == WRITE_NL, 1),
        _ => log_int(value as i32, place == INT_NL, 1),
    }
    Ok(())
}
