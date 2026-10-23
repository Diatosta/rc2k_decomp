//! `race::step_viewed_replay` (0x41EC47): the viewed replay one frame on while it plays:
//! the frame (`dword_5C2534`) one more; past 99 the next block from frame 0, which past
//! the end of the play (`dword_5C2538`) stops it, and is read
//! (`drive::read_replay_frame`). Not open (`dword_5BEC88`), at or past the last block, or
//! the read failing: `dword_5BEC90` 1 and carry. Else the block and frame stored and ebx
//! the frame's inputs (`dword_9613A4 + 0x2000 + frame * 16`).
//!
//! In:  nothing
//! Out: carry set when it did not go on; ebx the frame's inputs (on the ways that stop,
//!      the frame counted so far); ZF set at a block's first frame; eax kept (pushed)
//!
//! Caller: `race::replay_viewer` (each frame played; it reads carry). Refuses what
//! `read_replay_frame` refuses (the fatal errors of a block past 0x708). Shadow mode
//! compares (`read_packed` puts the file back). Self-test: the original against the Rust
//! version with `read_replay_frame`'s test replay: not open, frames 5 and 99, the end of
//! the play, the last block.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::a;
use crate::race::viewer::{AT_END, BLOCK, BLOCKS, FRAME, OPEN, PLAY_END, get, put, regs_entry};

/// The Rust version: the registers and flags it leaves.
pub fn step_viewed_replay(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    let stop = |r: &mut Regs, at_end: bool| {
        if at_end {
            put(AT_END, 1);
        }
        r.set_carry(true);
    };
    // Converted: `rc2k_core::race::viewer::step`.
    use rc2k_core::race::viewer::{Step, step};
    let open = get(OPEN) == 1;
    let (block, frame) = (get(BLOCK), get(FRAME));
    let (block, frame) = match step(open, block, frame, get(BLOCKS), get(PLAY_END)) {
        Step::Closed => {
            stop(&mut r, true);
            r.set_zero(false);
            return Ok(r);
        }
        Step::AtLast { last } => {
            r.ebx = frame;
            stop(&mut r, true);
            r.set_zero(last);
            return Ok(r);
        }
        Step::PlayEnd { end } => {
            r.ebx = 0;
            stop(&mut r, false);
            r.set_zero(end);
            return Ok(r);
        }
        Step::On { block, frame, read } => {
            r.ebx = frame;
            if read {
                let failed =
                    { unsafe { crate::drive::read_replay_frame::read_replay_frame(block, 0) } }?;
                if failed {
                    stop(&mut r, true);
                    return Ok(r);
                }
            }
            (block, frame)
        }
    };
    put(BLOCK, block);
    put(FRAME, frame);
    r.ebx = (a(crate::race::viewer::BLOCK_DATA) + 0x2000).wrapping_add(frame.wrapping_mul(16));
    r.set_carry(false);
    r.set_zero(get(FRAME) == 0);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &step_viewed_replay)
}
