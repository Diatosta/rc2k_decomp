use std::arch::naked_asm;
use std::ffi::{c_char, CStr};
use std::slice::from_raw_parts;
use crate::framework::{RawHook, MAX_LOGGED, SHADOW_MODE, log};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::strings::str_append;
use crate::strings::utils::written;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "strings::str_append_indexed",
    0x401F8E - 0x400000,
    thunk,
    &ORIGINAL,
)
.private_copy(0x15)
.relocate_calls(&[0x0D])
.self_test(self_test)
    .manual_patch();

/// Pointer to string #index in the packed table
unsafe fn nth(mut table: *const u8, index: i32) -> *const u8 {
    let mut skip = index;
    while skip > 0 {
        unsafe {
            while *table != 0 { table = table.add(1); }

            // Step past the NULL
            table = table.add(1);
        }

        skip -= 1;
    }

    table
}

unsafe fn reimplemented(table: *const u8, index: i32, dst: *mut u8) -> *mut u8 {
    unsafe { str_append::reimplemented(nth(table, index), dst) }
}

extern "C" fn dispatch(table: *const u8, index: i32, dst: *mut u8) -> *mut u8 {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    if !SHADOW_MODE.load(Ordering::Relaxed) {
        return unsafe { reimplemented(table, index, dst) };
    }

    // Shadow: Rust writes into scratch, the original does the real writing
    let s = unsafe { nth(table, index) };
    let len = unsafe { CStr::from_ptr(s as *const c_char) }.to_bytes().len();
    let mut scratch = vec![0u8; len + 1];
    let mine = unsafe { str_append::reimplemented(s, scratch.as_mut_ptr()) };
    let mine_len = mine as usize - scratch.as_ptr() as usize;

    let original = unsafe { call_original(table, index as u32, dst) };
    let orig_len = (original as usize).wrapping_sub(dst as usize);

    let same = orig_len == mine_len
    && unsafe { from_raw_parts(dst, mine_len) } == &scratch[..mine_len];

    if !same {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "Mismatch in str_append_indexed: original wrote {orig_len} bytes, rust {mine_len}"
            ));
        }
    }

    original
}

#[unsafe(naked)]
unsafe extern "C" fn thunk() {
    naked_asm!(
        "push eax", "push ecx", "push edx",
        "push ebx",              // arg 3: dst
        "push ecx",              // arg 2: index
        "push eax",              // arg 1: table
        "call {dispatch}",
        "add esp, 12",
        "mov ebx, eax",
        "pop edx", "pop ecx", "pop eax",
        "ret",
        dispatch = sym dispatch,
    )
}

#[unsafe(naked)]
unsafe extern "C" fn call_original(_table: *const u8, _index: u32, _dst: *mut u8) -> *mut u8 {
    naked_asm!(
        "push ebx",
        "mov eax, [esp + 8]",    // table
        "mov ecx, [esp + 12]",   // index
        "mov ebx, [esp + 16]",   // dst
        "call dword ptr [{t}]",
        "mov eax, ebx",
        "pop ebx",
        "ret",
        t = sym ORIGINAL,
    )
}

pub fn self_test() -> (usize, usize) {
    if ORIGINAL.load(Ordering::Relaxed) == 0 {
        return (0, 0);
    }

    // index 2 is an empty string on purpose
    let table: &[u8] = b"Alpha\0Beta\0\0Gamma\0";
    let (mut cases, mut failures) = (0, 0);

    for index in 0..=3u32 {
        cases += 1;
        let (mut buf_o, mut buf_r) = ([0xAAu8; 16], [0xAAu8; 16]);
        let (eo, er) = unsafe {
            (
                call_original(table.as_ptr(), index, buf_o.as_mut_ptr()),
                reimplemented(table.as_ptr(), index as i32, buf_r.as_mut_ptr()),
            )
        };
        let same_end =
            eo as usize - buf_o.as_ptr() as usize == er as usize - buf_r.as_ptr() as usize;
        if !(same_end && buf_o == buf_r) {
            failures += 1;
            log(&format!(
                "str_append_indexed selftest FAILED: index={index} original {} rust {}",
                written(&buf_o, eo), written(&buf_r, er)
            ));
        }
    }
    (cases, failures)
}