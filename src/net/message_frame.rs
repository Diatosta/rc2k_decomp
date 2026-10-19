//! `net::message_frame` (0x41748D): the frame of `net::show_message`: the surface locked
//! (`gfx::lock_surface`; failing, nothing), the background copied, the message (string esi)
//! drawn with `font_b` at 0x190, 0x12C, unlocked and shown.
//!
//! In:  esi = the message's string id
//! Out: every register kept (`pusha` .. `popa`)
//! Called by `screens::run_frame` for `net::show_message`. Its calls through `part` (the
//! frame's platform through the game). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose calls go to stand-ins, against the Rust version, the lock failing
//! or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    COPY_PICTURE, DRAW_STRING, LOCK, Part, SHOW_FRAME, UNLOCK, a, answer, dispatch_lobby, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::message_frame", rva(0x41748D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FONT_B: usize = 0x51E844; // font_b

/// A frame with one string: locked, the background, the string, unlocked, shown; nothing if
/// the lock fails. The registers as it leaves them.
pub fn string_frame(
    r: &mut Regs,
    id: u32,
    font: usize,
    x: u32,
    y: u32,
    part: Part,
) -> Result<(), Refused> {
    part(LOCK, r)?;
    if r.carry() {
        return Ok(());
    }
    string_frame_locked(r, id, font, x, y, part)
}

/// `string_frame` after the lock.
pub fn string_frame_locked(
    r: &mut Regs,
    id: u32,
    font: usize,
    x: u32,
    y: u32,
    part: Part,
) -> Result<(), Refused> {
    part(COPY_PICTURE, r)?;
    r.eax = id;
    r.ebx = a(font);
    r.ecx = x;
    r.edx = y;
    part(DRAW_STRING, r)?;
    part(UNLOCK, r)?;
    part(SHOW_FRAME, r)
}

/// The Rust version: the registers it leaves (the entry's).
pub fn message_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    string_frame(&mut r, entry.esi, FONT_B, 0x190, 0x12C, part)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, message_frame);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call gfx::lock_surface
    let copy = match crate::net::lobby::copy(
        0x41748D,
        0x30,
        &[0x60, 0xE8, 0x1A, 0xE4, 0xFE, 0xFF],
        &[0x1, 0x8, 0x1F, 0x24, 0x29],
        &[LOCK, COPY_PICTURE, DRAW_STRING, UNLOCK, SHOW_FRAME],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    for fails in [false, true] {
        run_both(
            &mut t,
            &format!("the lock failing {fails}"),
            copy.addr(),
            Regs {
                esi: 0x154,
                ..Regs::SENTINEL
            },
            &[],
            0,
            &[],
            &|| {
                answer(Box::new(move |place, r| {
                    r.set_carry(place == LOCK && fails)
                }))
            },
            &message_frame,
        );
    }
    t.finish()
}
