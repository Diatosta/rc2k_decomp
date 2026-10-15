//! `audio::play_primary` (0x40A775): the primary buffer played, looping.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Nothing with A3D or no `g_primaryBuffer`; else `Play` (+0x30) with 0, 0, 1; a failure
//! sets `g_soundBufferLost`. Callers: `audio::set_mode`, the lost buffers' restore
//! (`sub_40A7C9`). See `audio/thread_parts.rs`. Shadow mode runs the original alone.
//! Self-test: a redirected copy on the test DirectSound's buffer, 4 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::test_sound::{Results, TestSound};
use crate::audio::thread_parts::*;
use crate::audio::voices::{BUFFER_LOST_RVA, PRIMARY_RVA};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::play_primary", rva(0x40A775), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe {
        play_primary(
            game_addr(PRIMARY_RVA) as u32,
            game_addr(BUFFER_LOST_RVA) as u32,
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
    let copy = match copy_of(
        &HOOK,
        0x40A775,
        0x34,
        &[0x60, 0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
        &[],
        &[],
        &[(0x506A8C, at), (0x4F82CC, at + 4)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let sound = TestSound::new();
    for primary in [0, sound.buffer(2)] {
        for play in [0, 0x8878_0096u32 as i32] {
            let run = |rust: bool| {
                unsafe { (at as *mut [u32; 2]).write_volatile([primary, 0]) };
                sound.begin(Results {
                    play,
                    ..Results::default()
                });
                let before = Regs::SENTINEL;
                let mut ro = before;
                let done = if rust {
                    unsafe { play_primary(at, at + 4) };
                    true
                } else {
                    unsafe { call_game(&mut ro, copy.addr()) };
                    ro.preserved_except(&before, 0)
                };
                (done, sound.noted(), unsafe {
                    (at as *const [u32; 2]).read_volatile()
                })
            };
            let theirs = run(false);
            let mine = run(true);
            t.check(theirs.0 && theirs == mine, || {
                format!(
                    "primary {primary:#x}, play {play:#x}: original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    drop(vars);
    t.finish()
}
