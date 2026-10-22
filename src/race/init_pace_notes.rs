//! `race::init_pace_notes` (0x441A66): loadlevel_init's "init pacenotes": the pace notes'
//! tables made from the checkpoints' faces (see `race/pace_notes.rs`). `dword_615E14` and
//! `dword_615E10` 0; with fewer than two checkpoints nothing more. Else the start (the
//! first checkpoint of face kind 2, or the count, + 2); the notes (start / end / the end's
//! code); each note's distance to the next one's start; the groups (`race::
//! pace_distance_word`, `race::pace_note_calls` for each note, ')' between notes joined,
//! a NUL after each group but the last) and their count.
//!
//! In:  nothing
//! Out: fewer than two checkpoints: every register kept; else eax, ebx, esi as the last
//!      walk left them, ecx the groups less one, edx the notes' count plus one, edi the
//!      text's end, ebp the search's end or a corner's flags; no flags read (loadlevel
//!      goes on with its log)
//!
//! Caller: `race::loadlevel_init` (its step 25), which calls the Rust version directly.
//! Refuses what `race/pace_notes.rs` refuses: places outside the tables or the course,
//! and a search that would run on past the count (no face of kind 2 before the last
//! checkpoint but one). Nothing is written then.
//! Self-test: the original and the Rust version on test stages (`pace_notes::TestStage`):
//! counts 0 .. 48, codes and distances by lot; cases the Rust version refuses are not run.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::pace_notes::{Course, Lot, Machine, TestStage, case, regions};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::init_pace_notes", rva(0x441A66), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on a course.
pub fn init_pace_notes_on(r: &Regs, course: Course) -> Result<Left, Refused> {
    // Converted: `rc2k_core::race::pace_notes::Notes::init`.
    let mut m = Machine::load(r, course);
    m.init()?;
    m.commit();
    Ok(m.left())
}

/// The Rust version on the game's course: the registers it leaves.
pub fn init_pace_notes(r: &mut Regs) -> Result<(), Refused> {
    init_pace_notes_on(r, crate::race::pace_notes::game_course())?.apply(r);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &|r| {
        init_pace_notes_on(r, crate::race::pace_notes::game_course())
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &|r| {
        init_pace_notes_on(r, crate::race::pace_notes::game_course())
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let stage = TestStage::new();
    let mut lot = Lot(0x441A66);
    let (mut compared, mut cases) = (0, 0);
    for count in [0u32, 1, 2, 3, 5, 9, 16, 30, 48] {
        let course = stage.course(count);
        for n in 0..6u32 {
            let seed = lot.draw();
            // The first face of kind 2: early, late, and none (a refusal).
            let kind2 = match n {
                0 => 0,
                5 => count + 3,
                _ => lot.below(count.saturating_sub(2).max(1)),
            };
            cases += 1;
            if case(
                &mut t,
                &format!("count {count}, kind 2 at {kind2}, seed {seed:#x}"),
                call_original,
                Regs::SENTINEL,
                &|| stage.fill(count, kind2, seed),
                &|r| init_pace_notes_on(r, course),
            ) {
                compared += 1;
            }
        }
    }
    t.check(compared * 2 >= cases, || {
        format!("only {compared} of {cases} cases compared (the rest refused)")
    });
    drop(stage);
    t.finish()
}
