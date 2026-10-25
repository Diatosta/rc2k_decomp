//! `physics::reset_car` (0x43EFEF): the work car put back on the line. Its checkpoint
//! (`dword_71A06C`, `g_car` +0x94) into `dword_6365E4`, `dword_6365EC` 0. With
//! `dword_62389C` 2 a free checkpoint is looked for: offsets d = 0, 1, .. 0x3F
//! (`dword_614C4C`), each tried as +d then -d (`dword_614C50` = the checkpoint + it), a
//! try failing when `physics::reset_point_blocked` gives CF for any of the blocks at
//! 0x7103CC, 0x7113AC, 0x71238C (tried in that order, the first CF ends it). The first
//! free one goes into `dword_6365E4`; with none, "cant reset car anywhere - waiting a sec"
//! is logged (`log_write_l1_nl`, inside `pusha` / `popa`), +0x1E8 (`dword_71A1C0`, the
//! stuck count) less 100, and it returns. Else `race::place_car_on_line`,
//! `physics::reset_car_on_line`, and +0xFAC (`dword_71AF84`) 10.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: as the last call leaves them (eax the last try's checkpoint and esi the last
//!      block when none was free); no flags read (every caller returns next)
//!
//! Callers: `physics::reset_when_stuck`, `sub_43A0D3`, `sub_43BDD1`. Refuses what the calls
//! refuse.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use rc2k_core::physics::line_places::{RESET_TRIES, reset_offsets};

const LINE_PLACE: usize = rva(0x6365E4); // dword_6365E4
const OFFSET: usize = rva(0x614C4C); // dword_614C4C
const CANDIDATE: usize = rva(0x614C50); // dword_614C50
const STUCK: usize = rva(0x71A1C0); // g_car + 0x1E8
const BLOCKS: [usize; 3] = [0x7103CC, 0x7113AC, 0x71238C];
const NOWHERE_TEXT: &std::ffi::CStr = c"cant reset car anywhere - waiting a second"; // "cant reset car anywhere - waiting a sec"

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

fn p(a: usize, v: u32) {
    unsafe { game_write(a, v) }
}

/// One try: the candidate set; true when a block is in the way (CF).
fn blocked() -> bool {
    p(CANDIDATE, g(LINE_PLACE).wrapping_add(g(OFFSET)));
    BLOCKS.iter().any(|&b| unsafe {
        crate::physics::reset_point_blocked::reset_point_blocked(game_addr(rva(b)) as u32)
    })
}

/// The Rust version. The registers it leaves are read by none of its callers.
// Converted: `rc2k_core::physics::line_places::reset_offsets`.
pub unsafe fn reset_car() -> Result<(), Refused> {
    p(LINE_PLACE, g(rva(0x71A06C)));
    p(rva(0x6365EC), 0);
    if g(rva(0x62389C)) == 2 {
        let mut free = false;
        for offset in reset_offsets() {
            p(OFFSET, offset);
            if !blocked() {
                free = true;
                break;
            }
        }
        if !free {
            p(OFFSET, RESET_TRIES);
            // `log_write_l1_nl`.
            unsafe {
                crate::logfile::log_write::log_write(
                    NOWHERE_TEXT.as_ptr() as usize as *const u8,
                    true,
                    1,
                )
            };
            p(STUCK, g(STUCK).wrapping_sub(100));
            return Ok(());
        }
        p(LINE_PLACE, g(CANDIDATE));
    }
    unsafe {
        crate::race::start_place::place_car_on_line()?;
        crate::physics::reset_car_on_line::reset_car_on_line()?;
    }
    p(rva(0x71AF84), 10);
    Ok(())
}
