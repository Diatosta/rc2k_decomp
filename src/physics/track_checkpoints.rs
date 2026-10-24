//! `physics::track_checkpoints` (0x44157B): the work car's checkpoints followed.
//! `physics::find_car_checkpoint` into +0x98 and `dword_6365E4`;
//! `physics::checkpoint_offset`; `physics::off_line_warnings`; at the last checkpoint
//! of the current split (`g_checkCounts[+0x98]` - 1 equal to +0x90) +0x94 = +0x98; then,
//! below `g_splitColourCount` and when `physics::passed_split_point` gives CF, +0x94 and
//! +0x98 the split's checkpoint (`dword_601030[+0x90]`) and +0x90 up by one.
//!
//! In:  nothing
//! Out: the registers as the calls leave them; no flags read (the caller calls on)
//!
//! Caller: `sub_43B635` (0x43B90F), every physics step. Refuses (before writing) a
//! checkpoint past 0x1418 or a split past 0x40, and what the calls refuse.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use rc2k_core::physics::splits::{at_split_end, check_tracked, split_ahead};

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The steps. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::splits::{check_tracked, at_split_end, split_ahead}`.
pub fn track_checkpoints() -> Result<(), Refused> {
    let checkpoint = unsafe { crate::physics::find_car_checkpoint::find_car_checkpoint() }?;
    check_tracked(checkpoint, g(0x71A068))?;
    p(0x71A070, checkpoint);
    p(0x6365E4, checkpoint);
    unsafe { crate::physics::checkpoint_offset::checkpoint_offset() }?;
    crate::physics::off_line_warnings::off_line_warnings()?;
    let next = g(0x9E87A0 + 4 * g(0x71A070) as usize).wrapping_sub(1);
    if at_split_end(next.wrapping_add(1), g(0x71A068)) {
        p(0x71A06C, g(0x71A070));
    }
    if !split_ahead(g(0x71A068), g(0x601830)) {
        return Ok(());
    }
    if !unsafe { crate::physics::passed_split_point::passed_split_point() }? {
        return Ok(());
    }
    let split = g(0x601030 + 4 * g(0x71A068) as usize);
    p(0x71A06C, split);
    p(0x71A070, split);
    p(0x71A068, g(0x71A068).wrapping_add(1));
    Ok(())
}
