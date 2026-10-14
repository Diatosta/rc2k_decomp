//! `audio::set_buffer_place` (0x40C676): the voice's place to its 3D buffer.
//!
//! In:  esi = the voice
//! Out: eax = what the COM method returns; ecx, edx as it leaves them (no caller reads
//!      them); everything else preserved
//!
//! the 3D buffer's `SetPosition` (+0x4C) with the place's three floats, deferred (1)
//! while the voice sounds (state 2), else at once (0). Only in 3D modes.
//! Refuses no 3D buffer.
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
    RawHook::new("audio::set_buffer_place", rva(0x40C676), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(v: &Voice) -> Result<Option<i32>, Refused> {
    Ok(Some(unsafe { set_buffer_place(v) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_setting(&HOOK, r, call_original, rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let voices = [0u32, 1, 2, 3]
        .into_iter()
        .map(|state| Voice {
            state,
            place: [0x3F80_0000, 0xC120_0000 ^ state, state],
            ..Voice::default()
        })
        .collect::<Vec<_>>();
    check_setting(&mut t, call_original, rust, &voices);
    t.finish()
}
