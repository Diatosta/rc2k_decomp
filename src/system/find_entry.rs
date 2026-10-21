//! `system::find_entry` (0x40179A): looks a key up in a table of 8-byte entries.
//!
//! In:  eax = the key, ebx = the table, ecx = the number of entries
//! Out: found: ebx = the entry whose first dword is the key, ecx = the entries from it to
//!      the end, carry clear, zero set. Not found: ebx = the table's end, ecx = 0, carry
//!      and zero set. With no entries: nothing changes, carry set, zero clear.
//!      Everything else preserved.
//! 13 call sites (the video modes, the picture's surface, two more).
//! Manual patch with a private copy (0x11 bytes, nothing to relocate): a conditional jump
//! sits in the first bytes.
//! Refuses a null table with entries. The name is a reading.

use crate::errors::BadInput;
use crate::framework::{EBX, ECX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("system::find_entry", rva(0x40179A), thunk, &ORIGINAL)
    .private_copy(0x11)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const ENTRY_LEN: u32 = 8;

// ---- Rust version ----

/// The entry's address, if the key is in the table.
pub unsafe fn find_entry(key: u32, table: u32, count: u32) -> Result<Option<u32>, BadInput> {
    // Converted: `rc2k_core::system::tables::by_key`.
    if count != 0 && table == 0 {
        return Err(BadInput("no table to look in"));
    }
    let found = rc2k_core::system::tables::by_key(unsafe { entries(table, count) }, key);
    Ok(found.map(|n| table.wrapping_add(n as u32 * ENTRY_LEN)))
}

/// The `count` entries from `table`, read as they are asked for.
///
/// # Safety
/// Each entry asked for must be readable.
pub unsafe fn entries(
    table: u32,
    count: u32,
) -> impl Iterator<Item = rc2k_core::system::tables::Entry> {
    (0..count).map(move |n| unsafe {
        ((table.wrapping_add(8 * n)) as *const rc2k_core::system::tables::Entry).read_unaligned()
    })
}

// ---- hook ----

/// (ebx, ecx, carry, zero) for a result.
fn left(r: &Regs, found: Option<u32>) -> (u32, u32, bool, bool) {
    match found {
        Some(entry) => (
            entry,
            r.ecx - entry.wrapping_sub(r.ebx) / ENTRY_LEN,
            false,
            true,
        ),
        None if r.ecx == 0 => (r.ebx, 0, true, false),
        None => (
            r.ebx.wrapping_add(r.ecx.wrapping_mul(ENTRY_LEN)),
            0,
            true,
            true,
        ),
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let found = match unsafe { find_entry(r.eax, r.ebx, r.ecx) } {
        Ok(found) => found,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    let mine = left(r, found);
    if !shadow() {
        (r.ebx, r.ecx) = (mine.0, mine.1);
        r.set_carry(mine.2);
        r.set_zero(mine.3);
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = (o.ebx, o.ecx, o.carry(), o.eflags & 0x40 != 0);
    if theirs != mine || !o.preserved_except(r, EBX | ECX) {
        HOOK.mismatch(|| {
            format!(
                "key={:#x} table={:#x} entries={}: original {theirs:x?} rust {mine:x?}",
                r.eax, r.ebx, r.ecx
            )
        });
    }
    (r.ebx, r.ecx, r.eflags) = (o.ebx, o.ecx, o.eflags);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let table: [u32; 12] = [1, 100, 2, 200, 4, 300, 4, 400, 0, 500, 0xFFFF_FFFF, 600];
    for entries in [0u32, 1, 3, 6] {
        for key in [1u32, 2, 4, 0, 0xFFFF_FFFF, 7, 100] {
            let before = Regs {
                eax: key,
                ebx: table.as_ptr() as u32,
                ecx: entries,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let theirs = (ro.ebx, ro.ecx, ro.carry(), ro.eflags & 0x40 != 0);
            let mine =
                unsafe { find_entry(key, before.ebx, entries) }.map(|found| left(&before, found));
            let preserved = ro.preserved_except(&before, EBX | ECX);
            t.check(mine == Ok(theirs) && preserved, || {
                format!(
                    "key={key:#x} entries={entries}: original {theirs:x?} preserved={preserved}; rust {mine:x?}"
                )
            });
        }
    }
    t.finish()
}
