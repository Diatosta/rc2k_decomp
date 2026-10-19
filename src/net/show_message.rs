//! `net::show_message` (0x41745C): message eax shown (`net::message_frame` through
//! `screens::run_frame`), a quarter of a second's sleep, the pending key dropped
//! (`input::take_key`), then frames until a key is pressed.
//!
//! In:  eax = the message's string id (0x154: the connection failed, 0x155: the session)
//! Out: every register kept (`pusha` .. `popa`); the flags of the last `take_key` (no
//!      caller reads them)
//! Caller: `net::start_link`. Its calls through `part` (the frames through the game). It
//! waits for a key: ungated. Shadow mode runs the original alone. Self-test: a copy whose
//! calls go to stand-ins (a key after some frames), against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva};
use crate::net::lobby::{Part, RUN_FRAME, SLEEP, TAKE_KEY, answer, dispatch_lobby, game, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::show_message", rva(0x41745C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const MESSAGE_FRAME: u32 = 0x41748D; // net::message_frame

/// The Rust version: the registers it leaves (the entry's).
pub fn show_message(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.esi = r.eax;
    r.eax = game(MESSAGE_FRAME);
    part(RUN_FRAME, &mut r)?;
    r.eax = 250;
    part(SLEEP, &mut r)?;
    part(TAKE_KEY, &mut r)?;
    loop {
        r.eax = game(MESSAGE_FRAME);
        part(RUN_FRAME, &mut r)?;
        part(TAKE_KEY, &mut r)?;
        if r.eflags & ZERO_FLAG == 0 {
            break;
        }
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, show_message);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov esi, eax; lea eax, sub_41748D
    let copy = match crate::net::lobby::copy(
        0x41745C,
        0x31,
        &[0x60, 0x8B, 0xF0, 0x8D, 0x05, 0x8D, 0x74, 0x41, 0x00],
        &[0x9, 0x13, 0x18, 0x23, 0x28],
        &[RUN_FRAME, SLEEP, TAKE_KEY],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    for frames in [1u32, 4] {
        run_both(
            &mut t,
            &format!("a key after {frames} frames"),
            copy.addr(),
            Regs {
                eax: 0x154,
                ..Regs::SENTINEL
            },
            &[],
            0,
            &[],
            &|| {
                let mut taken = 0;
                answer(Box::new(move |place, r| {
                    if place == TAKE_KEY {
                        let key = if taken > frames { 0x1C } else { 0 };
                        taken += 1;
                        r.eax = r.eax & !0xFF | key;
                        r.set_zero(key == 0);
                    }
                }));
            },
            &show_message,
        );
    }
    t.finish()
}
