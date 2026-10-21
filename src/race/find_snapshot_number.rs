//! `race::find_snapshot_number` (0x41EF51): the viewer's pictures' name
//! (`savedata\snapshot\000.pcx`) given the highest number of the pictures there: its
//! digits made `???` and searched for (`files::find_first`, `files::find_next_wanted`,
//! `files::close_search`: `FindFirstFileA` and on in the game's search record
//! `FindFileData` and handle `hFindFile`, the path not resolved, `g_lastFilePath` the
//! pattern); of each name found whose first three characters are digits the number, the
//! highest (0 for none) written back over the `???` (`strings::format_u32`, flags 0x13).
//! `race::viewer_snapshot` counts on from it.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::replay_viewer`. Shadow mode compares (the search only reads the
//! folder). Self-test: the original against the Rust version on the folder as it is.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::a;
use crate::race::viewer::{Saved, check_original, dispatch_regs, get, put, region};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Storage::FileSystem::{
    FindClose, FindFirstFileA, FindNextFileA, WIN32_FIND_DATAA,
};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::find_snapshot_number",
    rva(0x41EF51),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const SNAPSHOT_PATH: usize = 0x529731; // "savedata\snapshot\000.pcx"
pub const SNAPSHOT_DIGITS: usize = SNAPSHOT_PATH + 0x12;
const FIND_DATA: usize = 0x4E05A8; // FindFileData
const FIND_HANDLE: usize = 0x4E05A4; // hFindFile
const SKIPPED: usize = 0x4E06E6; // the attributes a search passes over
const LAST_PATH: usize = 0x4F52B0; // g_lastFilePath
const NO_SEARCH: u32 = u32::MAX;

/// `files::find_next_wanted` (0x402FF6): the next name not of the skipped attributes;
/// false (carry) when there is none.
pub unsafe fn find_next_wanted() -> bool {
    let record = a(FIND_DATA) as *mut WIN32_FIND_DATAA;
    loop {
        if unsafe { FindNextFileA(get(FIND_HANDLE) as _, record) } == 0 {
            return false;
        }
        if get(FIND_DATA) & get(SKIPPED) == 0 {
            return true;
        }
    }
}

/// `files::find_first` (0x402FC2): the first name matching `pattern` (not resolved; it
/// becomes `g_lastFilePath`) not of the skipped attributes; false (carry) when none.
pub unsafe fn find_first(pattern: u32) -> bool {
    put(LAST_PATH, pattern);
    let record = a(FIND_DATA) as *mut WIN32_FIND_DATAA;
    let search = unsafe { FindFirstFileA(pattern as *const u8, record) } as u32;
    put(FIND_HANDLE, search);
    if search == NO_SEARCH {
        return false;
    }
    if get(FIND_DATA) & get(SKIPPED) != 0 {
        return unsafe { find_next_wanted() };
    }
    true
}

/// `files::close_search` (0x40301F): the search closed, if there is one.
pub unsafe fn close_search() {
    if get(FIND_HANDLE) != NO_SEARCH {
        unsafe { FindClose(get(FIND_HANDLE) as _) };
        put(FIND_HANDLE, NO_SEARCH);
    }
}

/// The Rust version.
pub unsafe fn find_snapshot_number() {
    unsafe {
        (a(SNAPSHOT_DIGITS) as *mut u32).write_unaligned(u32::from_le_bytes(*b"???."));
        let mut highest = 0u32;
        if find_first(a(SNAPSHOT_PATH)) {
            loop {
                // Converted: `rc2k_core::race::viewer::snapshot_number`.
                let name = &(*(a(FIND_DATA) as *const WIN32_FIND_DATAA)).cFileName;
                if let Some(number) = rc2k_core::race::viewer::snapshot_number(|n| name[n] as u8)
                    && number > highest
                {
                    highest = number;
                }
                if !find_next_wanted() {
                    break;
                }
            }
        }
        close_search();
        let digits = crate::strings::format_u32::format_u32(highest, 0x13);
        std::ptr::copy_nonoverlapping(digits.as_ptr(), a(SNAPSHOT_DIGITS) as *mut u8, digits.len());
    }
}

fn regions() -> Vec<Region> {
    vec![
        region(SNAPSHOT_PATH, 0x1A),
        region(FIND_DATA, 0x140),
        region(FIND_HANDLE, 4),
        region(LAST_PATH, 4),
    ]
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    unsafe { find_snapshot_number() };
    Ok(*r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &regions(), 0, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = Saved::new(&regions());
    check_original(
        &mut t,
        "the snapshot folder as it is",
        call_original,
        Regs::SENTINEL,
        &regions(),
        0,
        &|| {
            put(FIND_HANDLE, NO_SEARCH);
            crate::race::viewer::fill(FIND_DATA, 0x140, 13);
        },
        &rust,
    );
    drop(saved);
    t.finish()
}
