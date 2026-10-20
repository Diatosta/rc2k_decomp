//! `strings::text_from_ids` (0x40229B): builds the text of `count` consecutive string ids in
//! the shared text buffer and NUL-terminates it.
//!
//! In:  eax = first id, ecx = count (at least 1)
//! Out: ebx = the text buffer; everything else preserved
//! The next id is whatever `append_string_by_id` leaves in eax, plus 1. That is the previous id
//! for string-table ids, but a string pointer for the pointer-table branches, so a run only
//! makes sense over string-table ids. A count of 0 wraps and never ends: refused, as is a text
//! longer than the buffer, and the hook leaves both to the original.

use crate::errors::BadInput;
use crate::framework::{EBX, RawHook, Regs, SelfTest, game_ptr, hook_asm, rva, shadow};
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::string_table::{LOW_IDS, blob, entry_count, ptr_low};
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::text_from_ids", rva(0x40229B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TEXT_RVA: usize = rva(0x93E430); // text: shared text buffer, up to the kerning table
pub const TEXT_LEN: usize = 0x93E830 - 0x93E430;

pub fn text() -> *mut u8 {
    game_ptr(TEXT_RVA)
}

// ---- Rust version ----

/// The bytes the original leaves in the text buffer, including the NUL (converted:
/// `rc2k_core::strings::table::text_from_ids`).
pub unsafe fn text_from_ids(first_id: u32, count: u32) -> Result<Vec<u8>, BadInput> {
    if count == 0 {
        return Err(BadInput(
            "count 0: the original's counter wraps and it never ends",
        ));
    }
    rc2k_core::strings::table::text_from_ids(first_id, count, TEXT_LEN, |id, out| unsafe {
        append_string_by_id(id, &mut |p, n| out.extend_from_slice(from_raw_parts(p, n)))
    })
    .ok_or(BadInput(
        "text longer than the text buffer: the original writes past it",
    ))
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (first_id, count) = (r.eax, r.ecx);

    let Ok(mine) = (unsafe { text_from_ids(first_id, count) }) else {
        unsafe { call_original(r) };
        return;
    };

    if !shadow() {
        unsafe { copy_nonoverlapping(mine.as_ptr(), text(), mine.len()) };
        r.ebx = text() as u32;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.ebx != text() as u32 || unsafe { from_raw_parts(text(), mine.len()) } != &mine[..] {
        HOOK.mismatch(|| {
            format!(
                "first_id={first_id:#x} count={count}: original {} rust {}",
                bytes_str(unsafe { from_raw_parts(text(), mine.len()) }),
                bytes_str(&mine)
            )
        });
    }
    r.ebx = o.ebx;
}

// ---- self-test: runs of ids from the real string table ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if blob().is_null() {
        return t.skip("string table not loaded");
    }
    let entries = entry_count() as u32;
    // Only ids that take the string-table branch, so every step of a run is a real id.
    let from_table = |id: u32| {
        id < entries
            && id & 0x1000 == 0
            && !(LOW_IDS.contains(&id) && unsafe { !ptr_low(id).is_null() })
    };

    // The buffer is live game state and the game thread may read it at any time, so it is never
    // filled with a test pattern: it only ever holds real NUL-terminated text. Each case checks
    // the text the original wrote and that the bytes after it are untouched, then restores it.
    for first_id in (0..entries).step_by(37) {
        for count in 1..=3u32 {
            if !(0..count).all(|k| from_table(first_id + k)) {
                continue;
            }
            let Ok(mine) = (unsafe { text_from_ids(first_id, count) }) else {
                continue;
            };

            let saved = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
            let before = Regs {
                eax: first_id,
                ecx: count,
                ..Regs::default()
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };

            let mut want = saved.clone();
            want[..mine.len()].copy_from_slice(&mine);
            let now = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
            unsafe { copy_nonoverlapping(saved.as_ptr(), text(), TEXT_LEN) };
            let preserved = ro.preserved_except(&before, EBX);

            t.check(ro.ebx == text() as u32 && now == want && preserved, || {
                format!(
                    "first_id={first_id:#x} count={count} original {} rust {} ebx={:#x} preserved={preserved}",
                    bytes_str(&now[..mine.len()]),
                    bytes_str(&mine),
                    ro.ebx
                )
            });
        }
    }
    t.finish()
}
