//! `net::stopping` (0x417089): with the session on (`g_netSession` 1), the background
//! (`net::show_background`) and "stopping" shown (string 0x157, as `net::connecting` shows
//! its); then the network stopped (`net::stop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::start_link`. Its calls through `part` (the frames through the game). Shadow
//! mode runs the original alone. Self-test: a copy whose calls go to stand-ins, against the
//! Rust version, the session on or off, the lock failing or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::connecting::status;
use crate::net::lobby::{
    COPY_PICTURE, DRAW_STRING, LOCK, Part, RUN_FRAME, SESSION, SHOW_FRAME, SLEEP, STOP, Saved,
    UNLOCK, answer, dispatch_lobby, dword, g, p, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::stopping", rva(0x417089), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const SHOW_BACKGROUND: u32 = 0x4173E7; // net::show_background

/// The Rust version: the registers it leaves (the entry's).
pub fn stopping(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if g(SESSION) == 1 {
        part(SHOW_BACKGROUND, &mut r)?;
        status(&mut r, 0x157, part)?;
    }
    part(STOP, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, stopping);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        SHOW_BACKGROUND,
        RUN_FRAME,
        LOCK,
        COPY_PICTURE,
        DRAW_STRING,
        UNLOCK,
        SHOW_FRAME,
        SLEEP,
        STOP,
    ];
    // pusha; cmp g_netSession, 1
    let copy = match crate::net::lobby::copy(
        0x417089,
        0x57,
        &[0x60, 0x83, 0x3D, 0x10, 0xA9, 0x52, 0x00, 0x01],
        &[0xA, 0x11, 0x16, 0x1D, 0x37, 0x3C, 0x41, 0x4B, 0x50],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [dword(SESSION)];
    let saved = Saved::new(&all);
    for (session, fails) in [(0u32, false), (1, false), (1, true), (2, false)] {
        run_both(
            &mut t,
            &format!("session {session}, the lock failing {fails}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &[],
            &|| {
                p(SESSION, session);
                answer(Box::new(move |place, r| {
                    r.set_carry(place == LOCK && fails)
                }));
            },
            &stopping,
        );
    }
    drop(saved);
    t.finish()
}
