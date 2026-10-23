//! `single_rally`'s race (`sub_4843EC`, IDA 2026-10-17) and its rally's times
//! (`race::copy_rally_times`).

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};

fn at(a: usize) -> u32 {
    game_addr(rva(a)) as u32
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) };
}

/// The log calls' places.
pub const WRITE: u32 = 1;
pub const WRITE_NL: u32 = 2;
pub const INT: u32 = 3;
pub const INT_NL: u32 = 4;

/// " ", "-----", "Count:", "GB_Timed: t:- ", "CTs:-", "-----", "%%%%%", "-----",
/// "GB_Shaun: ", "CTs:", "-----", "%%%%%", "%%%%%", "%%%%%".
const WORDS: [usize; 14] = [
    0x4DE93A, 0x4DE93C, 0x4DE969, 0x4DE970, 0x4DE97F, 0x4DE985, 0x4DE9B2, 0x4DE9DF, 0x4DEA0C,
    0x4DEA17, 0x4DEA1C, 0x4DEA49, 0x4DEA76, 0x4DEAA3,
];

/// Where `copy_rally_times` takes its ten records from: the rally's times (by
/// `dword_7558C8`'s table `off_735512`, the rally `dword_7558D0`, the stage
/// `dword_7558CC` x 0xA4, its +0x3C x 0x618, from `dword_7556E4`, or `dword_7556E8`
/// unless `dword_7558C8` is 1), and puts them (`dword_7556F4`). None for a null pointer.
pub unsafe fn rally_times_places() -> Option<(u32, u32)> {
    unsafe {
        let kind = get(at(0x7558C8));
        let rallies = get(at(0x735512).wrapping_add(4 * kind));
        if rallies == 0 {
            return None;
        }
        let stages = get(rallies.wrapping_add(4 * get(at(0x7558D0))));
        if stages == 0 {
            return None;
        }
        let stage = stages.wrapping_add(get(at(0x7558CC)).wrapping_mul(0xA4));
        let offset = get(stage + 0x3C).wrapping_mul(0x618);
        let base = get(at(if kind == 1 { 0x7556E4 } else { 0x7556E8 }));
        let to = get(at(0x7556F4));
        if base == 0 || to == 0 {
            return None;
        }
        Some((offset.wrapping_add(base), to))
    }
}

/// `race::copy_rally_times` (0x48C997): ten records of the rally's times (0x9C each) into
/// the race's (0x50 each, `dword_7556F4`): +0xC, +4, +8, +0 from +0x14, +8, +0x10, +0xC
/// and 0x40 bytes from +0x5C at +0x10; each logged (the count, "GB_Timed", the sixteen
/// "CTs", "GB_Shaun", the name, the sixteen again, rules). Every register kept. Refuses,
/// before anything, a null pointer on the way (the original reads through it).
pub unsafe fn copy_rally_times(entry: &Regs) -> Result<Regs, Refused> {
    let g = entry;
    unsafe {
        let Some((mut esi, mut edi)) = rally_times_places() else {
            return Err(BadInput("a null pointer to the rally's times").into());
        };
        let mut ecx = 10u32;
        loop {
            // Converted: `rc2k_core::race::results::race_time`.
            let time = rc2k_core::race::results::race_time(|off| get(esi + off));
            for (k, &v) in time.iter().enumerate() {
                put(edi + 4 * k as u32, v);
            }
            let eax = time[0];
            // The registers inside each `pusha`: the loop's, eax as last loaded.
            // ebx `dword_7558D0` from the way in (the times' place).
            let regs = Regs {
                eax,
                ebx: get(at(0x7558D0)),
                ecx,
                esi,
                edi,
                ..*g
            };
            let log = |place: u32, value: u32| -> Result<(), Refused> {
                let mut r = Regs { eax: value, ..regs };
                crate::race::rally::log_part(place, &mut r)
            };
            let word = |n: usize| at(WORDS[n]);
            log(WRITE_NL, word(0))?;
            log(WRITE_NL, word(1))?;
            log(WRITE, word(2))?;
            log(INT, ecx)?;
            log(WRITE, word(3))?;
            log(INT, get(esi + 0x14))?;
            log(WRITE, word(4))?;
            for k in 0..16u32 {
                log(
                    if k == 7 || k == 15 { INT_NL } else { INT },
                    get(esi + 0x5C + 4 * k),
                )?;
            }
            log(WRITE_NL, word(5))?;
            log(WRITE_NL, word(6))?;
            log(WRITE_NL, word(7))?;
            log(WRITE, word(8))?;
            log(INT, get(edi + 0xC))?;
            log(WRITE_NL, edi)?;
            log(WRITE, word(9))?;
            for k in 0..16u32 {
                log(
                    if k == 7 || k == 15 { INT_NL } else { INT },
                    get(edi + 0x10 + 4 * k),
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
    Ok(*g)
}

/// The log calls in the game.
pub fn log_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    unsafe {
        match place {
            WRITE | WRITE_NL => log_write(r.eax as *const u8, place == WRITE_NL, 1),
            _ => log_int(r.eax as i32, place == INT_NL, 1),
        }
    }
    Ok(())
}
