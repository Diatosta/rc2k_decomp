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
//! Caller: `sub_43B635` (0x43B90F), every physics step. It begins with a call: a private
//! copy, its calls relocated, a hand-made patch.
//! Refuses (before writing) a checkpoint past 0x1418 or a split past 0x40, and what the
//! calls refuse.
//! Shadow mode runs the original alone (its calls send sounds and read the stage).
//! Self-test: a copy whose four calls go to stand-ins (the split's CF by a pattern),
//! against the Rust version with the same, on the game's variables (saved and put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use rc2k_core::physics::splits::{at_split_end, check_tracked, split_ahead};

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The calls by place: 0 `find_car_checkpoint`, 1 `checkpoint_offset`, 2
/// `off_line_warnings`, 3 `passed_split_point` (its CF in the registers' flags).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;
/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => *r = crate::physics::find_car_checkpoint::find_car_checkpoint(r)?,
            1 => crate::physics::checkpoint_offset::checkpoint_offset()?.apply(r),
            2 => crate::physics::off_line_warnings::off_line_warnings_in(r)?,
            _ => crate::physics::passed_split_point::passed_split_point(r)?.apply(r),
        }
    }
    Ok(())
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::splits::{check_tracked, at_split_end, split_ahead}`.
pub fn track_checkpoints_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    part(0, r)?;
    check_tracked(r.eax, g(0x71A068))?;
    p(0x71A070, r.eax);
    p(0x6365E4, r.eax);
    part(1, r)?;
    part(2, r)?;
    r.ebx = g(0x71A070);
    r.eax = g(0x9E87A0 + 4 * r.ebx as usize).wrapping_sub(1);
    if at_split_end(r.eax.wrapping_add(1), g(0x71A068)) {
        r.eax = g(0x71A070);
        p(0x71A06C, r.eax);
    }
    r.ecx = g(0x71A068);
    if !split_ahead(r.ecx, g(0x601830)) {
        return Ok(());
    }
    part(3, r)?;
    if !r.carry() {
        return Ok(());
    }
    r.ecx = g(0x71A068);
    r.eax = g(0x601030 + 4 * r.ecx as usize);
    p(0x71A06C, r.eax);
    p(0x71A070, r.eax);
    p(0x71A068, g(0x71A068).wrapping_add(1));
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mut g = *r;
    match track_checkpoints_in(&mut g, &mut game_part) {
        Ok(()) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

// ---- for the self-test ----
