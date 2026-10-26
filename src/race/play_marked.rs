//! The viewer's two plays (IDA 2026-10-19), sharing one tail (0x41E231):
//! - `race::play_marked` (0x41E224): from the first mark (`dword_96E478`, frame 0) to the
//!   second (`dword_96E47C`).
//! - `race::play_from_here` (0x41E1E2): from the block and frame shown (`dword_5C2530`,
//!   `dword_5C2534`) to the end (`dword_5BECD0`); it jumps into `play_marked`'s tail.
//!
//! The tail: the block to stop at into `dword_5C2538`, `dword_5BEC98` 0 (playing), the
//! start read (`drive::read_replay_frame`, eax the block, ebx the frame; its carry not
//! looked at), the play (`race::play_replay_range`), `dword_5C253C` 0.
//!
//! In:  nothing
//! Out: eax the block played from, ebx and ecx its frame; no flags read
//!
//! Callers: `race::viewer_action` (0x1A, 0x25 .. 0x28 `play_marked`; 6 and 0x22, 0x23
//! `play_from_here`). Their calls through `part`: 1 `read_replay_frame`, 2
//! `play_replay_range`; in the game their Rust versions. Ungated (they play a replay).
//! Shadow mode runs the original alone. Self-test: a copy of `play_marked` whose calls
//! go to stand-ins, from its start and (for `play_from_here`) from its tail with the
//! registers `play_from_here` sets, against the Rust versions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{
    BLOCK, BLOCKS, FRAME, MARK_IN, MARK_OUT, PLAY_END, PLAY_FROM_HERE, Part, STOPPED, Saved,
    dispatch_parts, dword, get, put, regs_entry, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::play_marked", rva(0x41E224), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

/// The shared tail (0x41E231): eax the block, ebx the block to stop at, ecx the frame.
pub fn play_tail(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    put(PLAY_END, r.ebx);
    put(STOPPED, 0);
    r.ebx = r.ecx;
    part(1, &mut r)?;
    part(2, &mut r)?;
    put(PLAY_FROM_HERE, 0);
    Ok(r)
}

/// `play_marked`.
pub fn play_marked(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::marked_play`.
    let play = rc2k_core::race::viewer_flow::marked_play(get(MARK_IN), get(MARK_OUT));
    play_tail(&play_regs(entry, play), part)
}

/// The tail's registers for a play: eax the block, ebx the block to stop at, ecx the frame.
fn play_regs(entry: &Regs, play: rc2k_core::race::viewer_flow::Play) -> Regs {
    Regs {
        eax: play.from,
        ebx: play.end,
        ecx: play.frame,
        ..*entry
    }
}

/// `play_from_here`.
pub fn play_from_here(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::play_from_here`.
    let play = rc2k_core::race::viewer_flow::play_from_here(get(BLOCK), get(FRAME), get(BLOCKS));
    play_tail(&play_regs(entry, play), part)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 1 {
        let failed = unsafe { crate::drive::read_replay_frame::read_replay_frame(r.eax, r.ebx) }?;
        r.set_carry(failed);
    } else {
        *r = crate::race::play_replay_range::play_replay_range(
            r,
            &mut crate::race::play_replay_range::game_part,
        )?;
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| play_marked(r, &mut game_part));
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| play_marked(r, &mut game_part))
}

/// The test of either: `from_here` enters the copy at the tail.
pub fn test_play(hook: &RawHook, from_here: bool) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    // mov eax, dword_96E478
    let copy = match crate::race::viewer::copy(
        0x41E224,
        0x34,
        &[0xA1, 0x78, 0xE4, 0x96, 0x00],
        &[0x1F, 0x24],
        &[(0x41EBAB, 1), (0x41D846, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::race::viewer::quiet();
    let regions = [
        dword(PLAY_END),
        dword(STOPPED),
        dword(PLAY_FROM_HERE),
        dword(MARK_IN),
        dword(MARK_OUT),
        dword(BLOCK),
        dword(BLOCKS),
        dword(FRAME),
    ];
    let saved = Saved::new(&regions);
    let prepare = || {
        put(MARK_IN, 3);
        put(MARK_OUT, 8);
        put(BLOCK, 5);
        put(BLOCKS, 9);
        put(FRAME, 0x2A);
        put(STOPPED, 1);
        put(PLAY_FROM_HERE, 1);
        put(PLAY_END, 0xAAAA_AAAA);
    };
    if from_here {
        let before = Regs {
            eax: 5,
            ebx: 9,
            ecx: 0x2A,
            ..Regs::SENTINEL
        };
        run_both(
            &mut t,
            "from the block shown",
            copy.addr() + 0xD,
            before,
            &regions,
            0,
            &prepare,
            &|_| play_from_here(&Regs::SENTINEL, &mut test_part),
        );
    } else {
        run_both(
            &mut t,
            "between the marks",
            copy.addr(),
            Regs::SENTINEL,
            &regions,
            0,
            &prepare,
            &|r| play_marked(r, &mut test_part),
        );
    }
    drop(saved);
    t.finish()
}

fn self_test() -> (usize, usize) {
    test_play(&HOOK, false)
}
