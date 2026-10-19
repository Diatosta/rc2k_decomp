//! `system::next_entry` (0x4017E5): one step on in a table of 8-byte entries.
//!
//! In:  ebx the entry, ecx the entries left with it
//! Out: ecx 0: carry set, nothing changed. Else ebx + 8 and ecx - 1; carry set if that
//!      leaves 0, else the carry of the `add ebx, 8` (`dec` keeps it: clear unless ebx
//!      wrapped). Everything else preserved.
//! Callers: `gfx::mark_3d_modes` (which does not read the carry) and 0x404718. Manual patch
//! with a private copy (0xD bytes): the `jb` at +3 is in the first bytes. Shadow mode:
//! both, compared. Self-test: the original against the Rust version on counts and places
//! (one that wraps).

use crate::errors::Refused;
use crate::framework::{EBX, ECX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("system::next_entry", rva(0x4017E5), thunk, &ORIGINAL)
    .private_copy(0xD)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// (carry, ebx, ecx).
pub fn next_entry(entry: u32, left: u32) -> (bool, u32, u32) {
    if left == 0 {
        return (true, entry, left);
    }
    let (next, wrapped) = entry.overflowing_add(8);
    let left = left - 1;
    (left == 0 || wrapped, next, left)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (carry, ebx, ecx) = next_entry(r.ebx, r.ecx);
    let left = Left::nothing().with(EBX, ebx).with(ECX, ecx);
    Ok(Left {
        carry: Some(carry),
        // a next one: `dec ecx` not zero; none: the `dec ecx` that reached 0, or `cmp ecx, 1` on 0
        zero: Some(carry && r.ecx != 0),
        ..left
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for count in [0u32, 1, 2, 5, 0xFFFF_FFFF] {
        for entry in [0x1000u32, 0xFFFF_FFF8, 0xFFFF_FFFC, 0x7FFF_FFFC] {
            check_state(
                &mut t,
                &format!("count {count:#x}, entry {entry:#x}"),
                call_original,
                Regs {
                    ebx: entry,
                    ecx: count,
                    ..Regs::SENTINEL
                },
                &|| {},
                &[],
                &rust,
            );
        }
    }
    t.finish()
}
