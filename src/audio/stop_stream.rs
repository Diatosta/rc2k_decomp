//! `audio::stop_stream` (0x40C468): a stream player stopped.
//!
//! In:  esi = the player (`AudioStream`)
//! Out: every register preserved
//!
//! See `audio/stream.rs`: not playing; a buffer is stopped (+0x48), released (+8) and
//! forgotten; an open file closed (`files::close`, `g_lastFilePath` 0). Refuses A3D
//! (0x40D1D0). Shadow mode runs the original alone. Self-test: the original on players of
//! the test's (the test DirectSound's buffer, the test's file), 4 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::stream::*;
use crate::errors::Refused;
use crate::files::stream::LastPath;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::stop_stream", rva(0x40C468), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe { stop_stream(stream(r.esi)) }?;
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
    if !make_test_file(false) {
        return t.skip("the test's file cannot be written");
    }
    let last = LastPath::save();
    let sound = crate::audio::test_sound::TestSound::new();
    for with_buffer in [false, true] {
        for open in [0, 1] {
            let run = |original: bool| {
                let h = open_test_file(0).unwrap_or(0);
                let mut s = AudioStream {
                    playing: 1,
                    volume: 0x20,
                    buffer: if with_buffer { sound.buffer(1) } else { 0 },
                    open,
                    file: h,
                    ..AudioStream::default()
                };
                let before = Regs {
                    esi: &mut s as *mut AudioStream as u32,
                    ..Regs::SENTINEL
                };
                sound.begin(Default::default());
                let mut r = before;
                let done = if original {
                    unsafe { call_original(&mut r) };
                    Ok(())
                } else {
                    unsafe { rust(&r) }.map(|_| ())
                };
                let seen = (
                    done.is_ok() && r.preserved_except(&before, 0),
                    without_file(&s),
                    sound.noted(),
                    position(h).is_none(),
                    LastPath::now(),
                );
                if position(h).is_some() {
                    unsafe { crate::files::handle::close(h) };
                }
                seen
            };
            let theirs = run(true);
            let mine = run(false);
            t.check(theirs.0 && theirs == mine, || {
                format!("buffer {with_buffer}, open {open}: original {theirs:x?}; rust {mine:x?}")
            });
        }
    }
    last.restore();
    remove_test_file();
    t.finish()
}
