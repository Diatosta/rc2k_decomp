//! `memory::take_from_list` (0x402956): hands out the next bytes of a list's newest block.
//!
//! In:  eax = bytes, ebx = the list (`MemoryList`, after `memory::rewind_list`)
//! Out: eax = their address, carry clear; or eax = 0, carry set, if the list has fewer bytes
//!      left; everything else preserved
//! The list's cursor moves on and what remains is counted down. 12 call sites: the lexicon
//! compiler and the sprite and font builder cut their work memory this way.
//! Shadow mode runs the original alone (the cursor cannot move twice) and checks what it
//! left. Manual patch with a private copy (0x15 bytes, nothing to relocate): a conditional
//! jump sits in the first bytes. Refuses a null list. The name is a reading.

use crate::errors::BadInput;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::memory::MemoryList;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("memory::take_from_list", rva(0x402956), thunk, &ORIGINAL)
    .private_copy(0x15)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The address of the bytes, or None if the list has not that many left.
pub unsafe fn take_from_list(list: *mut MemoryList, bytes: u32) -> Result<Option<u32>, BadInput> {
    if list.is_null() {
        return Err(BadInput("no list to take from"));
    }
    // Converted: `rc2k_core::memory::List::take`.
    Ok(unsafe { (*list).take(bytes) })
}

/// `memory::rewind_list` (0x402941): the cursor back to the newest block's start, and all
/// of the list's bytes left.
pub unsafe fn rewind_list(list: *mut MemoryList) {
    // Converted: `rc2k_core::memory::List::rewind`.
    unsafe { (*list).rewind() }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (list, bytes) = (r.ebx as *mut MemoryList, r.eax);
    if !shadow() {
        match unsafe { take_from_list(list, bytes) } {
            Ok(taken) => {
                r.eax = taken.unwrap_or(0);
                r.set_carry(taken.is_none());
            }
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }
    let before = (!list.is_null()).then(|| unsafe { list.read_unaligned() });
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let as_rust = before.is_none_or(|before| {
        let after = unsafe { list.read_unaligned() };
        if bytes > before.remaining {
            o.carry() && o.eax == 0 && after == before
        } else {
            !o.carry()
                && o.eax == before.cursor
                && after.cursor == before.cursor.wrapping_add(bytes)
                && after.remaining == before.remaining - bytes
        }
    });
    if !as_rust || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "bytes={bytes:#x} list before {before:x?}: original returned {:#x} carry={}",
                o.eax,
                o.carry()
            )
        });
    }
    r.eax = o.eax;
    r.eflags = o.eflags;
}

// ---- self-test: lists of the test's own (no memory behind them: nothing is written) ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for (remaining, bytes) in [
        (0x100u32, 0u32),
        (0x100, 1),
        (0x100, 0x100),
        (0x100, 0x101),
        (0, 0),
        (0, 4),
        (0xFFFF_FFFF, 0x8000_0000),
    ] {
        let start = MemoryList {
            head: 0x1000,
            total: 0x5000,
            cursor: 0x2000,
            remaining,
        };
        let (mut theirs, mut ours) = (start, start);
        let before = Regs {
            eax: bytes,
            ebx: &mut theirs as *mut MemoryList as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let mine = unsafe { take_from_list(&mut ours, bytes) };
        let preserved = ro.preserved_except(&before, EAX);
        let ok = mine == Ok((!ro.carry()).then_some(ro.eax))
            && (!ro.carry() || ro.eax == 0)
            && theirs == ours
            && preserved;
        t.check(ok, || {
            format!(
                "remaining={remaining:#x} bytes={bytes:#x}: original eax={:#x} carry={} left {theirs:x?} preserved={preserved}; rust {mine:x?} left {ours:x?}",
                ro.eax,
                ro.carry()
            )
        });
    }
    t.finish()
}
