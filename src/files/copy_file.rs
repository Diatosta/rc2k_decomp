//! `files::copy_file` (0x402C28): a file copied, its time with it: the source opened to
//! read (`files::open_by_mode` 0), the destination created (mode 1), 64 KB at a time
//! through `g_pngInfo` (the PNG reader's buffer) until a read gives 0 bytes, then the
//! source's last write time (`sub_402F1C`: `GetFileTime`, `FileTimeToDosDateTime`) put on
//! the destination as its last access and last write time (`sub_402F4D`:
//! `DosDateTimeToFileTime`, `SetFileTime`; so to two seconds). A failed read, a short or
//! failed write or a failed time ends it; both files are closed (the destination first).
//! `sub_402F1C` and `sub_402F4D` are not hooked (each a Win32 call or two, as
//! `files/handle.rs`'s): `file_time` and `set_file_time` here.
//!
//! In:  eax = the source's path, ebx = the destination's (both through `resolve_path`)
//! Out: every register kept (`pusha` .. `popa`); carry clear if it was all done, set if
//!      not (zero set if done)
//!
//! Five callers: `install::run_after_exit`, `sub_4139E5`, `sub_41EA1F`, `sub_421B42`,
//! `sub_4599AD`. Refuses (before anything) what `open_by_mode` refuses (a resolved path
//! past its buffer). Shadow mode runs the original alone (files are written). Self-test:
//! a file of the test's in the temp folder copied by the original and by the Rust version
//! (the buffer put back), the two copies' bytes and times compared; a missing source.

use crate::errors::BadInput;
use crate::files::handle::{close, open_by_mode};
use crate::framework::{
    CARRY_FLAG, RawHook, Regs, SelfTest, ZERO_FLAG, game_addr, hook_asm, rva, shadow,
};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::Storage::FileSystem::{GetFileTime, SetFileTime};
use windows_sys::Win32::System::WindowsProgramming::{
    DosDateTimeToFileTime, FileTimeToDosDateTime,
};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::copy_file", rva(0x402C28), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// g_pngInfo: the 64 KB the copy goes through.
const BUFFER: usize = 0x93E8B0;
const CHUNK: u32 = 0x1_0000;

/// `sub_402F1C`: the file's last write time as DOS date << 16 | time, or None.
pub unsafe fn file_time(file: u32) -> Option<u32> {
    let mut ft = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let (mut date, mut time) = (0u16, 0u16);
    unsafe {
        if GetFileTime(
            file as _,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut ft,
        ) == 0
        {
            return None;
        }
        if FileTimeToDosDateTime(&ft, &mut date, &mut time) == 0 {
            return None;
        }
    }
    Some((date as u32) << 16 | time as u32)
}

/// `sub_402F4D`: a DOS date << 16 | time put on the file as its last access and write
/// time; whether it worked.
pub unsafe fn set_file_time(file: u32, dos: u32) -> bool {
    let mut ft = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    unsafe {
        if DosDateTimeToFileTime((dos >> 16) as u16, dos as u16, &mut ft) == 0 {
            return false;
        }
        SetFileTime(file as _, std::ptr::null(), &ft, &ft) != 0
    }
}

