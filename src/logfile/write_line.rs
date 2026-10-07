use crate::framework::{RawHook, Regs, SHADOW_MODE, game_addr, hook_asm, log};
use std::ffi::CStr;
use std::io::{Read, Seek, SeekFrom};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{CreateFileA, SetFilePointer, WriteFile};
use windows_sys::Win32::System::Threading::Sleep;

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

// Shared body of six logging stubs. Entered by `jmp`; its `retn` returns to the stub's caller.
pub static HOOK: RawHook = RawHook::new(
    "logfile::write_line",
    0x40132C - 0x400000,
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

const fn rva(a: usize) -> usize {
    a - 0x400000
}
const LOG_LEVEL: usize = rva(0x4E0098);
const LOG_LOCK: usize = rva(0x4E009C);
const LOG_INDENT: usize = rva(0x4E00A0);
const LOG_LINE_START: usize = rva(0x4E00A4); // 1 after a CRLF-terminated line
const LAST_FILE_PATH: usize = rva(0x4F52B0);
const LINE_BUF: usize = rva(0x93E080);
const LINE_LIMIT: usize = 0x93E42C - 0x93E080; // bytes are stored while len <= this
// File mode table: 16-byte stride, the log uses mode 4 (offset 0x40).
const MODE_ACCESS: usize = rva(0x4E06EA) + 0x40;
const MODE_DISPOSITION: usize = rva(0x4E06EE) + 0x40;
const MODE_SHARE: usize = rva(0x4E06F2) + 0x40;
static LOG_PATH: &[u8] = b"var\\logfile.txt\0"; // starts with "var\": resolve_path returns it unchanged

unsafe fn rd32(r: usize) -> u32 {
    unsafe { (game_addr(r) as *const u32).read_unaligned() }
}
unsafe fn wr32(r: usize, v: u32) {
    unsafe { (game_addr(r) as *mut u32).write_unaligned(v) }
}
fn lock() -> &'static AtomicU32 {
    unsafe { &*(game_addr(LOG_LOCK) as *const AtomicU32) }
}

/// The bytes the original builds in the line buffer, and the new line-start flag.
pub fn format_line(msg: &[u8], newline: bool, indent: i32, line_start: u32) -> (Vec<u8>, u32) {
    let mut out = Vec::new();
    let mut flag = line_start;
    if flag == 1 {
        flag = 0;
        out.resize(indent.clamp(0, 80) as usize, b' '); // signed clamp
    }
    for &c in msg {
        if out.len() > LINE_LIMIT {
            break;
        } // truncation, not overflow
        out.push(c);
    }
    if newline {
        out.extend_from_slice(b"\r\n");
        flag = 1;
    } else {
        out.push(b' ');
    }
    (out, flag)
}

/// sub_402C9C. Some(bytes written) if the write was reached; None if open/seek failed.
unsafe fn append_to_file(path: &'static [u8], data: &[u8]) -> Option<u32> {
    unsafe {
        wr32(LAST_FILE_PATH, path.as_ptr() as u32); // sub_402DE8 stores resolve_path's result
        let h = CreateFileA(
            path.as_ptr(),
            rd32(MODE_ACCESS),
            rd32(MODE_SHARE),
            std::ptr::null(),
            rd32(MODE_DISPOSITION),
            0x80,
            0 as _,
        );
        if h == INVALID_HANDLE_VALUE {
            return None;
        } // original does not close: path stays set
        let (mut written, mut result) = (0u32, None);
        if SetFilePointer(h, 0, std::ptr::null_mut(), 2) != u32::MAX {
            // FILE_END
            WriteFile(
                h,
                data.as_ptr().cast(),
                data.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            );
            result = Some(written);
        }
        CloseHandle(h);
        wr32(LAST_FILE_PATH, 0);
        result
    }
}

