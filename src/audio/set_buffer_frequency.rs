//! `audio::set_buffer_frequency` (0x40B97A): the voice's frequency to its buffer.
//!
//! In:  esi = the voice
//! Out: eax = what the COM method returns; ecx, edx as it leaves them (no caller reads
//!      them); everything else preserved
//!
//! `SetFrequency` (+0x44) with the frequency shifted right by the format's four low bits.
//! Refuses A3D (`sub_40CEC5`), no buffer.
//! Callers: the turn's update (`sub_40AC77`) and `audio::make_buffer` (`sub_40B5CF`).
//! See `audio/buffer_settings.rs`. Shadow mode runs the original alone (DirectSound's
//! call). Self-test: the original on voices whose buffers are test objects.

use crate::audio::buffer_settings::*;
use crate::audio::voices::Voice;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "audio::set_buffer_frequency",
    rva(0x40B97A),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(v: &Voice) -> Result<Option<i32>, Refused> {
    Ok(Some(unsafe { set_buffer_frequency(v) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_setting(&HOOK, r, call_original, rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let voices = [100u32, 11025, 22050, 44100, 200_000]
        .into_iter()
        .flat_map(|frequency| {
            [0x110u32, 0x221, 0x123, 0x20F].map(|format| Voice {
                frequency,
                format,
                ..Voice::default()
            })
        })
        .collect::<Vec<_>>();
    check_setting(&mut t, call_original, rust, &voices);
    t.finish()
}
