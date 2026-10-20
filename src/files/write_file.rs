//! `files::write_file` (0x402BFB): creates a file (replacing one that exists), writes a
//! buffer to it, closes it.
//!
//! In:  eax = path, ebx = buffer, ecx = bytes to write
//! Out: carry = failed (zero flag = succeeded); eax = bytes written if the write was reached,
//!      else unchanged; everything else preserved
//! It fails if the open fails, if the write fails or writes less than asked, or if the close
//! fails. It does not create missing folders. The file is opened as the original's mode 1
//! says (table at 0x4E06EA). `open_by_mode`, `write` and `close` are inlined, with their
//! handling of `g_lastFilePath`.
//! Shadow mode writes the file twice, with the same bytes, and compares the results.

use crate::errors::BadInput;
use crate::files::read_file::LAST_FILE_PATH_RVA;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::write_file", rva(0x402BFB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

// Mode 1 of the file mode table (16 bytes per mode).

// ---- Rust version ----

use crate::files::handle::GameFiles;
pub use rc2k_core::files::io::WriteResult;

/// Refuses what `resolve_path` refuses.
pub unsafe fn write_file(
    path: *const u8,
    buffer: *const u8,
    count: u32,
) -> Result<WriteResult, BadInput> {
    // Converted: `rc2k_core::files::io::write_file`.
    unsafe {
        let data = std::slice::from_raw_parts(buffer, count as usize);
        Ok(rc2k_core::files::io::write_file(
            &mut GameFiles::new(),
            path,
            data,
        )?)
    }
}

// ---- hook ----

fn result_of(before: &Regs, after: &Regs) -> WriteResult {
    WriteResult {
        ok: !after.carry(),
        // Unchanged eax is read as "not reached"; a path pointer is never a byte count.
        bytes_written: (after.eax != before.eax).then_some(after.eax),
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (path, buffer, count) = (r.eax as *const u8, r.ebx as *const u8, r.ecx);
    let Ok(mine) = (unsafe { write_file(path, buffer, count) }) else {
        unsafe { call_original(r) };
        return;
    };
    if !shadow() {
        if let Some(n) = mine.bytes_written {
            r.eax = n;
        }
        r.set_carry(!mine.ok);
        r.set_zero(mine.ok);
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = result_of(r, &o);
    if theirs != mine || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "path={:?} count={count}: original {theirs:?} rust {mine:?}",
                unsafe { std::ffi::CStr::from_ptr(path.cast()) }
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

// ---- self-test: files of its own in the game's `var` folder, removed afterwards ----

const FILE_O: &str = "var\\rc2k_hook_selftest_o.bin";
const PATH_O: &[u8] = b"var\\rc2k_hook_selftest_o.bin\0";
const FILE_R: &str = "var\\rc2k_hook_selftest_r.bin";
const PATH_R: &[u8] = b"var\\rc2k_hook_selftest_r.bin\0";
const NO_FOLDER: &[u8] = b"var\\rc2k_hook_no_such_folder\\x.bin\0";

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if std::fs::write(FILE_O, b"x").is_err() {
        return t.skip("cannot create a test file in the game's var folder");
    }
    let saved_last: u32 = unsafe { game_read(LAST_FILE_PATH_RVA) };
    let content: Vec<u8> = (0..300u32).map(|i| (i * 5 + 1) as u8).collect();

    // Each count also shows that a longer file left by the case before is replaced.
    for (missing_folder, count) in [
        (false, 300u32),
        (false, 1),
        (false, 0),
        (false, 72),
        (true, 16),
    ] {
        let (path_o, path_r) = if missing_folder {
            (NO_FOLDER, NO_FOLDER)
        } else {
            (PATH_O, PATH_R)
        };
        let before = Regs {
            eax: path_o.as_ptr() as u32,
            ebx: content.as_ptr() as u32,
            ecx: count,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let last_o: u32 = unsafe { game_read(LAST_FILE_PATH_RVA) };
        let result_o = result_of(&before, &ro);
        let zero_o = ro.eflags & 0x40 != 0;

        let mine = unsafe { write_file(path_r.as_ptr(), content.as_ptr(), count) };
        let last_r: u32 = unsafe { game_read(LAST_FILE_PATH_RVA) };

        let (file_o, file_r) = (std::fs::read(FILE_O).ok(), std::fs::read(FILE_R).ok());
        let files_ok = missing_folder
            || (file_o.as_deref() == Some(&content[..count as usize]) && file_o == file_r);
        let ok = mine == Ok(result_o)
            && zero_o == result_o.ok
            && files_ok
            // After a failed open the path stays: both point into the same buffer or literal.
            && (last_o == last_r || !missing_folder)
            && (missing_folder || (last_o == 0 && last_r == 0))
            && ro.preserved_except(&before, EAX);
        t.check(ok, || {
            format!(
                "missing_folder={missing_folder} count={count}: original {result_o:?} zero={zero_o} last={last_o:#x} file={:?}; rust {mine:?} last={last_r:#x} file={:?}",
                file_o.as_ref().map(Vec::len),
                file_r.as_ref().map(Vec::len)
            )
        });
    }

    unsafe { game_write(LAST_FILE_PATH_RVA, saved_last) };
    let _ = std::fs::remove_file(FILE_O);
    let _ = std::fs::remove_file(FILE_R);
    t.finish()
}
