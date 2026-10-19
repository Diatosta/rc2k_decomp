//! `system::find_entry_with_bits` (0x4017AB): the first of ecx 8-byte entries from ebx
//! whose first dword has all of eax's bits.
//!
//! In:  eax the bits, ebx the table, ecx the count
//! Out: found: carry clear, ebx the entry, ecx the entries left with it; not found:
//!      carry set, ebx past the table, ecx 0 (a count of 0: ebx as it came); everything
//!      else preserved (edx is pushed and popped)
//! Callers `jb` on the carry: four, the screen modes' tables (`gfx::mark_3d_modes`,
//! 0x4046B7, 0x40471F, 0x404AE6). Refuses a null table with a count (before anything).
//! Manual patch with a private copy (0x17 bytes): the `jb` at +4 is in the first bytes.
//! Shadow mode: both, compared. Self-test: the original against the Rust version on a
//! table of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EBX, ECX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "system::find_entry_with_bits",
    rva(0x4017AB),
    thunk,
    &ORIGINAL,
)
.private_copy(0x17)
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// (found, ebx, ecx).
pub fn find_entry_with_bits(bits: u32, table: u32, count: u32) -> (bool, u32, u32) {
    let (mut at, mut left) = (table, count);
    while left != 0 {
        if unsafe { (at as *const u32).read_unaligned() } & bits == bits {
            return (true, at, left);
        }
        at = at.wrapping_add(8);
        left -= 1;
    }
    (false, at, left)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ebx == 0 && r.ecx != 0 {
        return Err(BadInput("a null table").into());
    }
    let (found, ebx, ecx) = find_entry_with_bits(r.eax, r.ebx, r.ecx);
    let left = Left::nothing().with(EBX, ebx).with(ECX, ecx);
    Ok(Left {
        carry: Some(!found),
        // found: `cmp edx, eax` equal; not: the `dec ecx` that reached 0, or `cmp ecx, 1` on 0
        zero: Some(found || r.ecx != 0),
        ..left
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

/// A table of the tests': 16 entries whose first dwords have mixed bits.
pub fn test_table() -> Vec<u32> {
    (0..16u32)
        .flat_map(|n| [n.wrapping_mul(0x0101_1011) ^ (n << 12), 0xAA00 + n])
        .collect()
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let table = test_table();
    let at = table.as_ptr() as u32;
    for count in [0u32, 1, 5, 16] {
        for bits in [0u32, 1, 0x10, 0x1011, 0x3000, 0xF000_0000, 0x0202_2022] {
            check_state(
                &mut t,
                &format!("count {count}, bits {bits:#x}"),
                call_original,
                Regs {
                    eax: bits,
                    ebx: at,
                    ecx: count,
                    ..Regs::SENTINEL
                },
                &|| {},
                &[],
                &rust,
            );
        }
    }
    drop(table);
    t.finish()
}
