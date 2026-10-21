//! `race::pace_note_code` (0x441A37): the pace note code of the face at esi in the
//! current slot: the byte at esi + `dword_614D14[dword_614D34]` (see `race/pace_notes.rs`).
//!
//! In:  esi the face, `dword_614D34` the slot
//! Out: eax the code (`movzx`); the rest kept (ebx pushed and popped); no flags read
//!
//! Callers: `race::init_pace_notes` (twice), `race::pace_note_calls`.
//! Refuses a slot past 7 and a face outside the course memory (the original reads past its
//! table or the course).
//! Self-test: the original and the Rust version on a test stage (`pace_notes::TestStage`),
//! every slot of several faces.

use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::pace_notes::{Course, Machine, TestStage, case};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::pace_note_code", rva(0x441A37), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on a course.
pub fn pace_note_code(r: &Regs, course: Course) -> Result<Left, crate::errors::Refused> {
    let mut m = Machine::load(r, course);
    m.code()?;
    Ok(Left::nothing().with(EAX, m.r.eax))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &|r| {
        pace_note_code(r, crate::race::pace_notes::game_course())
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let stage = TestStage::new();
    let count = 12;
    let course = stage.course(count);
    for face in [0u32, 3, 11, 13] {
        for slot in 0..8u32 {
            let fill = || {
                stage.fill(count, 2, 0x51 + face);
                unsafe { crate::framework::game_write(crate::race::pace_notes::SLOT_RVA, slot) };
            };
            case(
                &mut t,
                &format!("face {face}, slot {slot}"),
                call_original,
                Regs {
                    esi: course.faces + 64 * face,
                    ..Regs::SENTINEL
                },
                &fill,
                &|r| pace_note_code(r, course),
            );
        }
    }
    drop(stage);
    t.finish()
}
