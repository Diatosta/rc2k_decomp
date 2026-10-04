use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, TRACE, hook_asm, log};
use crate::strings::utils::{show, written};
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::str_append", 0x401F7D - 0x400000, thunk, &ORIGINAL)
        .private_copy(0x11)
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub(super) unsafe fn reimplemented(mut src: *const u8, mut dst: *mut u8) -> *mut u8 {
    unsafe {
        loop {
            let c = *src;
            src = src.add(1);
            if c == 0 {
                break;
            }
            *dst = c;
            dst = dst.add(1);
        }
    }
    dst
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let (src, dst) = (r.eax as *const u8, r.ebx as *mut u8);

    if TRACE.load(Ordering::Relaxed) {
        log(&format!("str_append({})", show(src)));
    }

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        r.ebx = unsafe { reimplemented(src, dst) } as u32;
        return;
    }

    // Shadow: Rust writes into scratch, the original does the real writing
    let mut len = 0;
    while unsafe { *src.add(len) } != 0 {
        len += 1;
    }

    let mut scratch = vec![0u8; len + 1];
    let mine = unsafe { reimplemented(src, scratch.as_mut_ptr()) };
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
                "Mismatch in str_append: original wrote {orig_len} bytes, rust {mine_len}"
            ));
        }
    }

    r.ebx = o.ebx;
}

/// Calls the original directly and compares the result with the Rust version
pub fn self_test() -> (usize, usize) {
    if ORIGINAL.load(Ordering::Relaxed) == 0 {
        return (0, 0);
    }
    let long: Vec<u8> = (0..300).map(|i| b'a' + (i % 26) as u8).collect();
    let samples: [&[u8]; 5] = [b"", b"a", b"hello", b"Error code [", &long];
    let mut failures = 0;

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
        let eo = ro.ebx as *const u8;

        let er = unsafe { reimplemented(src.as_ptr(), buf_r.as_mut_ptr()) };

        let same_end = (eo as usize).wrapping_sub(buf_o.as_ptr() as usize)
            == (er as usize).wrapping_sub(buf_r.as_ptr() as usize);
        if !(same_end && buf_o == buf_r) {
            failures += 1;
            log(&format!(
                "str_append selftest FAILED for original {} rust {}",
                written(&buf_o, eo),
                written(&buf_r, er)
            ));
        }
    }

    (samples.len(), failures)
}
