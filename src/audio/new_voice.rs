//! `audio::new_voice` (0x40B04C): the next voice of the pool.
//!
//! In:  nothing
//! Out: esi = the voice; everything else preserved
//!
//! `g_voices[g_voiceCount]`, and the count + 1; with 0x200 in use fatal error 0x80A28501.
//! One caller, command 3 (`sub_40AEB9`).
//! Compared in shadow mode. Self-test: a redirected copy on a pool of the test's, 5 cases.

use crate::audio::voices::*;
use crate::errors::{FatalError, Refused};
use crate::framework::{ESI, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{at, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::new_voice", rva(0x40B04C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `count`: where the count is kept; `voices`: the pool.
pub unsafe fn new_voice(count: u32, voices: u32) -> Result<u32, Refused> {
    unsafe {
        let n = read(count);
        if n >= VOICES {
            return Err(FatalError {
                code: 0x80A2_8501,
                message: "",
            }
            .into());
        }
        write(count, n + 1);
        Ok(voices + n * std::mem::size_of::<Voice>() as u32)
    }
}

pub unsafe fn new_voice_in_game() -> Result<u32, Refused> {
    unsafe {
        new_voice(
            game_addr(VOICE_COUNT_RVA) as u32,
            game_addr(VOICES_RVA) as u32,
        )
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(ESI, unsafe { new_voice_in_game() }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(VOICE_COUNT_RVA)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut count = Box::new(0u32);
    let count_at = &mut *count as *mut u32 as u32;
    let base = 0x1000_0000u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40B04C,
            len: 0x22,
            entry: &[0x50, 0xA1, 0x48, 0x83, 0x4F, 0x00],
            calls: &[],
            jumps_out: &[0xB],
        },
        &|_| None,
        &[(0x4F8348, count_at), (0x4F834C, base)],
        3,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    for c in [0u32, 1, 7, 0x100, 0x1FF] {
        *count = c;
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (*count, ro.esi);
        *count = c;
        let mine = unsafe { new_voice(count_at, base) };
        let ok = mine.as_ref().is_ok_and(|&v| (*count, v) == theirs)
            && ro.preserved_except(&Regs::SENTINEL, ESI);
        t.check(ok, || {
            format!("count {c:#x}: original {theirs:x?}; rust {mine:?}")
        });
    }
    t.finish()
}
