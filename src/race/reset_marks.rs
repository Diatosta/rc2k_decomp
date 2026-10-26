//! `race::reset_marks` (0x41E403): after an edit, the block shown held to the replay's
//! blocks (`dword_5C2530` at most `dword_5BECD0`, signed), the marks round the whole
//! replay (`dword_96E478` 0, `dword_96E47C` the count) and the block shown read again
//! (`drive::read_replay_frame_0`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::viewer_action` (the cut and the paste), `race::undo_edit`. Refuses what
//! `read_replay_frame_0` refuses (what it wrote put back). Shadow mode compares (the
//! file put back where it was by `read_packed`). Self-test: the original against the
//! Rust version with `read_replay_frame`'s test replay: blocks below, at and past the
//! count, not playing.

use crate::drive::read_replay_frame::{regions as read_regions, test_regions, test_replay};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::guard;
use crate::race::viewer::{
    BLOCK, BLOCKS, MARK_IN, MARK_OUT, OPEN, Saved, check_original, dispatch_regs, dword, get, put,
    regs_entry,
};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::reset_marks", rva(0x41E403), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub fn reset_marks() -> Result<(), Refused> {
    guard(&regions(), || {
        // Converted: `rc2k_core::race::viewer::reset_marks`.
        let (block, mark_in, mark_out) =
            rc2k_core::race::viewer::reset_marks(get(BLOCK), get(BLOCKS));
        put(BLOCK, block);
        put(MARK_IN, mark_in);
        put(MARK_OUT, mark_out);
        unsafe { crate::drive::read_replay_frame_0::read_replay_frame_0(get(BLOCK)) }?;
        Ok(())
    })
}

pub fn regions() -> Vec<Region> {
    let mut all = read_regions();
    all.extend([dword(MARK_IN), dword(MARK_OUT)]);
    all
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    reset_marks()?;
    Ok(*r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &regions(), 0, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = test_regions();
    all.extend(regions());
    let saved = Saved::new(&all);
    let Some(file) = test_replay() else {
        drop(saved);
        return t.skip("no test file in the temp folder");
    };
    for (block, playing) in [(0u32, 1u32), (1, 1), (2, 1), (5, 1), (1, 0)] {
        // (A block below 0 stays so and is the read's fatal error: not tested.)
        check_original(
            &mut t,
            &format!("block {block:#x}, playing {playing}"),
            call_original,
            Regs::SENTINEL,
            &regions(),
            0,
            &|| {
                put(BLOCK, block);
                put(OPEN, playing);
                put(MARK_IN, 0xAAAA_AAAA);
                put(MARK_OUT, 0xAAAA_AAAA);
                file.rewind();
            },
            &rust,
        );
    }
    drop(file);
    drop(saved);
    t.finish()
}
