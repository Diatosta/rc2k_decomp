//! `physics::off_line_warnings` (0x44148F): unless finished (+0xA8) or +0xA4 set, two
//! counters with their next cue times: +0x270 (`unk_71A248`) counts the steps the car is
//! far from the checkpoints (`g_nearestCheckpointDistance` not below `flt_614C6C`, or
//! +0x98 four or more from +0x94), else 0 with its time +0x278 300; +0x274 counts the
//! steps its heading is `dword_614C74` or more (unsigned) from the checkpoint's
//! (`dword_9ED808[+0x98]`), else 0 with +0x27C 300. Then, unless `dword_5BEC94` is set,
//! each counter at or past its time (by the sign) moves the time on 3000 and cues a sound
//! (`physics::cue_listed_car_sound`, ebx 0x50 turned, 0x52 off the line).
//!
//! In:  nothing
//! Out: eax, ebx and what the cue leaves; no flags read
//!
//! Caller: `sub_44157B`.
//! Refuses (before anything) a checkpoint (+0x98) past 0x1418.
//! Shadow mode runs the original alone (`physics/sound_cues.rs`).
//! Self-test: a copy whose cue goes to a stand-in, run on the game's variables (saved and
//! put back), against the Rust version: distances, checkpoints, headings, counters about
//! their times.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::physics::sound_cues::{LISTED, Part, cue_entry};
use rc2k_core::physics::progress::{
    LAST_CHECKPOINT, OFF_LINE_CUE, TURNED_CUE, Warnings, cue_due, off_line, spread,
};
use rc2k_core::physics::x87::below as x87_below;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The counters (IDA addresses: off, turned, their times).
const COUNTERS: [usize; 4] = [0x71A248, 0x71A24C, 0x71A250, 0x71A254];

fn load() -> Warnings {
    let [off, turned, off_time, turned_time] = COUNTERS.map(g);
    Warnings {
        off,
        turned,
        off_time,
        turned_time,
    }
}

fn store(w: &Warnings) {
    for (x, v) in COUNTERS
        .iter()
        .zip([w.off, w.turned, w.off_time, w.turned_time])
    {
        p(*x, v);
    }
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::progress::{Warnings, off_line, cue_due}`.
pub fn off_line_warnings_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if g(0x71A080) != 0 || g(0x71A07C) != 0 {
        return Ok(());
    }
    let cp = g(0x71A070);
    if cp > LAST_CHECKPOINT {
        return Err(BadInput("a checkpoint past 0x1418").into());
    }
    let near = x87_below(
        f32::from_bits(g(0x614C68)) as f64,
        f32::from_bits(g(0x614C6C)) as f64,
    );
    let mut w = load();
    w.count_off(off_line(near, || {
        r.eax = spread(cp, g(0x71A06C));
        r.eax
    }));
    r.ecx = cp;
    r.ebx = g(0x9ED808 + 4 * cp as usize);
    r.eax = spread(g(0x71A998), r.ebx);
    w.count_turned(r.eax, g(0x614C74));
    store(&w);
    if g(0x5BEC94) != 0 {
        return Ok(());
    }
    for (count, time, cue) in [
        (0x71A24C, 0x71A254, TURNED_CUE),
        (0x71A248, 0x71A250, OFF_LINE_CUE),
    ] {
        r.eax = g(count);
        if let Some(next) = cue_due(r.eax, g(time)) {
            p(time, next);
            r.ebx = cue;
            part(LISTED, r)?;
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    cue_entry(r, off_line_warnings_in)
}
