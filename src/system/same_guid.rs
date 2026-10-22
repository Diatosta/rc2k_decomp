//! `system::same_guid` (0x4016F9): whether two 16-byte GUIDs are the same.
//!
//! In:  eax, ebx = the two GUIDs
//! Out: zero set if they are the same (the flags of the last `cmp`: the first differing
//!      dword's, else the fourth's); every register preserved (ecx pushed and popped)
//! Compared as four dwords, the first difference ending it. Callers: the network's
//! (`net::find_saved_connection`, `net::choose_session`, `net::connection_found`,
//! `net::build_address`) and two of the input's (0x4083F4, 0x40953B). Shadow mode: both
//! run, the zero flag compared. Self-test: against the original on GUIDs that differ in
//! each dword and none.

use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("system::same_guid", rva(0x4016F9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: whether the GUIDs at the two addresses are the same.
pub fn same_guid(a: u32, b: u32) -> bool {
    // Converted: `rc2k_core::system::machine::same_guid`.
    let at = |base: u32| move |k: u32| unsafe { ((base + 4 * k) as *const u32).read_unaligned() };
    rc2k_core::system::machine::same_guid(at(a), at(b))
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = same_guid(r.eax, r.ebx);
    if !shadow() {
        r.set_zero(mine);
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = o.eflags & ZERO_FLAG != 0;
    if theirs != mine || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "eax={:#x} ebx={:#x}: original {theirs}, rust {mine}",
                r.eax, r.ebx
            )
        });
    }
    r.eflags = o.eflags;
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mine = same_guid(r.eax, r.ebx);
    r.set_zero(mine);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let first: [u32; 4] = [0x1111_1111, 0x2222_2222, 0x3333_3333, 0x4444_4444];
    for differs in [None, Some(0usize), Some(1), Some(2), Some(3)] {
        let mut second = first;
        if let Some(k) = differs {
            second[k] ^= 0x100;
        }
        let before = Regs {
            eax: first.as_ptr() as u32,
            ebx: second.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut o = before;
        unsafe { call_original(&mut o) };
        let theirs = o.eflags & ZERO_FLAG != 0;
        let mine = same_guid(before.eax, before.ebx);
        t.check(theirs == mine && o.preserved_except(&before, 0), || {
            format!("differing dword {differs:?}: original {theirs}, rust {mine}")
        });
    }
    t.finish()
}
