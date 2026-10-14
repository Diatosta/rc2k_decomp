//! `audio::set_buffer_volume` (0x40B99D): the voice's volume to its buffer.
//!
//! In:  esi = the voice
//! Out: eax = what the COM method returns; ecx, edx as it leaves them (no caller reads
//!      them); everything else preserved
//!
//! `SetVolume` (+0x3C) with `g_volumeTable[volume * volume_factor >> 6]` (signed).
//! Refuses A3D (`sub_40CEFE`), an index outside 0 .. 0x40, no buffer.
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
    RawHook::new("audio::set_buffer_volume", rva(0x40B99D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(v: &Voice) -> Result<Option<i32>, Refused> {
    Ok(Some(unsafe { set_buffer_volume(v) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_setting(&HOOK, r, call_original, rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let voices = (0..=0x40u32)
        .step_by(8)
        .flat_map(|volume| {
            [0x40u32, 0x20, 0x10, 1].map(|factor| Voice {
                volume,
                volume_factor: factor,
                ..Voice::default()
            })
        })
        .collect::<Vec<_>>();
    check_setting(&mut t, call_original, rust, &voices);
    t.finish()
}
