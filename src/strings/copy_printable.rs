//! `strings::copy_printable` (0x4017F2): copies a name, leaving out what cannot be shown.
//!
//! In:  eax = the text (0: none), ebx = where to write, ecx = the room, NUL included
//! Out: every register preserved
//! At most room - 1 characters are looked at, up to the text's end. One whose low seven
//! bits are under 0x20 is left out; the others are written, in lower case through
//! `g_lowerCase` unless `dword_4E0144` is 1. A NUL ends what was written. Nine callers.
//! A room of 0 lets the original look at 2^32 - 1 characters: refused.
//! Manual patch with a private copy (0x37 bytes): short jumps in the first bytes.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::strings::same_text::case_table;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("strings::copy_printable", rva(0x4017F2), thunk, &ORIGINAL)
    .private_copy(0x37)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// dword_4E0144: 1 = the case is kept.
const KEEP_CASE_RVA: usize = rva(0x4E0144);

// ---- Rust version ----

/// What is written, with its NUL (converted: `rc2k_core::strings::case::CaseTable::printable`).
pub unsafe fn printable(text: *const u8, room: u32) -> Result<Vec<u8>, BadInput> {
    let mut looked_at = Vec::new();
    if !text.is_null() {
        // As far as the original reads: room - 1 characters or the NUL.
        for n in 0..(room as usize).saturating_sub(1) {
            let c = unsafe { text.add(n).read() };
            if c == 0 {
                break;
            }
            looked_at.push(c);
        }
    }
    let keep_case = unsafe { game_read::<u32>(KEEP_CASE_RVA) } == 1;
    case_table()
        .printable(&looked_at, room as usize, keep_case)
        .ok_or(BadInput("no room for a name: the original runs on"))
}

pub unsafe fn copy_printable(text: *const u8, to: *mut u8, room: u32) -> Result<(), BadInput> {
    unsafe {
        let bytes = printable(text, room)?;
        std::ptr::copy(bytes.as_ptr(), to, bytes.len());
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (text, to, room) = (r.eax as *const u8, r.ebx as *mut u8, r.ecx);
    // The text and the place may be the same: what is written is worked out first.
    let mine = match unsafe { printable(text, room) } {
        Ok(mine) => mine,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, Refused::from(e), r, call_original) };
            return;
        }
    };
    if !shadow() {
        unsafe { std::ptr::copy(mine.as_ptr(), to, mine.len()) };
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let same = unsafe { std::slice::from_raw_parts(to, mine.len()) } == &mine[..];
    if !same || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "room={room:#x}: the original did not write {:?}",
                String::from_utf8_lossy(&mine)
            )
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let keep_before: u32 = unsafe { game_read(KEEP_CASE_RVA) };
    let texts: [&[u8]; 7] = [
        b"\0",
        b"Rafael\0",
        b"Mixed CASE and 123\0",
        b"tab\there\rand\nthere\0",
        b"\x81\x9Fhigh \xC9\xE9\xA0\0",
        b"a name that is longer than most rooms are\0",
        b"\x01\x02\x03\0",
    ];
    for keep_case in [0u32, 1, 2] {
        for text in texts
            .iter()
            .map(|text| text.as_ptr())
            .chain([std::ptr::null()])
        {
            for room in [1u32, 2, 5, 0x1F, 0x40] {
                let run = |rust: bool| unsafe {
                    game_write(KEEP_CASE_RVA, keep_case);
                    let mut to = [0xAAu8; 0x48];
                    let before = Regs {
                        eax: text as u32,
                        ebx: to.as_mut_ptr() as u32 + 4,
                        ecx: room,
                        ..Regs::default()
                    };
                    let done = if rust {
                        copy_printable(text, to.as_mut_ptr().add(4), room).is_ok()
                    } else {
                        let mut ro = before;
                        call_original(&mut ro);
                        ro.preserved_except(&before, 0)
                    };
                    (done, to)
                };
                let (theirs, mine) = (run(false), run(true));
                t.check(theirs == mine && theirs.0, || {
                    format!(
                        "keep case={keep_case} text={:?} room={room:#x}: original {} {:?}; rust {} {:?}",
                        (!text.is_null())
                            .then(|| unsafe { std::ffi::CStr::from_ptr(text as *const _) }),
                        theirs.0,
                        crate::strings::utils::bytes_str(&theirs.1),
                        mine.0,
                        crate::strings::utils::bytes_str(&mine.1)
                    )
                });
            }
        }
    }
    unsafe { game_write(KEEP_CASE_RVA, keep_before) };
    t.finish()
}
