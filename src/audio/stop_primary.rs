//! `audio::stop_primary` (0x40A7A9): the primary buffer stopped.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Nothing with A3D or no `g_primaryBuffer`; else `Stop` (+0x48). One caller, the primary
//! buffer's release (`sub_40A61F`). See `audio/thread_parts.rs`. Shadow mode runs the
//! original alone. Self-test: a redirected copy on the test DirectSound's buffer, 2 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::test_sound::TestSound;
use crate::audio::thread_parts::*;
use crate::audio::voices::PRIMARY_RVA;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::stop_primary", rva(0x40A7A9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe { stop_primary(game_addr(PRIMARY_RVA) as u32) };
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
    let mut vars = Box::new([0u32; 1]);
    let at = vars.as_mut_ptr() as u32;
    let copy = match copy_of(
        &HOOK,
        0x40A7A9,
        0x20,
        &[0x60, 0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
        &[],
        &[],
        &[(0x506A8C, at)],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let sound = TestSound::new();
    for primary in [0, sound.buffer(2)] {
        let run = |rust: bool| {
            unsafe { (at as *mut u32).write_volatile(primary) };
            sound.begin(Default::default());
            let before = Regs::SENTINEL;
            let mut ro = before;
            let done = if rust {
                unsafe { stop_primary(at) };
                true
            } else {
                unsafe { call_game(&mut ro, copy.addr()) };
                ro.preserved_except(&before, 0)
            };
            (done, sound.noted(), unsafe {
                (at as *const u32).read_volatile()
            })
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!("primary {primary:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop(vars);
    t.finish()
}
