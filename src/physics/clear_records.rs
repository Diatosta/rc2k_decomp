//! `physics::clear_records` (0x43F3A6): ecx records of 0x20 bytes from edi, the first six
//! dwords of each 0 (`dec ecx; jnz` back to its own first instruction).
//!
//! In:  edi the first record, ecx their number
//! Out: edi past the last, ecx 0; the rest kept; ZF set (`dec`), not read (its callers
//!      return next, theirs go on with calls)
//!
//! Its loop goes back to its first byte: `.private_copy` (the whole function).
//! Callers: `physics::clear_forces` (0x614770, 0x17), `physics::clear_contacts`
//! (0x614A50, 7).
//! Refuses (before anything) ecx 0 (the original would loop 2^32 times) or above 0x100.
//! Self-test: a copy of the original on a test buffer, counts 1 .. 0x20.

use crate::errors::{BadInput, Refused};
use crate::framework::{ECX, EDI, RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("physics::clear_records", rva(0x43F3A6), thunk, &ORIGINAL)
    .private_copy(0x30)
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn clear_records(at: u32, count: u32) -> Result<Left, Refused> {
    if count == 0 || count > 0x100 {
        return Err(BadInput("a record count of 0 or above 0x100").into());
    }
    for k in 0..count {
        for d in 0..6 {
            unsafe { ((at + 0x20 * k + 4 * d) as *mut u32).write_unaligned(0) };
        }
    }
    Ok(Left::nothing().with(EDI, at + 0x20 * count).with(ECX, 0))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { clear_records(r.edi, r.ecx) }
}

extern "C" fn dispatch(r: &mut Regs) {
    let len = 0x20 * r.ecx.clamp(1, 0x100) as usize;
    dispatch_state(&HOOK, r, call_original, &[(r.edi, len)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43F3A6,
            len: 0x30,
            entry: &[0xC7, 0x07, 0x00, 0x00, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut buffer = vec![0u32; 0x20 * 8 + 8];
    let b = buffer.as_mut_ptr() as u32;
    let all = [(b, buffer.len() * 4)];
    for count in [1u32, 2, 7, 0x17, 0x20] {
        let fill = || {
            for (k, v) in unsafe { std::slice::from_raw_parts_mut(b as *mut u32, 0x20 * 8 + 8) }
                .iter_mut()
                .enumerate()
            {
                *v = 0xA5A5_0000 | k as u32;
            }
        };
        fill();
        let before = Regs {
            edi: b,
            ecx: count,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        fill();
        let mine = unsafe { clear_records(b, count) };
        let state = capture(&all);
        let ok = mine.as_ref().is_ok_and(|l| l.matches(&before, &ro)) && theirs == state;
        t.check(ok, || {
            format!("count {count:#x}: {mine:?}, the buffer differs")
        });
    }
    let refused = unsafe { clear_records(b, 0) }.is_err();
    t.check(refused, || "count 0 not refused".into());
    drop((copy, buffer));
    t.finish()
}
