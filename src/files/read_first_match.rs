//! `files::read_first_match` (0x402F82): reads from the first file that matches a path with
//! wildcards.
//!
//! In:  eax = the path with wildcards, ebx = buffer, ecx = count, edx = offset in the file
//! Out: nothing matched: eax = 0, carry set, zero clear; else as `files::read_file` leaves
//!      them: carry = failed (zero = ok), eax = bytes read if the read was reached, else
//!      the address of the path read from; everything else preserved
//! The path is resolved and `FindFirstFileA` finds a file, going on with `FindNextFileA`
//! past anything with the attributes of `dword_4E06E6` (0x10: directories). The name found
//! is put after the directory part of the path as it was given (not as resolved) in the
//! scratch block at 0x93E8B0, `read_file` reads that, and the search is closed. Like the
//! original it works in the game's own search record (`FindFileData`, 0x4E05A8) and handle
//! (`hFindFile`, 0x4E05A4).
//! Three call sites: the language's loading and the lexicon compiler.
//! Manual patch with a private copy (0x40 bytes, seven calls relocated): a call starts in
//! the first five bytes.
//! Shadow mode searches and reads twice and compares. Self-test: files and a directory of
//! its own in the game's `var` folder.

use crate::errors::BadInput;
use crate::files::read_file::{LAST_FILE_PATH_RVA, ReadResult, read_file};
use crate::files::resolve_path::resolve_path_in_game;
use crate::files::stream::LastPath;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Storage::FileSystem::{
    FindClose, FindFirstFileA, FindNextFileA, WIN32_FIND_DATAA,
};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("files::read_first_match", rva(0x402F82), thunk, &ORIGINAL)
    .private_copy(0x40)
    .relocate_calls(&[0x04, 0x09, 0x17, 0x22, 0x27, 0x2F, 0x39])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const FIND_DATA_RVA: usize = rva(0x4E05A8); // FindFileData
const FIND_HANDLE_RVA: usize = rva(0x4E05A4); // hFindFile
/// The attributes a match must not have: the dword 0x13E into the search record.
const SKIPPED_ATTRIBUTES_RVA: usize = rva(0x4E06E6);
/// Where the found file's path is put together (the block the PNG reader also uses).
const PATH_SCRATCH_RVA: usize = rva(0x93E8B0);
const NO_SEARCH: u32 = u32::MAX;

// ---- Rust version ----

/// None if nothing matched, else what the reading of the file found came to.
/// Refuses what `resolve_path` and `read_file` refuse, and a null path.
pub unsafe fn first_match_read(
    pattern: *const u8,
    buffer: *mut u8,
    count: u32,
    offset: u32,
) -> Result<Option<ReadResult>, BadInput> {
    unsafe {
        if pattern.is_null() {
            return Err(BadInput("no path to match"));
        }
        let resolved = resolve_path_in_game(pattern)?;
        game_write(LAST_FILE_PATH_RVA, resolved as u32);
        let record = game_addr(FIND_DATA_RVA) as *mut WIN32_FIND_DATAA;
        let search = FindFirstFileA(resolved, record);
        game_write(FIND_HANDLE_RVA, search as u32);
        let skipped =
            || game_read::<u32>(FIND_DATA_RVA) & game_read::<u32>(SKIPPED_ATTRIBUTES_RVA) != 0;
        let mut found = search as u32 != NO_SEARCH;
        if found && skipped() {
            loop {
                if FindNextFileA(search, record) == 0 {
                    found = false;
                    break;
                }
                if !skipped() {
                    break;
                }
            }
        }
        let read = if found {
            // The pattern up to and with its last `/`, `\` or `:` (one at its very start
            // does not count), then the name found.
            let pattern = std::ffi::CStr::from_ptr(pattern.cast()).to_bytes();
            let directory = (1..pattern.len())
                .rev()
                .find(|&n| matches!(pattern[n], b'/' | b'\\' | b':'))
                .map_or(0, |n| n + 1);
            let name = std::ffi::CStr::from_ptr((&raw const (*record).cFileName).cast()).to_bytes();
            let path = game_addr(PATH_SCRATCH_RVA) as *mut u8;
            std::ptr::copy(pattern.as_ptr(), path, directory);
            std::ptr::copy(name.as_ptr(), path.add(directory), name.len());
            path.add(directory + name.len()).write(0);
            Some(read_file(path, buffer, count, offset))
        } else {
            None
        };
        if game_read::<u32>(FIND_HANDLE_RVA) != NO_SEARCH {
            FindClose(game_read::<u32>(FIND_HANDLE_RVA) as _);
            game_write(FIND_HANDLE_RVA, NO_SEARCH);
        }
        read.transpose()
    }
}

/// For Rust callers: the bytes read, or None if nothing matched or the reading failed.
pub unsafe fn read_first_match(
    pattern: *const u8,
    buffer: *mut u8,
    count: u32,
    offset: u32,
) -> Result<Option<u32>, BadInput> {
    let read = unsafe { first_match_read(pattern, buffer, count, offset) }?;
    Ok(read.filter(|read| read.ok).and_then(|read| read.bytes_read))
}

// ---- hook ----

