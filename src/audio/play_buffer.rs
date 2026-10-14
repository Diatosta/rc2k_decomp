//! `audio::play_buffer` (0x40B922): a voice's buffer played.
//!
//! In:  esi = the voice
//! Out: eax = what `Play` (+0x30) returns; carry = it failed; ecx, edx as it leaves them; everything else preserved
//!
//! `Play` with 0, 0 and the looping flag for a voice of kind 1; failing, `DSERR_BUFFERLOST`
//! (0x88780096) sets `g_soundBufferLost`. Refuses A3D (`sub_40CEA0`), no buffer. Callers:
//! the turn's update, `sub_40A7C9`.
//! See `audio/buffer_play.rs`. Shadow mode runs the original alone. Self-test: the original
//! on a voice whose buffer is the test DirectSound's.

use crate::audio::buffer_play::*;
use crate::audio::voices::voice;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::play_buffer", rva(0x40B922), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let (eax, carry) = unsafe { play_buffer_in_game(voice(r.esi)) }?;
    Ok(Leaves {
        eax: eax as u32,
        carry: Some(carry),
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
        &[0, 1, 3],
        &[
            crate::audio::test_sound::Results::default(),
            crate::audio::test_sound::Results {
                play: 0x8878_0096u32 as i32,
                ..Default::default()
            },
            crate::audio::test_sound::Results {
                play: -1,
                ..Default::default()
            },
        ],
    );
    t.finish()
}
