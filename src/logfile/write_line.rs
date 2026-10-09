//! `logfile::write_line` (0x40132C): the tail shared by the game's seven log functions. Formats
//! one message into the line buffer and appends it to `var\logfile.txt`.
//!
//! In:  eax = message, dl = newline flag, dh = level - 1; `g_logLock` held by the caller
//! Out, logged:  eax = bytes written (or the path pointer if open or seek failed),
//!               ebx = line buffer start, ecx = line length, edx = dl
//! Out, skipped: ecx = level, edx = dl; eax and ebx unchanged
//! The lock is released on both paths. Entered by `jmp`, so its `retn` returns to the log
//! function's caller.
//!
//! The original writes the real file, so there is no shadow comparison: in shadow mode the
//! original runs alone, and the self-test is the only check of the Rust version.

use crate::framework::{
    EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, game_ptr, game_read, game_write, hook_asm, rva,
    shadow,
};
use std::ffi::{CStr, c_char};
use std::io::{Read, Seek, SeekFrom};
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{CreateFileA, SetFilePointer, WriteFile};
use windows_sys::Win32::System::Threading::Sleep;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("logfile::write_line", rva(0x40132C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const LOG_LEVEL_RVA: usize = rva(0x4E0098); // g_logLevel
const LOG_LOCK_RVA: usize = rva(0x4E009C); // g_logLock
const LOG_INDENT_RVA: usize = rva(0x4E00A0); // g_logIndent
const LOG_LINE_START_RVA: usize = rva(0x4E00A4); // g_logAtLineStart: 1 after a CRLF-terminated line
const LAST_FILE_PATH_RVA: usize = rva(0x4F52B0); // g_lastFilePath
const LINE_BUF_RVA: usize = rva(0x93E080);
const LINE_LIMIT: usize = 0x93E42C - 0x93E080; // bytes are stored while len <= this
// File mode table: 16-byte stride, the log uses mode 4 (offset 0x40).
const MODE_ACCESS_RVA: usize = rva(0x4E06EA) + 0x40;
const MODE_DISPOSITION_RVA: usize = rva(0x4E06EE) + 0x40;
const MODE_SHARE_RVA: usize = rva(0x4E06F2) + 0x40;

static LOG_PATH: &[u8] = b"var\\logfile.txt\0"; // starts with "var\": resolve_path returns it unchanged

fn lock() -> &'static AtomicU32 {
    unsafe { &*game_ptr::<AtomicU32>(LOG_LOCK_RVA) }
}

// ---- Rust version ----

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
            break; // truncation, not overflow
        }
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

