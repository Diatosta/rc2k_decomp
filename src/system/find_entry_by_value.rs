//! `system::find_entry_by_value` (0x4017D3): the first of ecx 8-byte entries from ebx
//! whose second dword is eax.
//!
//! In:  eax the value, ebx the table, ecx the count
//! Out: found: carry clear, ebx the entry, ecx the entries left with it; not found:
//!      carry set, ebx past the table, ecx 0 (a count of 0: ebx as it came); everything
//!      else preserved
//! Callers `jb` on the carry: twelve, the DirectDraw / DirectInput set-up's tables
//! (0x404xxx, 0x405C40, 0x40B31F, 0x41A6xx, 0x41B3xx). Refuses a null table with a
//! count (before anything). Shadow mode: both, compared.
//! Self-test: the original against the Rust version on a table of the test's.

use crate::errors::Refused;
use crate::framework::{EBX, ECX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "system::find_entry_by_value",
    rva(0x4017D3),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// (found, ebx, ecx).
pub fn find_entry_by_value(value: u32, table: u32, count: u32) -> (bool, u32, u32) {
    // Converted: `rc2k_core::system::tables::by_value`.
    use rc2k_core::system::tables::{by_value, left};
    let found = by_value(
        unsafe { crate::system::find_entry::entries(table, count) },
        value,
    );
    let (at, n) = left(table, count, found);
    (found.is_some(), at, n)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ebx == 0 && r.ecx != 0 {
        return Err(crate::errors::BadInput("a null table").into());
    }
    let (found, ebx, ecx) = find_entry_by_value(r.eax, r.ebx, r.ecx);
    let left = Left::nothing().with(EBX, ebx).with(ECX, ecx);
    Ok(Left {
        carry: Some(!found),
        // found: `cmp eax, [ebx+4]` equal; not: the `dec ecx` that reached 0, or `cmp ecx, 1` on 0
        zero: Some(found || r.ecx != 0),
        ..left
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let table: Vec<u32> = (0..16u32).flat_map(|n| [0xAA00 + n, 0x10 * n]).collect();
    let at = table.as_ptr() as u32;
    for count in [0u32, 1, 5, 16] {
        for value in [0u32, 0x10, 0x40, 0xF0, 0x41] {
            check_state(
                &mut t,
                &format!("count {count}, value {value:#x}"),
                call_original,
                Regs {
                    eax: value,
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
