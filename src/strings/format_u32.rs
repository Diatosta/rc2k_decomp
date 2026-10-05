use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, hook_asm, log};
use crate::strings::utils::bytes_str;
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "strings::format_u32",
    0x40204A - 0x400000,
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

/// Mirrors the original's 16-byte stack buffer: 5 filler '0', 10 digits, NUL.
pub fn reimplemented(value: u32, flags: u8) -> Vec<u8> {
    let mut buf = [b'0'; 16];
    buf[5..15].copy_from_slice(format!("{value:010}").as_bytes());
    buf[15] = 0;

    let width = (flags & 0x0F) as usize;
    let mut start = 15 - width;
    if flags & 0x10 != 0 {
        // fixed width, zero padded: print the last `width` characters as they are
    } else if flags & 0x20 != 0 {
        let mut s = start;
        while buf[s] == b'0' {
            buf[s] = b' ';
            s += 1;
        }
        if buf[s] == 0 {
            buf[s - 1] = b'0'; // all zeros: keep a final '0'
        }
    } else {
        start = 5;
        while buf[start] == b'0' {
            start += 1;
        }
        if buf[start] == 0 {
            start -= 1; // all zeros: keep a single '0'
        }
    }
    buf[start..15].to_vec()
}

// eax = value, ecx = flags, ebx = dst -> new dst in ebx; everything else preserved
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let (value, flags, dst) = (r.eax, r.ecx as u8, r.ebx as *mut u8);
    let mine = reimplemented(value, flags);

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        unsafe { std::ptr::copy_nonoverlapping(mine.as_ptr(), dst, mine.len()) };
        r.ebx = (dst as usize + mine.len()) as u32;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.ebx as usize).wrapping_sub(dst as usize);
    let same = orig_len == mine.len() && unsafe { from_raw_parts(dst, orig_len) } == &mine[..];
    if !same {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "MISMATCH format_u32(value={value}, flags={flags:#04x}): original wrote {orig_len} bytes, rust {}",
                mine.len()
            ));
        }
    }
    r.ebx = o.ebx;
}

pub fn self_test() -> (usize, usize) {
    if TRAMPOLINE.load(Ordering::Relaxed) == 0 {
        log("format_u32 selftest SKIPPED: original not available");
        return (0, 0);
    }
    let values = [
        0u32,
        1,
        9,
        10,
        99,
        100,
        1234,
        99_999,
        123_456,
        999_999_999,
        1_000_000_000,
        2_147_483_648,
        4_294_967_295,
    ];
    let (mut cases, mut failures) = (0, 0);

    for &v in &values {
        for mode in [0x00u8, 0x10, 0x20, 0x30] {
            for width in 0..=15u8 {
                for extra in [0x00u8, 0xC0] {
                    // 0x40/0x80 are ignored by the original
                    let flags = mode | width | extra;
                    cases += 1;
                    let mut buf = [0xAAu8; 32];
                    let mut ro = Regs {
                        eax: v,
                        ecx: flags as u32,
                        ebx: buf.as_mut_ptr() as u32,
                        ..Regs::default()
                    };
                    unsafe { call_original(&mut ro) };
                    let orig_len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);
                    let mine = reimplemented(v, flags);

                    if buf.get(..orig_len) != Some(&mine[..]) {
                        failures += 1;
                        log(&format!(
                            "format_u32 selftest FAILED: value={v} flags={flags:#04x} original {} rust {}",
                            bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                            bytes_str(&mine)
                        ));
                    }
                }
            }
        }
    }
    (cases, failures)
}
