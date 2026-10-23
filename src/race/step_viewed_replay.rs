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

use crate::drive::read_replay_frame::{regions as read_regions, test_regions, test_replay};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, guard};
use crate::race::viewer::{
    AT_END, BLOCK, BLOCKS, CF, FRAME, OPEN, PLAY_END, Saved, ZF, check_original, dispatch_regs,
    dword, get, put, regs_entry,
};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::step_viewed_replay", rva(0x41EC47), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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
                let failed = guard(&regions(), || unsafe {
                    crate::drive::read_replay_frame::read_replay_frame(block, 0)
                })?;
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

fn regions() -> Vec<Region> {
    let mut all = read_regions();
    all.push(dword(AT_END));
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &regions(), CF, &step_viewed_replay);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &step_viewed_replay)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = test_regions();
    all.extend(regions());
    all.push(dword(PLAY_END));
    let saved = Saved::new(&all);
    let Some(file) = test_replay() else {
        drop(saved);
        return t.skip("no test file in the temp folder");
    };
    // (open, block, frame, end of the play)
    for (open, block, frame, end) in [
        (0u32, 0u32, 5u32, 2u32),
        (1, 0, 5, 2),
        (1, 0, 99, 2),
        (1, 0, 99, 1),
        (1, 1, 99, 2),
        (1, 2, 0, 2),
        (1, 1, 98, 0),
    ] {
        check_original(
            &mut t,
            &format!("open {open}, block {block}, frame {frame}, end {end}"),
            call_original,
            Regs::SENTINEL,
            &regions(),
            CF | ZF,
            &|| {
                put(OPEN, open);
                put(BLOCK, block);
                put(FRAME, frame);
                put(PLAY_END, end);
                put(AT_END, 0xAA);
                file.rewind();
            },
            &step_viewed_replay,
        );
    }
    drop(file);
    drop(saved);
    t.finish()
}
