//! `race::pace_note_calls` (0x441CB4): one pace note's codes into a group's text: from the
//! place ebx (checkpoint x 8 + slot), each code (`race::pace_note_code`) up to a 0 or 'd'
//! (0x64): a corner ('c', 0x63) turned into its sharpness 1 .. 5 (+ 8 long, + 0x10 one
//! way: from note edx + 1's end and start, see `race/pace_notes.rs`); 'b' (0x62) moves the
//! text on without writing; '/', '<', '4' .. '8' over a ')' just written take its place;
//! the rest written as they are, at the text's edi.
//!
//! In:  ebx the place, edx the note before, edi the text's place
//! Out: ebx, esi (its face), eax (0 or 0x64): where it stopped; edi on; ebp a corner's
//!      flags (0, 8, 0x10, 0x18) when there was one; ecx and edx kept (pushed); the
//!      slot `dword_614D34` that place's; no flags read
//!
//! Callers: `race::init_pace_notes` (twice). It jumps back to its fourth byte: hooked by a
//! private copy (its call relocated).
//! Refuses a place or a table place outside their room (see `race/pace_notes.rs`).
//! Self-test: the original and the Rust version on test stages (`pace_notes::TestStage`):
//! places, notes and text places by lot; cases the Rust version refuses are not run.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::pace_notes::{Course, Lot, Machine, TestStage, case, regions};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::pace_note_calls", rva(0x441CB4), thunk, &ORIGINAL)
    .private_copy(0x14D)
    .relocate_calls(&[0x26])
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on a course.
pub fn pace_note_calls(r: &Regs, course: Course) -> Result<Left, crate::errors::Refused> {
    // Converted: `rc2k_core::race::pace_notes::Notes::note_calls`.
    let mut m = Machine::load(r, course);
    m.note_calls()?;
    m.commit();
    Ok(m.left())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &|r| {
        pace_note_calls(r, crate::race::pace_notes::game_course())
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &|r| {
        pace_note_calls(r, crate::race::pace_notes::game_course())
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let stage = TestStage::new();
    let mut lot = Lot(0x1CB4);
    let (mut compared, mut cases) = (0, 0);
    for count in [3u32, 10, 30, 48] {
        let course = stage.course(count);
        for n in 0..12u32 {
            let seed = lot.draw();
            let regs = Regs {
                ebx: lot.below(8 * count),
                edx: lot.below(40),
                edi: if n == 0 { 0 } else { lot.below(0x200) },
                ..Regs::SENTINEL
            };
            cases += 1;
            if case(
                &mut t,
                &format!(
                    "count {count}, seed {seed:#x}, ebx {:#x}, edx {}",
                    regs.ebx, regs.edx
                ),
                call_original,
                regs,
                &|| stage.fill(count, 1, seed),
                &|r| pace_note_calls(r, course),
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
