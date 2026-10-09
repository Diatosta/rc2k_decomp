//! `strings::str_append` (0x401F7D): copies a NUL-terminated string to the cursor, without the NUL.
//!
//! In:  eax = source string, ebx = cursor
//! Out: ebx = new cursor

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, log, rva, shadow, trace};
use crate::strings::utils::{show, written};
use std::ffi::{CStr, c_char};
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

// A loop target lies inside the first 5 bytes, so the original runs from a private copy.
pub static HOOK: RawHook = RawHook::new("strings::str_append", rva(0x401F7D), thunk, &ORIGINAL)
    .private_copy(0x11)
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn str_append(mut src: *const u8, mut cursor: *mut u8) -> *mut u8 {
    unsafe {
        loop {
            let c = *src;
            src = src.add(1);
            if c == 0 {
                break;
            }
            *cursor = c;
            cursor = cursor.add(1);
        }
    }
    cursor
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (src, cursor) = (r.eax as *const u8, r.ebx as *mut u8);

    if trace() {
        log(&format!("str_append({})", show(src)));
    }

    if !shadow() {
        r.ebx = unsafe { str_append(src, cursor) } as u32;
        return;
    }

    // Shadow: Rust writes into scratch, the original does the real writing.
    let len = unsafe { CStr::from_ptr(src as *const c_char) }
        .to_bytes()
        .len();
    let mut scratch = vec![0u8; len + 1];
    let mine = unsafe { str_append(src, scratch.as_mut_ptr()) };
    let mine_len = mine as usize - scratch.as_ptr() as usize;

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.ebx as usize).wrapping_sub(cursor as usize);

    if orig_len != mine_len || unsafe { from_raw_parts(cursor, mine_len) } != &scratch[..mine_len] {
        HOOK.mismatch(|| format!("original wrote {orig_len} bytes, rust {mine_len}"));
    }
    r.ebx = o.ebx;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let long: Vec<u8> = (0..300).map(|i| b'a' + (i % 26) as u8).collect();
    let samples: [&[u8]; 5] = [b"", b"a", b"hello", b"Error code [", &long];

    for s in samples {
        let mut src = s.to_vec();
        src.push(0);
        let n = s.len() + 8;
        let (mut buf_o, mut buf_r) = (vec![0xAAu8; n], vec![0xAAu8; n]);

        let mut ro = Regs {
            eax: src.as_ptr() as u32,
            ebx: buf_o.as_mut_ptr() as u32,
            ..Regs::default()
        };
        unsafe { call_original(&mut ro) };
        let end_o = ro.ebx as *const u8;
        let end_r = unsafe { str_append(src.as_ptr(), buf_r.as_mut_ptr()) };

        let same_end = (end_o as usize).wrapping_sub(buf_o.as_ptr() as usize)
            == (end_r as usize).wrapping_sub(buf_r.as_ptr() as usize);
        t.check(same_end && buf_o == buf_r, || {
            format!(
                "original {} rust {}",
                written(&buf_o, end_o),
                written(&buf_r, end_r)
            )
        });
    }
    t.finish()
}
