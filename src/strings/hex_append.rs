use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, hook_asm, log};
use crate::strings::utils::written;
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "strings::hex_append",
    0x40211D - 0x400000,
    thunk,
    &TRAMPOLINE,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = TRAMPOLINE
);

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

unsafe fn reimplemented(value: u32, digits: usize, mut dst: *mut u8) -> *mut u8 {
    for i in (0..digits).rev() {
        unsafe {
            *dst = HEX_DIGITS[((value >> (i * 4)) & 0xF) as usize];
            dst = dst.add(1);
        }
    }

    dst
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let (value, digits, dst) = (r.eax, r.ecx, r.ebx as *mut u8);

    if digits > 8 {
        let mut o = *r;
        unsafe { call_original(&mut o) };
        r.ebx = o.ebx;
        return;
    }

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        r.ebx = unsafe { reimplemented(value, digits as usize, dst) } as u32;
        return;
    }

    let mut scratch = [0xAAu8; 8];
    let mine = unsafe { reimplemented(value, digits as usize, scratch.as_mut_ptr()) };
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
                "MISMATCH hex_append(value={value:#x}, digits={digits}): original wrote {orig_len} bytes, rust {mine_len}"
            ));
        }
    }

    r.ebx = o.ebx;
}

pub fn self_test() -> (usize, usize) {
    if TRAMPOLINE.load(Ordering::Relaxed) == 0 {
        return (0, 0);
    }

    let values = [
        0u32,
        1,
        0xF,
        0x10,
        0xFF,
        0x1234ABCD,
        0xDEADBEEF,
        0x8010_8601,
        0xFFFF_FFFF,
    ];
    let (mut cases, mut failures) = (0, 0);

    for &v in &values {
        for digits in 0..=8u32 {
            cases += 1;
            let (mut buf_o, mut buf_r) = ([0xAAu8; 16], [0xAAu8; 16]);

            let mut ro = Regs {
                eax: v,
                ecx: digits,
                ebx: buf_o.as_mut_ptr() as u32,
                ..Regs::default()
            };
            unsafe { call_original(&mut ro) };
            let eo = ro.ebx as *const u8;
            let er = unsafe { reimplemented(v, digits as usize, buf_r.as_mut_ptr()) };

            let same_end = (eo as usize).wrapping_sub(buf_o.as_ptr() as usize)
                == (er as usize).wrapping_sub(buf_r.as_ptr() as usize);

            if !(same_end && buf_o == buf_r) {
                failures += 1;
                log(&format!(
                    "hex_append selftest FAILED: value={v:#x} digits={digits} original {} rust {}",
                    written(&buf_o, eo),
                    written(&buf_r, er)
                ));
            }
        }
    }

    (cases, failures)
}
