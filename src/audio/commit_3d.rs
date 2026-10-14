//! `audio::commit_3d` (0x40C6A9): the 3D settings made so far deferred, carried out.
//!
//! In:  nothing
//! Out: eax = what `CommitDeferredSettings` (+0x44) returns; ecx, edx as it leaves them;
//!      everything else preserved
//!
//! On the 3D listener `g_soundListener` (0x506A90). Callers: the turn's update when
//! `g_commit3d` is 1, and the listener's set-up.
//! Refuses no listener (the original calls through it).
//! Shadow mode runs the original alone. Self-test: a redirected copy whose listener is the
//! test DirectSound's.

use crate::audio::buffer_play::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::commit_3d", rva(0x40C6A9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(_: &Regs) -> Result<Leaves, Refused> {
    Ok(Leaves {
        eax: unsafe { commit_3d(game_addr(LISTENER_3D_RVA) as u32) }? as u32,
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
    let sound = crate::audio::test_sound::TestSound::new();
    let mut var = Box::new(sound.listener());
    let at = &mut *var as *mut u32 as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40C6A9,
            len: 0xC,
            entry: &[0xA1, 0x90, 0x6A, 0x50, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x506A90, at)],
        1,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    sound.begin(Default::default());
    let mut ro = Regs::SENTINEL;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = sound.noted();
    sound.begin(Default::default());
    let mine = unsafe { commit_3d(at) };
    let noted = sound.noted();
    t.check(
        mine.as_ref().is_ok_and(|&e| e as u32 == ro.eax) && theirs == noted,
        || {
            format!(
                "original {:#x} {theirs:x?}; rust {mine:?} {noted:x?}",
                ro.eax
            )
        },
    );
    t.finish()
}
