//! `settings::parse_number` (0x411623): reads a number out of a settings text.
//!
//! In:  eax = where it starts (0 stays 0)
//! Out: eax = the number; everything else preserved
//! An optional `+` or `-`; then hexadecimal after `$` or `0x` (digits, A .. F, a .. f),
//! decimal otherwise; up to the first character that is no digit. No overflow check: the
//! number wraps. Eight callers. Manual patch with a private copy (0x73 bytes): a short
//! jump in the first bytes.

use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("settings::parse_number", rva(0x411623), thunk, &ORIGINAL)
    .private_copy(0x73)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn parse_number(text: *const u8) -> u32 {
    if text.is_null() {
        return 0;
    }
    unsafe {
        let mut at = text;
        let negative = at.read() == b'-';
        if negative || at.read() == b'+' {
            at = at.add(1);
        }
        let mut number = 0u32;
        let hex = if at.read() == b'$' {
            Some(1)
        } else if at.read() == b'0' && at.add(1).read() == b'x' {
            Some(2)
        } else {
            None
        };
        match hex {
            None => loop {
                let digit = at.read().wrapping_sub(b'0');
                at = at.add(1);
                if digit >= 10 {
                    break;
                }
                number = number.wrapping_mul(10).wrapping_add(digit as u32);
            },
            Some(prefix) => {
                at = at.add(prefix);
                loop {
                    let digit = match at.read() {
                        c @ b'0'..=b'9' => c - b'0',
                        c @ b'A'..=b'F' => c - b'A' + 10,
                        c @ b'a'..=b'f' => c - b'a' + 10,
                        _ => break,
                    };
                    at = at.add(1);
                    number = number << 4 | digit as u32;
                }
            }
        }
        if negative {
            number.wrapping_neg()
        } else {
            number
        }
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = unsafe { parse_number(r.eax as *const u8) };
    if !shadow() {
        r.eax = mine;
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.eax != mine || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "text={:?}: original {:#x} rust {mine:#x}",
                unsafe { std::ffi::CStr::from_ptr(r.eax as *const _) },
                o.eax
            )
        });
    }
    r.eax = o.eax;
    r.eflags = o.eflags;
}

// ---- self-test ----

pub const TEST_NUMBERS: [&[u8]; 30] = [
    b"\0",
    b"0\0",
    b"7\0",
    b"1234567890\0",
    b"4294967295\0",
    b"4294967296\0",
    b"99999999999\0",
    b"-1\0",
    b"+15\0",
    b"-\0",
    b"+\0",
    b"--3\0",
    b"12 34\0",
    b"12\r\n\0",
    b"$ff\0",
    b"$FF\0",
    b"$\0",
    b"0x1F\0",
    b"0X1F\0",
    b"0x\0",
    b"-0x10\0",
    b"-$10\0",
    b"$123456789\0",
    b"$@G`g/:\0",
    b"$9:\0",
    b"$fg\0",
    b"0x7fffFFFF\0",
    b"abc\0",
    b" 5\0",
    b"$-5\0",
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for text in TEST_NUMBERS
        .iter()
        .map(|text| text.as_ptr())
        .chain([std::ptr::null()])
    {
        let before = Regs {
            eax: text as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = unsafe { parse_number(text) };
        let preserved = ro.preserved_except(&before, EAX);
        t.check(mine == ro.eax && preserved, || {
            format!(
                "text={:?}: original {:#x} preserved={preserved}; rust {mine:#x}",
                (!text.is_null()).then(|| unsafe { std::ffi::CStr::from_ptr(text as *const _) }),
                ro.eax
            )
        });
    }
    t.finish()
}
