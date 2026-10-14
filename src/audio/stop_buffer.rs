//! `audio::stop_buffer` (0x40B963): a voice's buffer stopped.
//!
//! In:  esi = the voice
//! Out: eax = what `Stop` (+0x48) returns; ecx, edx as it leaves them; everything else preserved
//!
//! Refuses A3D (`sub_40CEBB`), no buffer. Callers: the turn's update,
//! `audio::release_voice`.
//! See `audio/buffer_play.rs`. Shadow mode runs the original alone. Self-test: the original
//! on a voice whose buffer is the test DirectSound's.

use crate::audio::buffer_play::*;
use crate::audio::voices::voice;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::stop_buffer", rva(0x40B963), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    Ok(Leaves {
        eax: unsafe { stop_buffer(voice(r.esi)) }? as u32,
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
        &[0],
        &[crate::audio::test_sound::Results::default()],
    );
    t.finish()
}
