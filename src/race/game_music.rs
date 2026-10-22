//! `race::game_music` (0x470F8A): the race's tune: 6 with `dword_75B108` 1, else 5.
//!
//! In:  nothing
//! Out: every register preserved (the tunes preserve them)
//!
//! See `race/music.rs`. One caller, `race::load_stage`. Shadow mode runs the original
//! alone (a command must not be sent twice). Self-test: a redirected copy whose tune
//! calls note themselves and their registers, `dword_75B108` 0, 1, 2.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::stand_in;
use crate::race::music::{check_music, game_music, game_tune};
use crate::redirect::RedirectedCopy;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::game_music", rva(0x470F8A), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = game_music(r, &mut game_tune) {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    game_music(r, &mut game_tune)?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The tunes' callees as IDA shows them, by place.
    let tunes: &[(u32, usize)] = &[(5, 0x40F7B6), (6, 0x40F7C8)];
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x470F8A)),
            0x15,
            &[0x83, 0x3D, 0x08, 0xB1, 0x75, 0x00, 0x01],
            &[0x09, 0x0F],
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
    check_music(&mut t, &copy, &numbers, game_music);
    t.finish()
}
