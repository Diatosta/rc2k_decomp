use std::arch::naked_asm;
use std::ptr::{copy_nonoverlapping, null};
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use detour::RawDetour;
use windows_sys::Win32::System::Memory::{VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
use crate::framework::{log, HookStats, MAX_LOGGED, SHADOW_MODE};

pub static STATS: HookStats = HookStats::new();
static DETOUR: OnceLock<RawDetour> = OnceLock::new();
static ORIGINAL_COPY: AtomicUsize = AtomicUsize::new(0);
const ORIG_LEN: usize = 0x11;

unsafe fn reimplemented(mut src: *const u8, mut dst: *mut u8) -> *mut u8 {
    unsafe {
        loop {
            let c = *src;
            src = src.add(1);
            if c == 0 { break; }
            *dst = c;
            dst = dst.add(1);
        }
    }
    dst
}

extern "C" fn dispatch(src: *const u8, dst: *mut u8) -> *mut u8 {
    STATS.calls.fetch_add(1, Ordering::Relaxed);
    if !SHADOW_MODE.load(Ordering::Relaxed) {
        return unsafe { reimplemented(src, dst) };
    }

    // Shadow: Rust writes into scratch, the original does the real writing
    let mut len = 0;
    while unsafe { *src.add(len) } != 0 {
        len += 1;
    }

    let mut scratch = vec![0u8; len + 1];
    let mine = unsafe { reimplemented(src, scratch.as_mut_ptr()) };
    let mine_len = mine as usize - scratch.as_ptr() as usize;

    let original = unsafe { call_original(src, dst) };
    let orig_len = original as usize - dst as usize;
    let written = unsafe { from_raw_parts(dst, orig_len) };

    if orig_len != mine_len || written != &scratch[..mine_len] {
        let n = STATS.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!("Mismatch in maybe_str_append: original wrote {orig_len} bytes, rust {mine_len}"));
        }
    }

    original
}

#[unsafe(naked)]
unsafe extern "C" fn thunk() {
    naked_asm!(
        "push eax", "push ecx", "push edx",
        "push ebx",  // arg 2: dst
        "push eax",  // arg 1: src
        "call {dispatch}",
        "add esp, 8",
        "mov ebx, eax",
        "pop edx", "pop ecx", "pop eax",
        "ret",
        dispatch = sym dispatch,
    )
}

#[unsafe(naked)]
unsafe extern "C" fn call_original(_src: *const u8, _dst: *mut u8) -> *mut u8 {
    naked_asm!(
        "push ebx",
        "mov eax, [esp + 8]",  // src
        "mov ebx, [esp + 12]", // dst
        "call dword ptr [{t}]",
        "mov eax, ebx",
        "pop ebx",
        "ret",
        t = sym ORIGINAL_COPY,
    )
}

pub unsafe fn install(addr: usize) -> Result<(), detour::Error> {
    unsafe {
        let mem = VirtualAlloc(null(), ORIG_LEN, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) as *mut u8;
        assert!(!mem.is_null(), "VirtualAlloc failed");
        copy_nonoverlapping(addr as *const u8, mem, ORIG_LEN);
        ORIGINAL_COPY.store(mem as usize, Ordering::Relaxed);

        let d = RawDetour::new(addr as *const (), thunk as *const ())?;
        d.enable()?;
        let _ = DETOUR.set(d);
    }

    Ok(())
}

pub unsafe fn set_enabled(on: bool) -> Result<(), detour::Error> {
    let Some(d) = DETOUR.get() else { return Ok(()) };
    unsafe { if on { d.enable() } else { d.disable() } }
}

/// Calls the original directly and compares the result with the Rust version
pub fn self_test() -> (usize, usize) {
    if ORIGINAL_COPY.load(Ordering::Relaxed) == 0 { return (0, 0); }
    let long: Vec<u8> = (0..300).map(|i| b'a' + (i % 26) as u8).collect();
    let samples: [&[u8]; 5] = [b"", b"a", b"hello", b"Error code [", &long];
    let mut failures = 0;

    for s in samples {
        let mut src = s.to_vec();
        src.push(0);
        let n = s.len() + 8;
        let (mut buf_o, mut buf_r) = (vec![0xAAu8; n], vec![0xAAu8; n]);
        let (eo, er) = unsafe {
            (call_original(src.as_ptr(), buf_o.as_mut_ptr()),
            reimplemented(src.as_ptr(), buf_r.as_mut_ptr()))
        };

        let same_end = eo as usize - buf_o.as_ptr() as usize == er as usize - buf_r.as_ptr() as usize;
        if !(same_end && buf_o == buf_r) {
            failures += 1;
            log(&format!("maybe_str_append selftest FAILED for {:?}", String::from_utf8_lossy(&s[..s.len().min(20)])));
        }
    }

    (samples.len(), failures)
}