/// `files::append_to_file` (0x402C9C), whose only caller is this tail.
/// Some(bytes written) if the write was reached; None if open or seek failed.
unsafe fn append_to_file(path: &'static [u8], data: &[u8]) -> Option<u32> {
    unsafe {
        // open_by_mode stores resolve_path's result
        game_write(LAST_FILE_PATH_RVA, path.as_ptr() as u32);
        let h = CreateFileA(
            path.as_ptr(),
            game_read(MODE_ACCESS_RVA),
            game_read(MODE_SHARE_RVA),
            std::ptr::null(),
            game_read(MODE_DISPOSITION_RVA),
            0x80,
            0 as _,
        );
        if h == INVALID_HANDLE_VALUE {
            return None; // the original does not close: the path stays set
        }
        let (mut written, mut result) = (0u32, None);
        if SetFilePointer(h, 0, std::ptr::null_mut(), 2) != u32::MAX {
            // 2 = FILE_END
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
        game_write(LAST_FILE_PATH_RVA, 0u32);
        result
    }
}

/// What a logged line leaves behind.
pub struct Logged {
    /// Length of the line in the line buffer.
    pub len: usize,
    /// Bytes written to the file; None if open or seek failed.
    pub written: Option<u32>,
}

/// None if the level check skipped the message. `msg` is only read when it is logged.
/// The caller must hold `g_logLock`; it is released here on both paths.
pub unsafe fn write_line(msg: *const u8, newline: bool, level_m1: u8) -> Option<Logged> {
    let level = level_m1.wrapping_add(1) as u32; // `inc dh` wraps as a byte
    unsafe {
        if game_read::<u32>(LOG_LEVEL_RVA) < level {
            lock().store(0, Ordering::Release);
            return None;
        }
        let msg = CStr::from_ptr(msg as *const c_char).to_bytes();
        let (bytes, new_flag) = format_line(
            msg,
            newline,
            game_read::<u32>(LOG_INDENT_RVA) as i32,
            game_read(LOG_LINE_START_RVA),
        );
        // The original leaves the line in the game's buffer.
        let buf = game_ptr::<u8>(LINE_BUF_RVA);
        copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
        game_write(LOG_LINE_START_RVA, new_flag);
        let written = append_to_file(LOG_PATH, from_raw_parts(buf, bytes.len()));
        lock().store(0, Ordering::Release);
        Some(Logged {
            len: bytes.len(),
            written,
        })
    }
}

// ---- hook ----

/// `write_line` with the original's registers in and out.
unsafe fn write_line_regs(r: &mut Regs) {
    let (newline, level_m1) = (r.edx as u8, (r.edx >> 8) as u8);
    r.edx = newline as u32;
    match unsafe { write_line(r.eax as *const u8, newline != 0, level_m1) } {
        Some(logged) => {
            r.eax = logged.written.unwrap_or(LOG_PATH.as_ptr() as u32);
            r.ebx = game_ptr::<u8>(LINE_BUF_RVA) as u32;
            r.ecx = logged.len as u32;
        }
        None => r.ecx = level_m1.wrapping_add(1) as u32,
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { write_line_regs(r) };
}

// ---- self-test: appends lines to the real var\logfile.txt ----

type Impl = unsafe fn(&mut Regs);
unsafe fn original_regs(r: &mut Regs) {
    unsafe { call_original(r) }
}

/// Takes the lock the way `log_lock_acquire` does.
fn acquire() {
    while lock().swap(1, Ordering::Acquire) != 0 {
        unsafe { Sleep(1) }
    }
}
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

/// Takes the lock as a log function would, sets the line state, runs `f` (the tail releases the
/// lock). The file offset is measured under the lock, so our bytes at [before, before+len)
/// cannot be disturbed by other logging threads.
/// Returns (registers after, file length before, line-start flag after).
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
        game_write(LOG_LINE_START_RVA, start);
        game_write(LOG_INDENT_RVA, indent as u32);
    }
    let before = file_len();
    let mut r = Regs {
        eax: msg as u32,
        edx: (dh as u32) << 8 | dl as u32,
        ..Regs::SENTINEL
    };
    unsafe { f(&mut r) };
    // may race with another logging thread
    (r, before, unsafe { game_read(LOG_LINE_START_RVA) })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let impls: [Impl; 2] = [original_regs, write_line_regs];
    let lvl: u32 = unsafe { game_read(LOG_LEVEL_RVA) };
    let dh_log = (lvl as u8).wrapping_sub(1); // level byte = lvl & 0xFF <= lvl, so it always logs
    acquire();
    let (saved_start, saved_indent): (u32, u32) =
        unsafe { (game_read(LOG_LINE_START_RVA), game_read(LOG_INDENT_RVA)) };
    lock().store(0, Ordering::Release);

    let long: Vec<u8> = std::iter::repeat_n(b'x', 1200).chain([0]).collect();
    // (message with NUL, newline, indent, line-start flag)
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
    let buf = game_ptr::<u8>(LINE_BUF_RVA) as u32;
    for (msg, newline, indent, start) in &msgs {
        let dl = *newline as u8;
        let text = &msg[..msg.len() - 1];
        let (want, want_flag) = format_line(text, *newline, *indent, *start);
        let regs_ok = |r: &Regs| {
            r.eax == want.len() as u32
                && r.ebx == buf
                && r.ecx == want.len() as u32
                && r.edx == dl as u32
                && r.preserved_except(&Regs::SENTINEL, EAX | EBX | ECX | EDX)
        };
        let mut ok = true;
        for f in impls {
            let (r, before, flag) = unsafe { run(f, msg.as_ptr(), dh_log, dl, *start, *indent) };
            ok &= file_bytes(before, want.len()) == want && regs_ok(&r) && flag == want_flag;
        }
        t.check(ok, || {
            format!(
                "{:?} (want len {})",
                String::from_utf8_lossy(&text[..text.len().min(40)]),
                want.len()
            )
        });
    }

    // Level gate: dh = lvl gives level lvl+1 > lvl, so nothing may be written.
    if lvl < 255 {
        let msg = b"rc2k_hook selftest: MUST NOT APPEAR\0";
        for f in impls {
            let (r, before, _) = unsafe { run(f, msg.as_ptr(), lvl as u8, 1, 0, 0) };
            let leaked = file_bytes(before, 1 << 16)
                .windows(msg.len() - 1)
                .any(|w| w == &msg[..msg.len() - 1]);
            let ok = !leaked
                && r.eax == msg.as_ptr() as u32
                && r.ecx == lvl + 1
                && r.edx == 1
                && r.ebx == Regs::SENTINEL.ebx;
            t.check(ok, || {
                format!(
                    "level gate (leaked={leaked}, eax={:#x}, ecx={})",
                    r.eax, r.ecx
                )
            });
        }
    }

    acquire();
    unsafe {
        game_write(LOG_LINE_START_RVA, saved_start);
        game_write(LOG_INDENT_RVA, saved_indent);
    }
    lock().store(0, Ordering::Release);
    t.finish()
}
