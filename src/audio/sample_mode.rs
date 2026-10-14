//! `audio::sample_mode` (0x40B398): the sound mode a voice's sample is prepared for.
//!
//! In:  eax = the sound mode, esi = the voice
//! Out: eax = the mode for it; everything else preserved
//!
//! For a voice whose sound is not negative, by its sample record: sound 0x1C4 keeps only
//! the halvings (low four bits); groups 1 and 2 at least one halving, group 3 at least
//! two; the rest as it is. One caller, command 3 (`sub_40AEB9`).
//! Refuses a sound of 0x1CD or more.
//! Compared in shadow mode. Self-test: the original on every sound under eight modes.

use crate::audio::samples::sample_mode;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::sample_mode", rva(0x40B398), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub unsafe fn sample_mode_of(mode: u32, sound: u32) -> Result<u32, Refused> {
    if (sound as i32) < 0 {
        return Ok(mode);
    }
    if sound >= SAMPLES {
        return Err(BadInput("a sound of 0x1CD or more").into());
    }
    Ok(sample_mode(
        mode,
        Some(unsafe { sample(&World::game(), sound) }),
    ))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let sound = unsafe { voice(r.esi).sound };
    Ok(Left::nothing().with(EAX, unsafe { sample_mode_of(r.eax, sound) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut v = Box::new(Voice::default());
    for sound in (0..SAMPLES).chain([0xFFFF_FFFF, 0x8000_0000]) {
        for mode in [0u32, 1, 2, 3, 0x11, 0x2441220, 0x11221, 0xFFFF_FFFF] {
            v.sound = sound;
            let before = Regs {
                eax: mode,
                esi: &*v as *const Voice as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let mine = unsafe { sample_mode_of(mode, sound) };
            t.check(
                mine.as_ref().is_ok_and(|&m| m == ro.eax) && ro.preserved_except(&before, EAX),
                || {
                    format!(
                        "sound {sound:#x}, mode {mode:#x}: original {:#x}; rust {mine:?}",
                        ro.eax
                    )
                },
            );
        }
    }
    t.finish()
}