unsafe fn rust_impl(r: &mut Regs) {
    let dl = r.edx as u8;
    let level = ((r.edx >> 8) as u8).wrapping_add(1) as u32; // `inc dh` wraps
    r.ecx = level;
    r.edx = dl as u32;
    unsafe {
        if rd32(LOG_LEVEL) < level {
            lock().store(0, Ordering::Release);
            return;
        } // eax, ebx unchanged
        let msg = CStr::from_ptr(r.eax as *const c_char).to_bytes();
        let (bytes, new_flag) =
            format_line(msg, dl != 0, rd32(LOG_INDENT) as i32, rd32(LOG_LINE_START));
        let buf = game_addr(LINE_BUF) as *mut u8; // the original leaves the line in the game buffer
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
        wr32(LOG_LINE_START, new_flag);
        let written = append_to_file(LOG_PATH, std::slice::from_raw_parts(buf, bytes.len()));
        lock().store(0, Ordering::Release);
        r.ebx = buf as u32;
        r.ecx = bytes.len() as u32;
        r.eax = written.unwrap_or(LOG_PATH.as_ptr() as u32);
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    // The original writes the real file, so there is no shadow comparison: self-test only.
    if SHADOW_MODE.load(Ordering::Relaxed) {
        unsafe { call_original(r) };
        return;
    }
    unsafe { rust_impl(r) };
}

// ---- self-test: appends lines to the real var\logfile.txt ----

type Impl = unsafe fn(&mut Regs);
unsafe fn orig(r: &mut Regs) {
    unsafe { call_original(r) }
}

fn acquire() {
    while lock().swap(1, Ordering::Acquire) != 0 {
        unsafe { Sleep(1) }
    }
} // like sub_40130A
fn file_len() -> u64 {
    std::fs::metadata("var\\logfile.txt")
        .map(|m| m.len())
        .unwrap_or(0)
}
fn file_bytes(start: u64, max: usize) -> Vec<u8> {
    let Ok(mut f) = std::fs::File::open("var\\logfile.txt") else {
        return Vec::new();
    };
    if f.seek(SeekFrom::Start(start)).is_err() {
        return Vec::new();
    }
    let mut v = Vec::new();
    let _ = f.take(max as u64).read_to_end(&mut v);
    v
}
fn regs(eax: u32, edx: u32) -> Regs {
    Regs {
        edi: 0x1111_1111,
        esi: 0x2222_2222,
        ebp: 0x3333_3333,
        esp: 0,
        ebx: 0x4444_4444,
        edx,
        ecx: 0x6666_6666,
        eax,
    }
}

/// Takes the lock as a stub would, sets the line state, runs `f` (the tail releases the lock).
/// The file offset is measured under the lock, so our bytes at [before, before+len) cannot be
/// disturbed by other logging threads.
unsafe fn run(
    f: Impl,
    msg: *const u8,
    dh: u8,
    dl: u8,
    start: u32,
    indent: i32,
) -> (Regs, u64, u32) {
    acquire();
    unsafe {
        wr32(LOG_LINE_START, start);
        wr32(LOG_INDENT, indent as u32);
    }
    let before = file_len();
    let mut r = regs(msg as u32, (dh as u32) << 8 | dl as u32);
    unsafe { f(&mut r) };
    (r, before, unsafe { rd32(LOG_LINE_START) }) // may race with another logging thread
}

pub fn self_test() -> (usize, usize) {
    let (mut cases, mut fails) = (0usize, 0usize);
    let impls: [Impl; 2] = [orig, rust_impl];
    let lvl = unsafe { rd32(LOG_LEVEL) };
    let dh_log = (lvl as u8).wrapping_sub(1); // level byte = lvl & 0xFF <= lvl, so it always logs
    acquire();
    let (saved_start, saved_indent) = unsafe { (rd32(LOG_LINE_START), rd32(LOG_INDENT)) };
    lock().store(0, Ordering::Release);

    let long: Vec<u8> = std::iter::repeat(b'x').take(1200).chain([0]).collect();
    let msgs: Vec<(Vec<u8>, bool, i32, u32)> = vec![
        (b"rc2k_hook selftest: plain\0".to_vec(), true, 0, 0),
        (b"rc2k_hook selftest: no newline\0".to_vec(), false, 0, 0),
        (b"rc2k_hook selftest: indented\0".to_vec(), true, 3, 1),
        (
            b"rc2k_hook selftest: indent clamped\0".to_vec(),
            true,
            200,
            1,
        ),
        (
            b"rc2k_hook selftest: negative indent\0".to_vec(),
            true,
            -5,
            1,
        ),
        (b"\0".to_vec(), true, 0, 0),
        (long, true, 0, 0), // truncated at the buffer limit
    ];
    let buf = game_addr(LINE_BUF) as u32;
    for (msg, newline, indent, start) in &msgs {
        let dl = *newline as u8;
        let text = &msg[..msg.len() - 1];
        let (want, want_flag) = format_line(text, *newline, *indent, *start);
        let regs_ok = |r: &Regs| {
            r.ebx == buf
                && r.ecx == want.len() as u32
                && r.edx == dl as u32
                && r.eax == want.len() as u32
                && r.esi == 0x2222_2222
                && r.edi == 0x1111_1111
                && r.ebp == 0x3333_3333
        };
        let mut ok = true;
        for f in impls {
            let (r, before, flag) = unsafe { run(f, msg.as_ptr(), dh_log, dl, *start, *indent) };
            ok &= file_bytes(before, want.len()) == want && regs_ok(&r) && flag == want_flag;
        }
        cases += 1;
        if !ok {
            fails += 1;
            log(&format!(
                "write_line FAIL {:?} (want len {})",
                String::from_utf8_lossy(&text[..text.len().min(40)]),
                want.len()
            ));
        }
    }

    // Level gate: dh = lvl gives level lvl+1 > lvl, so nothing may be written.
    if lvl < 255 {
        let msg = b"rc2k_hook selftest: MUST NOT APPEAR\0";
        for f in impls {
            let (r, before, _) = unsafe { run(f, msg.as_ptr(), lvl as u8, 1, 0, 0) };
            let leaked = file_bytes(before, 1 << 16)
                .windows(msg.len() - 1)
                .any(|w| w == &msg[..msg.len() - 1]);
            cases += 1;
            if leaked
                || r.eax != msg.as_ptr() as u32
                || r.ecx != lvl + 1
                || r.edx != 1
                || r.ebx != 0x4444_4444
            {
                fails += 1;
                log(&format!(
                    "write_line FAIL level gate (leaked={leaked}, eax={:#x}, ecx={})",
                    r.eax, r.ecx
                ));
            }
        }
    }

    acquire();
    unsafe {
        wr32(LOG_LINE_START, saved_start);
        wr32(LOG_INDENT, saved_indent);
    }
    lock().store(0, Ordering::Release);
    (cases, fails)
}
