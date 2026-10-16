//! `race::game_music_8` (0x470378): tune 8 (`sound::play_tune_8`; a `call` and a `retn`).
//!
//! In:  nothing
//! Out: every register preserved (the tunes preserve them)
//!
//! See `race/music.rs`. One caller, `race::load_stage`. Shadow mode runs the original
//! alone (a command must not be sent twice). Self-test: a redirected copy whose tune
//! calls note themselves and their registers, `dword_75B108` 0, 1, 2.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::stand_in;
use crate::race::music::{check_music, game_music_8, game_tune};
use crate::redirect::RedirectedCopy;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::game_music_8", rva(0x470378), thunk, &ORIGINAL)
    .private_copy(0x6)
    .relocate_calls(&[0x00])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = game_music_8(r, &mut game_tune) {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The tunes' callees as IDA shows them, by place.
    let tunes: &[(u32, usize)] = &[(8, 0x40F7EC)];
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x470378)),
            0x6,
            &[0xE8, 0x6F, 0xF4, 0xF9, 0xFF],
            &[0x00],
            &|c| {
                tunes
                    .iter()
                    .position(|&(_, k)| rva(k) == c)
                    .map(|n| stand_in(n as u32))
            },
        )
    };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let numbers: Vec<u32> = tunes.iter().map(|&(n, _)| n).collect();
    check_music(&mut t, &copy, &numbers, game_music_8);
    t.finish()
}