/// (eax, carry) as the original leaves them.
fn left(read: Option<ReadResult>) -> (u32, bool) {
    match read {
        None => (0, true),
        Some(read) => (
            read.bytes_read
                .unwrap_or(game_addr(PATH_SCRATCH_RVA) as u32),
            !read.ok,
        ),
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (pattern, buffer, count, offset) = (r.eax as *const u8, r.ebx as *mut u8, r.ecx, r.edx);
    let mine = match unsafe { first_match_read(pattern, buffer, count, offset) } {
        Ok(mine) => mine,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    let (eax, carry) = left(mine);
    if !shadow() {
        r.eax = eax;
        r.set_carry(carry);
        r.set_zero(!carry);
        return;
    }
    let got = mine
        .and_then(|read| read.bytes_read)
        .unwrap_or(0)
        .min(count) as usize;
    let my_bytes = unsafe { std::slice::from_raw_parts(buffer, got) }.to_vec();
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let same_bytes = unsafe { std::slice::from_raw_parts(buffer, got) } == &my_bytes[..];
    if (o.eax, o.carry()) != (eax, carry) || !same_bytes || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "path={:?} count={count:#x} offset={offset}: original eax={:#x} carry={}; rust eax={eax:#x} carry={carry} same_bytes={same_bytes}",
                unsafe { std::ffi::CStr::from_ptr(pattern.cast()) },
                o.eax,
                o.carry()
            )
        });
    }
    r.eax = o.eax;
    r.eflags = o.eflags;
}

// ---- self-test: files and a directory of its own in the game's `var` folder ----

const TEST_FILES: [(&str, &[u8]); 2] = [
    ("var\\rc2k_hook_match_one.bin", b"the first and only match"),
    (
        "var\\rc2k_hook_matchd_file.bin",
        b"a file beside a directory",
    ),
];
const TEST_DIRECTORIES: [&str; 2] = ["var\\rc2k_hook_matchd_dir.bin", "var\\rc2k_hook_matcho_dir"];
/// One match; a directory and a file; a directory alone; nothing; no wildcard.
const PATTERNS: [&[u8]; 5] = [
    b"var\\rc2k_hook_match_*.bin\0",
    b"var\\rc2k_hook_matchd_*.bin\0",
    b"var\\rc2k_hook_matcho_*\0",
    b"var\\rc2k_hook_matchn_*.bin\0",
    b"var\\rc2k_hook_match_one.bin\0",
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let made = TEST_FILES
        .iter()
        .all(|(path, bytes)| std::fs::write(path, bytes).is_ok())
        && TEST_DIRECTORIES
            .iter()
            .all(|path| std::fs::create_dir(path).is_ok() || std::path::Path::new(path).is_dir());
    let state: [Region; 3] = [
        (game_addr(FIND_DATA_RVA) as u32, 0x140),
        (game_addr(FIND_HANDLE_RVA) as u32, 4),
        (game_addr(PATH_SCRATCH_RVA) as u32, 0x200),
    ];
    let saved = capture(&state);
    // Of the search record only the attributes and the name: the times may move.
    let compared: [Region; 4] = [
        (game_addr(FIND_DATA_RVA) as u32, 4),
        (game_addr(FIND_DATA_RVA) as u32 + 0x2C, 0x104),
        state[1],
        state[2],
    ];
    let last = LastPath::save();
    if made {
        for pattern in PATTERNS {
            for (count, offset) in [(0x40u32, 0u32), (5, 4)] {
                let mut buffer = [0u8; 0x40];
                let before = Regs {
                    eax: pattern.as_ptr() as u32,
                    ebx: buffer.as_mut_ptr() as u32,
                    ecx: count,
                    edx: offset,
                    ..Regs::SENTINEL
                };
                let mut run = |rust: bool| unsafe {
                    buffer.fill(0xAA);
                    restore(&state, &vec![0xAA; saved.len()]);
                    game_write(FIND_HANDLE_RVA, NO_SEARCH);
                    let result = if rust {
                        first_match_read(pattern.as_ptr(), buffer.as_mut_ptr(), count, offset)
                            .ok()
                            .map(|read| {
                                let (eax, carry) = left(read);
                                (eax, carry, !carry)
                            })
                    } else {
                        let mut ro = before;
                        call_original(&mut ro);
                        ro.preserved_except(&before, EAX).then_some((
                            ro.eax,
                            ro.carry(),
                            ro.eflags & 0x40 != 0,
                        ))
                    };
                    (result, buffer, capture(&compared), LastPath::now())
                };
                let (theirs, mine) = (run(false), run(true));
                t.check(theirs == mine && theirs.0.is_some(), || {
                    format!(
                        "pattern={:?} count={count:#x} offset={offset}: original {:x?} read {:?} last={:#x}; rust {:x?} read {:?} last={:#x}; same state={}",
                        String::from_utf8_lossy(pattern),
                        theirs.0,
                        crate::strings::utils::bytes_str(&theirs.1),
                        theirs.3,
                        mine.0,
                        crate::strings::utils::bytes_str(&mine.1),
                        mine.3,
                        theirs.2 == mine.2
                    )
                });
            }
        }
    }
    restore(&state, &saved);
    last.restore();
    for (path, _) in TEST_FILES {
        let _ = std::fs::remove_file(path);
    }
    for path in TEST_DIRECTORIES {
        let _ = std::fs::remove_dir(path);
    }
    if !made {
        return t.skip("cannot create the test files in the game's var folder");
    }
    t.finish()
}
