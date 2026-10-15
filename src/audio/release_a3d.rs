//! `audio::release_a3d` (0x40C8A2): the A3D object given back.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `audio/sound_device.rs`. Shadow mode runs the original alone. Self-test: a redirected copy with the test DirectSound's buffer as the object, 2 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::sound_device::*;
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::release_a3d", rva(0x40C8A2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe {
        release_a3d(
            game_addr(A3D_OBJECT_RVA) as u32,
            game_addr(A3D_READY_RVA) as u32,
        )
    };
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
    let mut vars = Box::new([0u32; 2]);
    let at = vars.as_mut_ptr() as u32;
    let c = match copy(
        &HOOK,
        0x40C8A2,
        0x2B,
        &[0x60, 0x83, 0x3D, 0x68, 0x6C, 0x50, 0x00, 0x00],
        &[],
        &[],
        &[(0x506C68, at), (0x506C64, at + 4)],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let sound = TestSound::new();
    for object in [0, sound.buffer(3)] {
        let run = |rust: bool| {
            unsafe { (at as *mut [u32; 2]).write_volatile([object, 0x55]) };
            sound.begin(Results::default());
            let before = Regs::SENTINEL;
            let mut ro = before;
            let done = if rust {
                unsafe { release_a3d(at, at + 4) };
                true
            } else {
                unsafe { call_game(&mut ro, c.addr()) };
                ro.preserved_except(&before, 0)
            };
            (done, sound.noted(), unsafe {
                (at as *const [u32; 2]).read_volatile()
            })
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!("object {object:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop(vars);
    t.finish()
}
