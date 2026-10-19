//! `strings::rot13_into_text` (0x4022C4): a text coded by rot13 (letters 13 on, `A` .. `Z`
//! and `a` .. `z` each in their own round; other bytes as they are) into the shared text
//! buffer (`text`, 0x93E430), with its NUL.
//!
//! In:  eax = the text
//! Out: eax = the text buffer; everything else kept (`pusha` .. `popa`; the original's
//!      `mov ah, 'z'` in the loop is lost in the `popa`)
//! Caller: `menu::check_cheat_name` (the fourth player's name, against the coded cheat
//! names). Compared in shadow mode. Refuses (before anything) a null text and one longer
//! than the buffer. Self-test: the original and the Rust version on texts of the test's
//! (every byte value, the round's edges), the buffer filled with 0xAA first and put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::rot13_into_text", rva(0x4022C4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// One byte coded.
pub fn rot13(b: u8) -> u8 {
    match b {
        b'A'..=b'Z' => (b - b'A' + 13) % 26 + b'A',
        b'a'..=b'z' => (b - b'a' + 13) % 26 + b'a',
        _ => b,
    }
}

/// The Rust version: the text buffer's address.
pub unsafe fn rot13_into_text(from: *const u8) -> Result<u32, BadInput> {
    if from.is_null() {
        return Err(BadInput("a null text"));
    }
    let mut coded = Vec::new();
    unsafe {
        loop {
            let b = from.add(coded.len()).read();
            coded.push(rot13(b));
            if b == 0 {
                break;
            }
            if coded.len() >= TEXT_LEN {
                return Err(BadInput("a text longer than the text buffer"));
            }
        }
        std::ptr::copy_nonoverlapping(coded.as_ptr(), text(), coded.len());
    }
    Ok(text() as u32)
}

fn region() -> Region {
    (text() as u32, TEXT_LEN)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let at = unsafe { rot13_into_text(r.eax as *const u8) }?;
    Ok(Left::nothing().with(EAX, at))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[region()], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = capture(&[region()]);
    let every: Vec<u8> = (1..=255u8).chain([0]).collect();
    let texts: [&[u8]; 5] = [
        b"\0",
        b"arcade action\0",
        b"AZaz@[`{MNmn\0",
        b"Nepnqr Npgvba 123!\0",
        &every,
    ];
    for (n, &text_bytes) in texts.iter().enumerate() {
        let source = text_bytes.to_vec();
        check_state(
            &mut t,
            &format!("text {n}"),
            call_original,
            Regs {
                eax: source.as_ptr() as u32,
                ..Regs::SENTINEL
            },
            &|| unsafe { std::ptr::write_bytes(text(), 0xAA, TEXT_LEN) },
            &[region()],
            &rust,
        );
    }
    restore(&[region()], &saved);
    t.finish()
}
