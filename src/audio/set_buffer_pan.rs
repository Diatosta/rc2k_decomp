//! `audio::set_buffer_pan` (0x40B9C6): the voice's pan to its buffer.
//!
//! In:  esi = the voice
//! Out: eax = what the COM method returns; ecx, edx as it leaves them (no caller reads
//!      them); everything else preserved
//!
//! nothing in 3D (`g_soundMode` bit 0x100000); else `SetPan` (+0x40) with
//! `g_volumeTable[pan + 0x40]` for a pan below 0, else its negation at `0x40 - pan`.
//! Refuses A3D (`nullsub_45`), a pan outside -0x40 .. 0x40, no buffer.
//! Callers: the turn's update (`sub_40AC77`) and `audio::make_buffer` (`sub_40B5CF`).
//! See `audio/buffer_settings.rs`. Shadow mode runs the original alone (DirectSound's
//! call). Self-test: the original on voices whose buffers are test objects.

use crate::audio::buffer_settings::*;
use crate::audio::voices::Voice;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::set_buffer_pan", rva(0x40B9C6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(v: &Voice) -> Result<Option<i32>, Refused> {
    unsafe { set_buffer_pan(v) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_setting(&HOOK, r, call_original, rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let voices = (-0x40..=0x40i32)
        .step_by(4)
        .map(|pan| Voice {
            pan,
            ..Voice::default()
        })
        .collect::<Vec<_>>();
    check_setting(&mut t, call_original, rust, &voices);
    t.finish()
}
