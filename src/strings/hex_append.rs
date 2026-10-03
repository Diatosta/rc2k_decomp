use crate::framework::{MAX_LOGGED, RawHook, SHADOW_MODE, log};
use crate::strings::utils::written;
use std::arch::naked_asm;
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

extern "C" fn dispatch(value: u32, digits: u32, dst: *mut u8) -> *mut u8 {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);

    if digits > 8 {
        return unsafe { call_original(value, digits, dst) };
    }

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        return unsafe { reimplemented(value, digits as usize, dst) };
    }

    let mut scratch = [0xAAu8; 8];
    let mine = unsafe { reimplemented(value, digits as usize, scratch.as_mut_ptr()) };
    let mine_len = mine as usize - scratch.as_ptr() as usize;

    let original = unsafe { call_original(value, digits, dst) };
    let orig_len = original as usize - dst as usize;
    let written = unsafe { from_raw_parts(dst, orig_len) };

    if orig_len != mine_len || written != &scratch[..mine_len] {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "MISMATCH hex_append(value={value:#x}, digits={digits}): original wrote {orig_len} bytes, rust {mine_len}"
            ));
        }
    }

    original
}

#[unsafe(naked)]
unsafe extern "C" fn thunk() {
    naked_asm!(
        "push eax", "push ecx", "push edx",
        "push ebx",            // arg 3: dst
        "push ecx",            // arg 2: digits
        "push eax",            // arg 1: value
        "call {dispatch}",
        "add esp, 12",
        "mov ebx, eax",
        "pop edx", "pop ecx", "pop eax",
        "ret",
        dispatch = sym dispatch,
    )
}

#[unsafe(naked)]
unsafe extern "C" fn call_original(_value: u32, _digits: u32, _dst: *mut u8) -> *mut u8 {
    naked_asm!(
        "push ebx",
        "mov eax, [esp + 8]",   // value
        "mov ecx, [esp + 12]",  // digits
        "mov ebx, [esp + 16]",  // dst
        "call dword ptr [{t}]",
        "mov eax, ebx",
        "pop ebx",
        "ret",
        t = sym TRAMPOLINE,
    )
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
            let (eo, er) = unsafe {
                (
                    call_original(v, digits, buf_o.as_mut_ptr()),
                    reimplemented(v, digits as usize, buf_r.as_mut_ptr()),
                )
            };

            let same_end =
                eo as usize - buf_o.as_ptr() as usize == er as usize - buf_r.as_ptr() as usize;
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
