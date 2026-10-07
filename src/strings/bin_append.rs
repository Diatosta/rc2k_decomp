use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, hook_asm, log};
use crate::strings::utils::bytes_str;
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "strings::bin_append",
    0x40201D - 0x400000,
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

/// The 32 binary digits of `value`, MSB first.
fn bits(value: u32) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = b'0' + ((value >> (31 - i)) & 1) as u8;
    }
    out
}

/// Writes the low `digits` bits (digits <= 32), no NUL; returns the new cursor.
unsafe fn reimplemented(value: u32, digits: u32, dst: *mut u8) -> *mut u8 {
    let t = bits(value);
    let n = digits as usize;
    unsafe {
        std::ptr::copy_nonoverlapping(t.as_ptr().add(32 - n), dst, n);
        dst.add(n)
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let (value, digits, dst) = (r.eax, r.ecx, r.ebx as *mut u8);

    // Original reads below its stack buffer here: do not reimplement.
    if digits > 32 {
        unsafe { call_original(r) };
        return;
    }
    if !SHADOW_MODE.load(Ordering::Relaxed) {
        r.ebx = unsafe { reimplemented(value, digits, dst) } as u32;
        return;
    }
    // Shadow: the original writes to the real buffer; check what it wrote.
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let n = digits as usize;
    let want = bits(value);
    let wrote = o.ebx.wrapping_sub(r.ebx) as usize;
    let ok = wrote == n && unsafe { std::slice::from_raw_parts(dst, n) } == &want[32 - n..];
    if !ok && HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed) < MAX_LOGGED {
        log(&format!(
            "bin_append MISMATCH value={value:#x} digits={digits} wrote={wrote}"
        ));
    }
    r.ebx = o.ebx;
}

pub fn self_test() -> (usize, usize) {
    let values = [
        0u32,
        1,
        0x8000_0000,
        0xFFFF_FFFF,
        0xA5A5_A5A5,
        0x1234_5678,
        0xDEAD_BEEF,
        0xFFFF,
    ];
    let (mut cases, mut fails) = (0usize, 0usize);
    for &v in &values {
        for digits in 0..=32u32 {
            let mut buf = [0xAAu8; 64];
            let dst = unsafe { buf.as_mut_ptr().add(8) };
            let mut r = Regs {
                edi: 0x1111_1111,
                esi: 0x2222_2222,
                ebp: 0x3333_3333,
                esp: 0,
                ebx: dst as u32,
                edx: 0x5555_5555,
                ecx: digits,
                eax: v,
            };
            unsafe { call_original(&mut r) };

            let n = digits as usize;
            let mut want = [0xAAu8; 64];
            want[8..8 + n].copy_from_slice(&bits(v)[32 - n..]);
            let preserved = r.eax == v
                && r.ecx == digits
                && r.edx == 0x5555_5555
                && r.esi == 0x2222_2222
                && r.edi == 0x1111_1111
                && r.ebp == 0x3333_3333;
            cases += 1;
            if buf != want || r.ebx != dst as u32 + digits || !preserved {
                fails += 1;
                log(&format!(
                    "bin_append FAIL v={v:#x} digits={digits}: got {} want {} ebx={:#x} preserved={preserved}",
                    bytes_str(&buf),
                    bytes_str(&want),
                    r.ebx
                ));
            }
        }
    }
    (cases, fails) // failures only are logged
}
