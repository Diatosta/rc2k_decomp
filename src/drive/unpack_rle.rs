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

use crate::bridge::Raw;
use crate::errors::Refused;
use crate::framework::{ECX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use rc2k_core::drive::rle;
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

/// The bytes it would write (checked before writing).
unsafe fn out_len(from: u32) -> Result<u32, Refused> {
    Ok(rle::unpacked_len(&unsafe { Raw::new(from as *const u8) })?)
}

/// The Rust version: the bytes written.
// Converted: `rc2k_core::drive::rle::unpack_rle`.
pub unsafe fn unpack_rle(from: u32, to: u32) -> Result<u32, Refused> {
    let packed = unsafe { Raw::new(from as *const u8) };
    Ok(rle::unpack_rle(&packed, |k, b| unsafe {
        ((to + k) as *mut u8).write(b)
    })?)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(ECX, unsafe { unpack_rle(r.esi, r.edi) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let n = unsafe { out_len(r.esi) }.unwrap_or(0) as usize;
    dispatch_state(&HOOK, r, call_original, &[(r.edi, n)], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
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
