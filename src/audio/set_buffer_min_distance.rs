//! `audio::set_buffer_min_distance` (0x40C69A): a voice's 3D buffer's minimum distance.
//!
//! In:  esi = the voice
//! Out: eax = the result; ecx, edx as the method leaves them
//!
//! `SetMinDistance` (+0x44) of its `buffer_3d` with its `min_distance` (an f32),
//! deferred (1). One caller, `audio::make_buffer`. Refuses a voice without its 3D buffer
//! (the original calls through 0). Shadow mode runs the original alone. Self-test: the
//! original on voices whose 3D buffer is the test DirectSound's, 3 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::test_sound::{Results, TestSound};
use crate::audio::voices::Voice;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "audio::set_buffer_min_distance",
    rva(0x40C69A),
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

const SET_MIN_DISTANCE: u32 = 0x44;
const DEFERRED: u32 = 1;

pub unsafe fn set_buffer_min_distance(v: &Voice) -> Result<i32, Refused> {
    if v.buffer_3d == 0 {
        return Err(BadInput("a voice without its 3D buffer").into());
    }
    unsafe {
        let f: unsafe extern "system" fn(u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(v.buffer_3d, SET_MIN_DISTANCE));
        Ok(f(v.buffer_3d, v.min_distance, DEFERRED))
    }
}

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let eax = unsafe { set_buffer_min_distance(&*(r.esi as *const Voice)) }?;
    Ok(Leaves {
        eax: eax as u32,
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
    let sound = TestSound::new();
    for (n, distance) in [(0, 1.0f32), (1, 0.5), (0, 12.25)] {
        let v = Voice {
            buffer_3d: sound.three_d(n),
            min_distance: distance.to_bits(),
            ..Voice::default()
        };
        let before = Regs {
            esi: &v as *const Voice as u32,
            ..Regs::SENTINEL
        };
        sound.begin(Results::default());
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let theirs = (ro.eax, sound.noted());
        let preserved = ro.preserved_except(
            &before,
            crate::framework::EAX | crate::framework::ECX | crate::framework::EDX,
        );
        sound.begin(Results::default());
        let mine = unsafe { set_buffer_min_distance(&v) }.map(|e| (e as u32, sound.noted()));
        t.check(preserved && mine.as_ref() == Ok(&theirs), || {
            format!("buffer {n}, distance {distance}: original {theirs:x?} preserved {preserved}; rust {mine:x?}")
        });
    }
    t.finish()
}
