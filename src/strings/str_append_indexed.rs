//! `strings::str_append_indexed` (0x401F8E): appends string number `index` of a packed table
//! of NUL-terminated strings.
//!
//! In:  eax = table, ecx = index, ebx = cursor
//! Out: ebx = new cursor

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::str_append::str_append;
use crate::strings::utils::written;
use std::ffi::{CStr, c_char};
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

// `detour` refuses to patch this function, so the jump is written by hand and the original
// runs from a private copy (its call at +0x0D relocated).
pub static HOOK: RawHook = RawHook::new(
    "strings::str_append_indexed",
    rva(0x401F8E),
    thunk,
    &ORIGINAL,
)
.private_copy(0x15)
.relocate_calls(&[0x0D])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Pointer to string number `index` in the packed table.
unsafe fn nth(mut table: *const u8, index: i32) -> *const u8 {
    let mut skip = index;
    while skip > 0 {
        unsafe {
            while *table != 0 {
                table = table.add(1);
            }
            table = table.add(1); // step past the NUL
        }
        skip -= 1;
    }
    table
}

pub unsafe fn str_append_indexed(table: *const u8, index: i32, cursor: *mut u8) -> *mut u8 {
    unsafe { str_append(nth(table, index), cursor) }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (table, index, cursor) = (r.eax as *const u8, r.ecx as i32, r.ebx as *mut u8);

    if !shadow() {
        r.ebx = unsafe { str_append_indexed(table, index, cursor) } as u32;
        return;
    }

    // Shadow: Rust writes into scratch, the original does the real writing.
    let s = unsafe { nth(table, index) };
    let len = unsafe { CStr::from_ptr(s as *const c_char) }
        .to_bytes()
        .len();
    let mut scratch = vec![0u8; len + 1];
    let mine = unsafe { str_append(s, scratch.as_mut_ptr()) };
    let mine_len = mine as usize - scratch.as_ptr() as usize;

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.ebx as usize).wrapping_sub(cursor as usize);

    if orig_len != mine_len || unsafe { from_raw_parts(cursor, mine_len) } != &scratch[..mine_len] {
        HOOK.mismatch(|| {
            format!("index={index}: original wrote {orig_len} bytes, rust {mine_len}")
        });
    }
    r.ebx = o.ebx;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // index 2 is an empty string on purpose
    let table: &[u8] = b"Alpha\0Beta\0\0Gamma\0";

    for index in 0..=3u32 {
        let (mut buf_o, mut buf_r) = ([0xAAu8; 16], [0xAAu8; 16]);

        let mut ro = Regs {
            eax: table.as_ptr() as u32,
            ecx: index,
            ebx: buf_o.as_mut_ptr() as u32,
            ..Regs::default()
        };
        unsafe { call_original(&mut ro) };
        let end_o = ro.ebx as *const u8;
        let end_r = unsafe { str_append_indexed(table.as_ptr(), index as i32, buf_r.as_mut_ptr()) };

        let same_end = (end_o as usize).wrapping_sub(buf_o.as_ptr() as usize)
            == (end_r as usize).wrapping_sub(buf_r.as_ptr() as usize);
        t.check(same_end && buf_o == buf_r, || {
            format!(
                "index={index} original {} rust {}",
                written(&buf_o, end_o),
                written(&buf_r, end_r)
            )
        });
    }
    t.finish()
}
