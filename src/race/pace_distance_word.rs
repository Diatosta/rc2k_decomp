//! `race::pace_distance_word` (0x441C78): a pace note group's first word, for the distance
//! before it: eax (16.16) >> 16 / 10 (by `div`, `mul`, `div`), at most 40, its word from
//! `byte_614D38`; unless 0, it goes into the text (`byte_61BE18`) at edi and edi moves on.
//!
//! In:  eax the distance, edi the text's place
//! Out: edi (+1 when a word was written); the rest kept (`pusha` / `popa`, the dead load
//!      of `dword_615D6C[word]` into eax popped); no flags read
//!
//! Caller: `race::init_pace_notes`. See `race/pace_notes.rs`.
//! Refuses a place outside the pace tables (the original writes past the text).
//! Self-test: the original and the Rust version on the game's tables (put back): distances
//! of every word, below and past 40 tens, places in the text.

use crate::framework::{EDI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::pace_notes::{Machine, TestStage, case, regions};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::pace_distance_word", rva(0x441C78), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub fn pace_distance_word(r: &Regs) -> Result<Left, crate::errors::Refused> {
    let mut m = Machine::load(r, crate::race::pace_notes::game_course());
    m.distance_word()?;
    m.commit();
    Ok(Left::nothing().with(EDI, m.r.edi))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &pace_distance_word);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &pace_distance_word)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let stage = TestStage::new();
    for tens in [0u32, 1, 9, 10, 23, 39, 40, 41, 0xFFFF] {
        for edi in [0u32, 7, 0x3FFF] {
            let eax = (tens * 10 + tens % 10) << 16 | 0x8001;
            case(
                &mut t,
                &format!("eax {eax:#x}, edi {edi:#x}"),
                call_original,
                Regs {
                    eax,
                    edi,
                    ..Regs::SENTINEL
                },
                &|| stage.fill(6, 1, tens),
                &pace_distance_word,
            );
        }
    }
    drop(stage);
    t.finish()
}
