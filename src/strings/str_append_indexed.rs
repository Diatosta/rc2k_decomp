use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, hook_asm, log};
use crate::strings::str_append;
use crate::strings::utils::written;
use std::ffi::{CStr, c_char};
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicUsize, Ordering};

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

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Pointer to string #index in the packed table
unsafe fn nth(mut table: *const u8, index: i32) -> *const u8 {
    let mut skip = index;
    while skip > 0 {
        unsafe {
            while *table != 0 {
                table = table.add(1);
            }

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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let (table, index, dst) = (r.eax as *const u8, r.ebx as i32, r.ecx as *mut u8);

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        r.ebx = unsafe { reimplemented(table, index, dst) } as u32;
        return;
    }

    // Shadow: Rust writes into scratch, the original does the real writing
    let s = unsafe { nth(table, index) };
    let len = unsafe { CStr::from_ptr(s as *const c_char) }
        .to_bytes()
        .len();
    let mut scratch = vec![0u8; len + 1];
    let mine = unsafe { str_append::reimplemented(s, scratch.as_mut_ptr()) };
    let mine_len = mine as usize - scratch.as_ptr() as usize;

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.ebx as usize).wrapping_sub(dst as usize);

    let same =
        orig_len == mine_len && unsafe { from_raw_parts(dst, mine_len) } == &scratch[..mine_len];

    if !same {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "Mismatch in str_append_indexed: original wrote {orig_len} bytes, rust {mine_len}"
            ));
        }
    }

    r.ebx = o.ebx;
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

        let mut ro = Regs {
            eax: table.as_ptr() as u32,
            ecx: index,
            ebx: buf_o.as_mut_ptr() as u32,
            ..Regs::default()
        };

        unsafe { call_original(&mut ro) };
        let eo = ro.ebx as *const u8;
        let er = unsafe { reimplemented(table.as_ptr(), index as i32, buf_r.as_mut_ptr()) };

        let same_end =
            eo as usize - buf_o.as_ptr() as usize == er as usize - buf_r.as_ptr() as usize;
        if !(same_end && buf_o == buf_r) {
            failures += 1;
            log(&format!(
                "str_append_indexed selftest FAILED: index={index} original {} rust {}",
                written(&buf_o, eo),
                written(&buf_r, er)
            ));
        }
    }
    (cases, failures)
}
