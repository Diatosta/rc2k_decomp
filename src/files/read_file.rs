//! `files::read_file` (0x402BB6): opens a file for reading, seeks if the offset is not 0,
//! reads, closes.
//!
//! In:  eax = path, ebx = buffer, ecx = bytes to read, edx = offset
//! Out: carry = failed (zero flag = succeeded); eax = bytes read if the read was reached,
//!      else unchanged; everything else preserved
//! It fails if the open, the seek, the read or the close fails. A short read is not a failure:
//! the caller has to look at eax. The file is opened as the original's mode 0 says (table at
//! 0x4E06EA). Like the original's `open_by_mode` and `close`, which are inlined here, it sets
//! `g_lastFilePath` to the resolved path while the file is open and to 0 after closing; after
//! a failed open the path stays.
//! Shadow mode reads the file twice, into the same buffer, and compares.

use crate::errors::BadInput;
use crate::files::resolve_path::resolve_path_in_game;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow,
};
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{CreateFileA, ReadFile, SetFilePointer};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::read_file", rva(0x402BB6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const LAST_FILE_PATH_RVA: usize = rva(0x4F52B0); // g_lastFilePath
// Mode 0 of the file mode table (16 bytes per mode).
const READ_ACCESS_RVA: usize = rva(0x4E06EA); // dword_4E06EA[0]
const READ_DISPOSITION_RVA: usize = rva(0x4E06EE); // dword_4E06EE[0]
const READ_SHARE_RVA: usize = rva(0x4E06F2); // dword_4E06F2[0]

// ---- Rust version ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadResult {
    pub ok: bool,
    /// None if the read was not reached (the open or the seek failed).
    pub bytes_read: Option<u32>,
}

/// Refuses what `resolve_path` refuses.
pub unsafe fn read_file(
    path: *const u8,
    buffer: *mut u8,
    count: u32,
    offset: u32,
) -> Result<ReadResult, BadInput> {
    unsafe {
        let resolved = resolve_path_in_game(path)?;
        game_write(LAST_FILE_PATH_RVA, resolved as u32);
        let file = CreateFileA(
            resolved,
            game_read(READ_ACCESS_RVA),
            game_read(READ_SHARE_RVA),
            std::ptr::null(),
            game_read(READ_DISPOSITION_RVA),
            0x80,
            0 as _,
        );
        if file == INVALID_HANDLE_VALUE {
            return Ok(ReadResult {
                ok: false,
                bytes_read: None,
            });
        }
        let mut bytes_read = None;
        let mut read_ok = false;
        // 0 = FILE_BEGIN
        if offset == 0 || SetFilePointer(file, offset as i32, std::ptr::null_mut(), 0) != u32::MAX {
            let mut n = 0u32;
            read_ok = ReadFile(file, buffer.cast(), count, &mut n, std::ptr::null_mut()) != 0;
            bytes_read = Some(n);
        }
        game_write(LAST_FILE_PATH_RVA, 0u32);
        let closed = CloseHandle(file) != 0;
        Ok(ReadResult {
            ok: closed && read_ok,
            bytes_read,
        })
    }
}

// ---- hook ----

fn apply(result: ReadResult, r: &mut Regs) {
    if let Some(n) = result.bytes_read {
        r.eax = n;
    }
    r.set_carry(!result.ok);
    r.set_zero(result.ok);
}

fn result_of(before: &Regs, after: &Regs) -> ReadResult {
    ReadResult {
        ok: !after.carry(),
        // Unchanged eax is read as "not reached"; a path pointer is never a byte count.
        bytes_read: (after.eax != before.eax).then_some(after.eax),
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (path, buffer, count, offset) = (r.eax as *const u8, r.ebx as *mut u8, r.ecx, r.edx);
    let Ok(mine) = (unsafe { read_file(path, buffer, count, offset) }) else {
        unsafe { call_original(r) };
        return;
    };
    if !shadow() {
        apply(mine, r);
        return;
    }

    let got = mine.bytes_read.unwrap_or(0).min(count) as usize;
    let my_bytes = unsafe { from_raw_parts(buffer, got) }.to_vec();
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = result_of(r, &o);
    let same_bytes = theirs.bytes_read == mine.bytes_read
        && unsafe { from_raw_parts(buffer, got) } == &my_bytes[..];
    if theirs != mine || !same_bytes || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "path={:?} count={count} offset={offset}: original {theirs:?} rust {mine:?} same_bytes={same_bytes}",
                unsafe { std::ffi::CStr::from_ptr(path.cast()) }
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

// ---- self-test: a file of its own in the game's `var` folder, removed afterwards ----

const TEST_FILE: &str = "var\\rc2k_hook_selftest.bin";
const TEST_PATH: &[u8] = b"var\\rc2k_hook_selftest.bin\0";
const MISSING_PATH: &[u8] = b"var\\rc2k_hook_no_such_file.bin\0";

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let content: Vec<u8> = (0..300u32).map(|i| (i * 7 + 3) as u8).collect();
    if std::fs::write(TEST_FILE, &content).is_err() {
        return t.skip("cannot create a test file in the game's var folder");
    }
    let saved_last: u32 = unsafe { game_read(LAST_FILE_PATH_RVA) };

    for (path, count, offset) in [
        (TEST_PATH, 0x48u32, 0u32),
        (TEST_PATH, 300, 0),
        (TEST_PATH, 400, 0), // short read
        (TEST_PATH, 20, 10),
        (TEST_PATH, 50, 280),  // short read after a seek
        (TEST_PATH, 10, 300),  // at the end
        (TEST_PATH, 10, 5000), // past the end
        (TEST_PATH, 0, 0),
        (MISSING_PATH, 16, 0),
        (MISSING_PATH, 16, 8),
    ] {
        let mut theirs = [0xAAu8; 512];
        let mut ours = [0xAAu8; 512];
        let before = Regs {
            eax: path.as_ptr() as u32,
            ebx: theirs.as_mut_ptr() as u32,
            ecx: count,
            edx: offset,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let last_o: u32 = unsafe { game_read(LAST_FILE_PATH_RVA) };
        let result_o = result_of(&before, &ro);
        let zero_o = ro.eflags & 0x40 != 0;

        let mine = unsafe { read_file(path.as_ptr(), ours.as_mut_ptr(), count, offset) };
        let last_r: u32 = unsafe { game_read(LAST_FILE_PATH_RVA) };

        let ok = mine == Ok(result_o)
            && zero_o == result_o.ok
            && theirs == ours
            && last_o == last_r
            && ro.preserved_except(&before, EAX);
        t.check(ok, || {
            format!(
                "path={:?} count={count} offset={offset}: original {result_o:?} zero={zero_o} last={last_o:#x}; rust {mine:?} last={last_r:#x} same_bytes={}",
                String::from_utf8_lossy(&path[..path.len() - 1]),
                theirs == ours
            )
        });
    }

    unsafe { game_write(LAST_FILE_PATH_RVA, saved_last) };
    let _ = std::fs::remove_file(TEST_FILE);
    t.finish()
}
