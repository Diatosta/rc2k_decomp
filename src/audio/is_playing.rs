//! `audio::is_playing` (0x40B8E7): whether a voice's buffer plays.
//!
//! In:  esi = the voice
//! Out: eax = the status `GetStatus` (+0x24) gives; zero = not playing (bit 0 clear);
//!      ecx, edx as the COM method leaves them; everything else preserved
//!
//! A3D (`sub_40CE81`), no buffer. Callers: the turn's update.
//! See `audio/buffer_play.rs`. Shadow mode runs the original alone. Self-test: the original
//! on a voice whose buffer is the test DirectSound's.

use crate::audio::buffer_play::*;
use crate::audio::voices::voice;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::is_playing", rva(0x40B8E7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let status = unsafe { is_playing(voice(r.esi)) }?;
    Ok(Leaves {
        eax: status,
        carry: None,
        zero: Some(status & PLAYING == 0),
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
        &[
            crate::audio::test_sound::Results {
                status: 0,
                ..Default::default()
            },
            crate::audio::test_sound::Results {
                status: 1,
                ..Default::default()
            },
            crate::audio::test_sound::Results {
                status: 5,
                ..Default::default()
            },
            crate::audio::test_sound::Results {
                status: 4,
                ..Default::default()
            },
        ],
    );
    t.finish()
}
