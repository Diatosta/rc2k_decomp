//! `strings::expand_entry` (0x401FE1): appends entry `id` of the packed string table, expanding
//! child entries recursively.
//!
//! In:  eax = id, edi = cursor
//! Out: edi = new cursor
//! The original also clobbers eax, ecx and esi; its callers save those around the call.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::string_table::{blob, entry_count, table};
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::expand_entry", rva(0x401FE1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Entry `id` of the string table `g_stringTable` points at, given to `emit` (converted:
/// `rc2k_core::strings::table::StringTable::expand`). `blob` is that pointer, as the callers read
/// it. A bad entry or a table the core does not know is logged once and gives nothing (the
/// original read wherever it pointed: fixed, `notes/bugs.md`).
pub unsafe fn expand_entry(blob: *const u8, id: u32, emit: &mut impl FnMut(*const u8, usize)) {
    let mut out = Vec::new();
    let done = match table() {
        Some(t) if t.bytes.as_ptr() == blob => t.expand(id, &mut out).map_err(|e| e.0),
        _ => Err("the string table is not the lexicon's"),
    };
    match done {
        Ok(()) => emit(out.as_ptr(), out.len()),
        Err(why) => HOOK.mismatch(|| format!("id={id}: {why}; nothing written")),
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (id, cursor) = (r.eax, r.edi as *mut u8);
    let table = blob();

    if !shadow() {
        let mut out = cursor;
        unsafe {
            expand_entry(table, id, &mut |p, n| {
                copy_nonoverlapping(p, out, n);
                out = out.add(n);
            });
        }
        r.edi = out as u32;
        return;
    }

    // Shadow: Rust collects into a Vec, the original does the real writing.
    let mut mine = Vec::new();
    unsafe {
        expand_entry(table, id, &mut |p, n| {
            mine.extend_from_slice(from_raw_parts(p, n))
        })
    };

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.edi as usize).wrapping_sub(cursor as usize);

    if orig_len != mine.len() || unsafe { from_raw_parts(cursor, orig_len) } != &mine[..] {
        HOOK.mismatch(|| {
            format!(
                "id={id}: original wrote {orig_len} bytes, rust {}",
                mine.len()
            )
        });
    }
    r.edi = o.edi;
}

// ---- self-test: every entry of the real table ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let table = blob();
    if table.is_null() {
        return t.skip("string table not loaded");
    }

    // Unhooked, so the original's own recursion runs original code too.
    HOOK.run_unhooked(|| {
        for id in 0..entry_count() as u32 {
            let mut mine = Vec::new();
            unsafe {
                expand_entry(table, id, &mut |p, n| {
                    mine.extend_from_slice(from_raw_parts(p, n))
                })
            };

            // Slack, so a wrong Rust length can't make the original overflow the buffer.
            let mut buf = vec![0xAAu8; mine.len() + 4096];
            let mut ro = Regs {
                eax: id,
                edi: buf.as_mut_ptr() as u32,
                ..Regs::default()
            };
            unsafe { call_original(&mut ro) };
            let orig_len = (ro.edi as usize).wrapping_sub(buf.as_ptr() as usize);

            t.check(buf.get(..orig_len) == Some(&mine[..]), || {
                format!(
                    "id={id} original {} rust {}",
                    bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                    bytes_str(&mine)
                )
            });
        }
    });
    t.finish()
}
