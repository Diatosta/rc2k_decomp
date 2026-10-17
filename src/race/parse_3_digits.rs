//! `race::parse_3_digits` (0x4379EB): the three decimal digits at `byte_6047B8` + ecx (the
//! cars' data file's text) as a number.
//!
//! In:  ecx = the place in the text
//! Out: carry clear, eax = the number; carry set, eax = 0 with any not a digit (`movzx`
//!      and `sub` of the low byte: a byte below '0' wraps within the byte); the rest
//!      preserved (ebx pushed and popped)
//!
//! See `race/race_cars.rs`. One caller, `sub_437971` (the cars' data). The failure path
//! (0x437A4B) is `parse_2_digits`' own end, which `parse_3_digits` jumps to. Shadow
//! mode: the Rust version and the original, registers and carry compared (nothing
//! written). Self-test: texts of the test's in the game's buffer (put back), every place
//! of a run of digits and letters, the bytes round '0' and '9', the last places.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::race_cars::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::parse_3_digits", rva(0x4379EB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(match unsafe { parse_3_digits(r.ecx) } {
        Some(n) => Left::carry(false).with(EAX, n),
        None => Left::carry(true).with(EAX, 0),
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let text = (game_addr(TEXT_RVA) as u32, TEXT_LEN);
    let saved = capture(&[text]);
    let mut bytes = vec![b' '; TEXT_LEN];
    let sample = b"L2104512399/0:09\x2F\x3A\x00\xFF\x30\x39\x39x987";
    bytes[..sample.len()].copy_from_slice(sample);
    bytes[TEXT_LEN - 3..].copy_from_slice(b"123");
    let places: Vec<u32> = (0..sample.len() as u32)
        .chain([TEXT_LEN as u32 - 3, TEXT_LEN as u32 - 4])
        .collect();
    for at in places {
        check_state(
            &mut t,
            &format!("at {at:#x}"),
            call_original,
            Regs {
                ecx: at,
                ..Regs::SENTINEL
            },
            &|| restore(&[text], &bytes),
            &[],
            &rust,
        );
    }
    restore(&[text], &saved);
    t.finish()
}
