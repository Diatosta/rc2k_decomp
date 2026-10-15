//! `audio::set_stream_volume` (0x40C4B2): a stream player's volume given to its buffer.
//!
//! In:  esi = the player (`AudioStream`)
//! Out: every register preserved
//!
//! `SetVolume` (+0x3C) with `g_volumeTable[volume]`. Refuses A3D (0x40D247), no buffer and
//! a volume past the table (the original does not look). Callers: the two commands'
//! action 3, while playing. Shadow mode runs the original alone. Self-test: the original
//! on players of the test's with the test DirectSound's buffer, 5 volumes.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::stream::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::set_stream_volume", rva(0x40C4B2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe { set_stream_volume(stream(r.esi)) }?;
    Ok(Leaves {
        eax: r.eax,
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
    if crate::audio::buffer_settings::a3d() {
        return t.skip("A3D is on");
    }
    let sound = crate::audio::test_sound::TestSound::new();
    for volume in [0, 1, 0x20, 0x3F, 0x40] {
        let mut s = AudioStream {
            playing: 1,
            volume,
            buffer: sound.buffer(1),
            ..AudioStream::default()
        };
        let before = Regs {
            esi: &mut s as *mut AudioStream as u32,
            ..Regs::SENTINEL
        };
        sound.begin(Default::default());
        let mut r = before;
        unsafe { call_original(&mut r) };
        let theirs = (r.preserved_except(&before, 0), sound.noted());
        sound.begin(Default::default());
        let mine = unsafe { rust(&before) }.map(|_| (true, sound.noted()));
        t.check(
            theirs.0 && mine.as_ref().is_ok_and(|m| *m == theirs),
            || format!("volume {volume:#x}: original {theirs:x?}; rust {mine:x?}"),
        );
    }
    t.finish()
}
