//! `strings::append_string_by_id` (0x401FA3): appends the text for a string id.
//!
//! In:  eax = id, ebx = cursor
//! Out: ebx = new cursor; eax = the string pointer (pointer-table branches) or the id itself
//!      (string-table branch)
//! Part of the original lives in a separate chunk at 0x402010.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::expand_entry::expand_entry;
use crate::strings::string_table::{LOW_IDS, blob, entry_count, ptr_high, ptr_low};
use crate::strings::utils::bytes_str;
use rc2k_core::strings::table::Source;
use std::ffi::{CStr, c_char};
use std::ops::Range;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "strings::append_string_by_id",
    rva(0x401FA3),
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

/// `emit` receives each run of bytes to append. Returns the `eax` the original leaves behind.
pub unsafe fn append_string_by_id(id: u32, emit: &mut impl FnMut(*const u8, usize)) -> u32 {
    unsafe {
        // Converted: `rc2k_core::strings::table::Source`.
        let direct = match Source::of(id, || !ptr_low(id).is_null()) {
            Source::Made => Some(ptr_high(id)),
            Source::Pointer => Some(ptr_low(id)),
            Source::Table => None,
        };
        match direct {
            Some(p) => {
                let len = CStr::from_ptr(p as *const c_char).to_bytes().len();
                emit(p, len);
                p as usize as u32
            }
            None => {
                expand_entry(blob(), id, emit);
                id
            }
        }
    }
}

/// Appends the text of string `id` to `out`.
pub unsafe fn push_string(out: &mut Vec<u8>, id: u32) {
    unsafe { append_string_by_id(id, &mut |p, n| out.extend_from_slice(from_raw_parts(p, n))) };
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (id, cursor) = (r.eax, r.ebx as *mut u8);

    if !shadow() {
        let mut out = cursor;
        let eax = unsafe {
            append_string_by_id(id, &mut |p, n| {
                copy_nonoverlapping(p, out, n);
                out = out.add(n);
            })
        };
        r.ebx = out as u32;
        r.eax = eax;
        return;
    }

    // Shadow: Rust collects into a Vec, the original does the real writing.
    let mut mine = Vec::new();
    let mine_eax = unsafe {
        append_string_by_id(id, &mut |p, n| mine.extend_from_slice(from_raw_parts(p, n)))
    };

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.ebx as usize).wrapping_sub(cursor as usize);

    if orig_len != mine.len()
        || unsafe { from_raw_parts(cursor, orig_len) } != &mine[..]
        || o.eax != mine_eax
    {
        HOOK.mismatch(|| {
            format!(
                "id={id:#x}: original wrote {orig_len} bytes (eax={:#x}), rust {} (eax={mine_eax:#x})",
                o.eax,
                mine.len()
            )
        });
    }
    r.ebx = o.ebx;
    r.eax = o.eax;
}

// ---- self-test: ids from the real tables ----

/// Ids to test in the 0x1000 branch. Fill this in once the length of dword_525DE8 is known
/// (check the array in IDA), e.g. 0x1000..0x1100. Empty = that branch isn't covered here.
const HIGH_IDS: Range<u32> = 0x1000..0x1000;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if blob().is_null() {
        return t.skip("string table not loaded");
    }
    let entries = entry_count() as u32;
    // Low range: ids whose src[] pointer is set (direct branch), or that fall through
    // to expand_entry and are valid entries.
    let ids =
        HIGH_IDS.chain(LOW_IDS.filter(|&id| unsafe { !ptr_low(id).is_null() } || id < entries));

    for id in ids {
        let mut mine = Vec::new();
        let mine_eax = unsafe {
            append_string_by_id(id, &mut |p, n| mine.extend_from_slice(from_raw_parts(p, n)))
        };

        let mut buf = vec![0xAAu8; mine.len() + 4096];
        let mut ro = Regs {
            eax: id,
            ebx: buf.as_mut_ptr() as u32,
            ..Regs::default()
        };
        unsafe { call_original(&mut ro) };
        let orig_len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);

        t.check(
            buf.get(..orig_len) == Some(&mine[..]) && ro.eax == mine_eax,
            || {
                format!(
                    "id={id:#x} original {} rust {}",
                    bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                    bytes_str(&mine)
                )
            },
        );
    }
    t.finish()
}