/// The Rust version: whether it was all done.
pub unsafe fn copy_file(from: *const u8, to: *const u8) -> Result<bool, BadInput> {
    // Converted: `rc2k_core::files::io::copy_file`, through the game's 64 KB block.
    unsafe {
        let buffer =
            std::slice::from_raw_parts_mut(game_addr(rva(BUFFER)) as *mut u8, CHUNK as usize);
        Ok(rc2k_core::files::io::copy_file(
            &mut crate::files::handle::GameFiles::new(),
            from,
            to,
            buffer,
        )?)
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { copy_file(r.eax as *const u8, r.ebx as *const u8) } {
        Ok(done) => {
            r.eflags =
                r.eflags & !(CARRY_FLAG | ZERO_FLAG) | if done { ZERO_FLAG } else { CARRY_FLAG };
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

fn temp_path(name: &str) -> Option<String> {
    use windows_sys::Win32::Storage::FileSystem::GetTempPathA;
    let mut dir = [0u8; 260];
    let n = unsafe { GetTempPathA(dir.len() as u32, dir.as_mut_ptr()) } as usize;
    (n > 0 && n < dir.len() && dir[1] == b':')
        .then(|| format!("{}{name}", String::from_utf8_lossy(&dir[..n])))
}

fn c(path: &str) -> Vec<u8> {
    let mut v = path.as_bytes().to_vec();
    v.push(0);
    v
}

/// The last write time of the file at the path, or None.
fn write_time(path: &[u8]) -> Option<u64> {
    let file = unsafe { open_by_mode(path.as_ptr(), 0) }.ok()??;
    let mut ft = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let ok = unsafe {
        GetFileTime(
            file as _,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut ft,
        )
    };
    unsafe { close(file) };
    (ok != 0).then_some((ft.dwHighDateTime as u64) << 32 | ft.dwLowDateTime as u64)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let (Some(source), Some(mine), Some(theirs), Some(missing)) = (
        temp_path("rc2k_copy_source.tmp"),
        temp_path("rc2k_copy_rust.tmp"),
        temp_path("rc2k_copy_original.tmp"),
        temp_path("rc2k_copy_missing.tmp"),
    ) else {
        return t.skip("no temp folder with a drive letter");
    };
    let buffer = game_addr(rva(BUFFER));
    let saved = unsafe { std::slice::from_raw_parts(buffer as *const u8, CHUNK as usize) }.to_vec();
    let last_file: u32 =
        unsafe { crate::framework::game_read(crate::files::read_file::LAST_FILE_PATH_RVA) };
    for len in [0usize, 5, 0x1_0000, 0x1_2345] {
        let bytes: Vec<u8> = (0..len).map(|n| (n * 31 + len) as u8).collect();
        if std::fs::write(&source, &bytes).is_err() {
            return t.skip("the temp folder cannot be written");
        }
        let (s, m, o) = (c(&source), c(&mine), c(&theirs));
        let done = unsafe { copy_file(s.as_ptr(), m.as_ptr()) };
        let mut r = Regs {
            eax: s.as_ptr() as u32,
            ebx: o.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let before = r;
        unsafe { call_original(&mut r) };
        let same_bytes = std::fs::read(&mine).ok() == std::fs::read(&theirs).ok()
            && std::fs::read(&mine).ok().as_deref() == Some(&bytes[..]);
        let times = (write_time(&m), write_time(&o));
        t.check(
            done == Ok(true)
                && !r.carry()
                && r.preserved_except(&before, 0)
                && same_bytes
                && times.0.is_some()
                && times.0 == times.1,
            || format!("{len} bytes: rust {done:?}, original carry {}, bytes same {same_bytes}, times {times:x?}", r.carry()),
        );
    }
    // A missing source: nothing done, carry set.
    let (s, m) = (c(&missing), c(&mine));
    let _ = std::fs::remove_file(&missing);
    let done = unsafe { copy_file(s.as_ptr(), m.as_ptr()) };
    let mut r = Regs {
        eax: s.as_ptr() as u32,
        ebx: m.as_ptr() as u32,
        ..Regs::SENTINEL
    };
    unsafe { call_original(&mut r) };
    t.check(done == Ok(false) && r.carry(), || {
        format!(
            "a missing source: rust {done:?}, original carry {}",
            r.carry()
        )
    });
    for path in [&source, &mine, &theirs] {
        let _ = std::fs::remove_file(path);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(saved.as_ptr(), buffer as *mut u8, saved.len());
        crate::framework::game_write(crate::files::read_file::LAST_FILE_PATH_RVA, last_file);
    }
    t.finish()
}
