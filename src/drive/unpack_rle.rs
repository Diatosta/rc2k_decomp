//! `drive::unpack_rle` (0x41F080): run-length unpacking from esi to edi: a signed count
//! byte; below 0, that many bytes copied; above 0, the next byte repeated count + 1
//! times; 0 ends it.
//!
//! In:  esi the packed bytes, edi where they go
//! Out: ecx the bytes written; eax, esi, edi kept (pushed); no flags read
//!
//! Caller: `drive::read_packed`. A loop runs back to its fourth byte: hooked by a private
//! copy (0x23 bytes, no calls) and a hand-made patch.
//! Refuses (before anything) more than 0x100000 bytes out (the original runs on: a guard
//! against a broken frame).
//! Self-test: the original and the Rust version on test runs (copies, repeats, both,
//! empty) into a test buffer.

use crate::errors::{BadInput, Refused};
use crate::framework::{ECX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("drive::unpack_rle", rva(0x41F080), thunk, &ORIGINAL)
    .private_copy(0x23)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MOST: u32 = 0x10_0000;

/// The bytes it would write (checked before writing).
unsafe fn out_len(mut from: u32) -> Result<u32, Refused> {
    let mut n = 0u32;
    loop {
        let c = unsafe { (from as *const i8).read() } as i32;
        from += 1;
        if c == 0 {
            return Ok(n);
        }
        if c < 0 {
            n += (-c) as u32;
            from += (-c) as u32;
        } else {
            n += c as u32 + 1;
            from += 1;
        }
        if n > MOST {
            return Err(BadInput("a packed run past 0x100000 bytes").into());
        }
    }
}

/// The Rust version: the bytes written.
pub unsafe fn unpack_rle(mut from: u32, to: u32) -> Result<u32, Refused> {
    unsafe { out_len(from) }?;
    let mut at = to;
    loop {
        let c = unsafe { (from as *const i8).read() } as i32;
        from += 1;
        if c == 0 {
            return Ok(at - to);
        }
        if c < 0 {
            let n = (-c) as usize;
            unsafe { std::ptr::copy(from as *const u8, at as *mut u8, n) };
            from += n as u32;
            at += n as u32;
        } else {
            let b = unsafe { (from as *const u8).read() };
            from += 1;
            let n = c as usize + 1;
            unsafe { std::ptr::write_bytes(at as *mut u8, b, n) };
            at += n as u32;
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(ECX, unsafe { unpack_rle(r.esi, r.edi) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let n = unsafe { out_len(r.esi) }.unwrap_or(0) as usize;
    dispatch_state(&HOOK, r, call_original, &[(r.edi, n)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let runs: [&[u8]; 4] = [
        &[0],
        &[0xFD, 1, 2, 3, 4, 0x41, 0],
        &[0x7F, 0xEE, 0x80, 0],
        &[0x02, 0x11, 0xFF, 0x22, 0x01, 0x33, 0xFE, 0x44, 0x55, 0],
    ];
    let mut out = vec![0xAAu8; 0x400];
    let at = out.as_mut_ptr() as u32;
    for (k, run) in runs.iter().enumerate() {
        let mut packed = run.to_vec();
        if k == 2 {
            // 0x80 copies 128 bytes: give them.
            packed.splice(3..3, (0..128).map(|x| x as u8));
        }
        let src = packed.as_ptr() as u32;
        let set = || unsafe { std::ptr::write_bytes(at as *mut u8, 0xAA, 0x400) };
        check_state(
            &mut t,
            &format!("run {k}"),
            call_original,
            Regs {
                esi: src,
                edi: at,
                ..Regs::SENTINEL
            },
            &set,
            &[(at, 0x400)],
            &rust,
        );
    }
    t.finish()
}
