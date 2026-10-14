//! `audio::rewind_buffer` (0x40B909): a voice's buffer back to its start.
//!
//! In:  esi = the voice
//! Out: eax = what `SetCurrentPosition` (+0x34, with 0) returns; ecx, edx as it leaves
//!      them; everything else preserved
//!
//! A3D (`sub_40CE96`), no buffer. Callers: the turn's update.
//! See `audio/buffer_play.rs`. Shadow mode runs the original alone. Self-test: the original
//! on a voice whose buffer is the test DirectSound's.

use crate::audio::buffer_play::*;
use crate::audio::voices::voice;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::rewind_buffer", rva(0x40B909), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    Ok(Leaves {
        eax: unsafe { rewind_buffer(voice(r.esi)) }? as u32,
        carry: None,
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_leaf(
        &mut t,
        call_original,
        rust,
        &[0, 1],
        &[crate::audio::test_sound::Results::default()],
    );
    t.finish()
}
