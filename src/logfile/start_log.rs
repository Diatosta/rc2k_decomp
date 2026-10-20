//! `logfile::start_log` (0x401135): begins the game's log anew.
//!
//! In:  nothing
//! Out: every register preserved
//! Nothing if the log is off (`g_logLevel` 0). Else `var\logfile.txt` is deleted and the
//! head is written: "Log started" with `logfile::date_text`, the version, a second text if
//! there is one (`byte_4E0077`), then what `start` has found out by then: the processor
//! (`cpu=`, `clk=`, `fpu=`, `mmx=`, `amdk3d=`, `katmai=`), the keyboard type
//! (`machinetype=`), the Windows version (`platform=`), the memory (`GlobalMemoryStatus`
//! into `Buffer`, 0x4E0558) and the free space of the current drive in Kb (`sub_403175`
//! and `sub_40317D` with no drive given, here as `free_space_kb`; kept in `dword_4E073A`).
//! One caller, `start`.
//! Shadow mode runs the original alone (the file can be deleted and begun once).
//! Self-test, with the game running since it logs: a redirected copy that deletes
//! nothing; both versions add a head to the log, and what each added is compared but for
//! the two lines about memory, whose numbers move.

use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::logfile::date_text::write_date_text;
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::logfile::write_line::LOG_LEVEL_RVA;
use crate::menu::frame_test;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{capture, restore};
use crate::settings::TEXT_RVA;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Storage::FileSystem::{DeleteFileA, GetDiskFreeSpaceA};
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatus, MEMORYSTATUS};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("logfile::start_log", rva(0x401135), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const LOG_PATH_RVA: usize = rva(0x4E00A8); // "var\logfile.txt"
const VERSION_RVA: usize = rva(0x4E0000); // "$VER: Rally 5.30 (4.8.2000)"
const SECOND_TEXT_RVA: usize = rva(0x4E0077); // byte_4E0077
const SCRATCH_RVA: usize = rva(0x93E030); // dst
const CPU_FAMILY_RVA: usize = rva(0x5183D0); // dword_5183D0
const CPU_FLAGS_RVA: usize = rva(0x5183D4); // dword_5183D4
const CPU_NAME_RVA: usize = rva(0x5183D8); // dword_5183D8: a text
const CLOCK_RVA: usize = rva(0x5183E8); // dword_5183E8
const FPU_RVA: usize = rva(0x5183EC); // g_hasFpu
const MMX_RVA: usize = rva(0x5183F8); // dword_5183F8
const KATMAI_RVA: usize = rva(0x5183FC); // dword_5183FC
const AMD_3D_RVA: usize = rva(0x518400); // dword_518400
const KEYBOARD_TYPE_RVA: usize = rva(0x518404); // g_keyboardType
/// VersionInformation, an `OSVERSIONINFOA`: +4 major, +8 minor, +0xC build, +0x10 the
/// platform, +0x14 the service pack's text.
const VERSION_INFO_RVA: usize = rva(0x518408);
const MEMORY_RVA: usize = rva(0x4E0558); // Buffer, a `MEMORYSTATUS`
const FREE_SPACE_RVA: usize = rva(0x4E073A); // dword_4E073A
const DELETE_FILE: usize = rva(0x402E8C); // sub_402E8C

// ---- Rust version ----

/// `sub_40317D` with eax = 0: the free space of the current drive in Kb, or 0xFFFFFFFF if
/// it cannot be asked or is 2^42 bytes or more. The four numbers asked for are left in
/// the scratch block at 0x93E8B0, as the original leaves them.
pub unsafe fn free_space_kb() -> u32 {
    unsafe {
        let numbers = game_addr(TEXT_RVA) as *mut u32;
        let (total, free, sector_bytes, cluster_sectors) =
            (numbers, numbers.add(1), numbers.add(2), numbers.add(3));
        if GetDiskFreeSpaceA(std::ptr::null(), cluster_sectors, sector_bytes, free, total) == 0 {
            return u32::MAX;
        }
        // The cluster's bytes as 32 bits, times the free clusters as 64.
        let cluster_bytes = cluster_sectors
            .read_unaligned()
            .wrapping_mul(sector_bytes.read_unaligned());
        let bytes = cluster_bytes as u64 * free.read_unaligned() as u64;
        if bytes >> 42 != 0 {
            u32::MAX
        } else {
            (bytes >> 10) as u32
        }
    }
}

unsafe fn text(text: &[u8], newline: bool) {
    debug_assert!(text.ends_with(&[0]));
    unsafe { log_write(text.as_ptr(), newline, 1) };
}

/// `delete` stands for the deleting of the log's file. Refuses, before doing anything, a
/// start date the date's text refuses.
pub unsafe fn start_log(delete: &mut dyn FnMut()) -> Result<(), BadInput> {
    unsafe {
        if game_read::<u32>(LOG_LEVEL_RVA) == 0 {
            return Ok(());
        }
        crate::logfile::date_text::date_text()?;
        delete();
        text(b"Log started\0", false);
        let scratch = game_addr(SCRATCH_RVA) as *mut u8;
        write_date_text(scratch)?;
        log_write(scratch, true, 1);
        log_write(game_addr(VERSION_RVA) as *const u8, true, 1);
        if game_read::<u8>(SECOND_TEXT_RVA) != 0 {
            log_write(game_addr(SECOND_TEXT_RVA) as *const u8, true, 1);
        }
        text(b"\0", true);

        let number = |name: &[u8], at: usize| {
            text(name, false);
            log_int(game_read(at), true, 1);
        };
        text(b"cpu=\0", false);
        log_int(game_read(CPU_FAMILY_RVA), false, 1);
        log_hex8(game_read(CPU_FLAGS_RVA), false, 1);
        log_write(game_addr(CPU_NAME_RVA) as *const u8, true, 1);
        number(b"clk=\0", CLOCK_RVA);
        number(b"fpu=\0", FPU_RVA);
        number(b"mmx=\0", MMX_RVA);
        number(b"amdk3d=\0", AMD_3D_RVA);
        number(b"katmai=\0", KATMAI_RVA);
        number(b"machinetype=\0", KEYBOARD_TYPE_RVA);
        text(b"platform=\0", false);
        log_int(game_read(VERSION_INFO_RVA + 0x10), false, 1);
        log_int(game_read(VERSION_INFO_RVA + 4), false, 1);
        log_int(game_read(VERSION_INFO_RVA + 8), false, 1);
        log_int(game_read::<u16>(VERSION_INFO_RVA + 0xC) as i32, false, 1);
        log_write(game_addr(VERSION_INFO_RVA + 0x14) as *const u8, true, 1);

        let memory = game_addr(MEMORY_RVA) as *mut MEMORYSTATUS;
        (*memory).dwLength = 0x20;
        GlobalMemoryStatus(memory);
        let kb = |bytes: usize| log_int((bytes as u32 >> 10) as i32, false, 1);
        text(b"ram avail/total:\0", false);
        kb((*memory).dwAvailPhys);
        kb((*memory).dwTotalPhys);
        text(b"Kb\0", true);
        text(b"swp avail/total:\0", false);
        kb((*memory).dwAvailPageFile);
        kb((*memory).dwTotalPageFile);
        text(b"Kb\0", true);
        text(b"free space on partition=\0", false);
        let free = free_space_kb();
        game_write(FREE_SPACE_RVA, free);
        log_int(free as i32, false, 1);
        text(b"Kb\0", true);
        text(b"\0", true);
        text(b"\0", true);
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let done = unsafe {
        start_log(&mut || {
            DeleteFileA(game_addr(LOG_PATH_RVA) as *const u8);
        })
    };
    if let Err(e) = done {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

const LEN: usize = 0x1D5;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x83, 0x3D, 0x98, 0x00, 0x4E, 0x00, 0x00];
const CALLS: [usize; 44] = [
    0x14, 0x1F, 0x2C, 0x31, 0x3C, 0x4C, 0x51, 0x5C, 0x66, 0x70, 0x7B, 0x86, 0x90, 0x9B, 0xA5, 0xB0,
    0xBA, 0xC5, 0xCF, 0xDA, 0xE4, 0xEF, 0xF9, 0x104, 0x10E, 0x118, 0x122, 0x12F, 0x13A, 0x13F,
    0x14A, 0x157, 0x164, 0x16F, 0x17A, 0x187, 0x194, 0x19F, 0x1AA, 0x1AF, 0x1B9, 0x1C4, 0x1C9,
    0x1CE,
];
const LOG_FILE: &str = "var\\logfile.txt";

/// What a run added to the log, without the numbers of the two lines about memory.
fn added(from: usize) -> Option<Vec<String>> {
    let log = std::fs::read(LOG_FILE).ok()?;
    let lines = String::from_utf8_lossy(log.get(from..)?)
        .lines()
        .map(|line| match line.find("avail/total:") {
            Some(at) => line[..at].to_string(),
            None => line.to_string(),
        })
        .collect();
    Some(lines)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(LOG_LEVEL_RVA) } == 0 {
        return t.skip("the game's log is off");
    }
    // Inside a race the game turns `g_logEnabled` off round its steps: then neither
    // version writes the head (both runs add nothing), which says nothing about either.
    if unsafe { game_read::<i32>(rva(0x4E0094)) } <= 0 {
        return t.skip("the game's log is held off (g_logEnabled)");
    }
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x401135)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == DELETE_FILE).then(|| frame_test::stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let written = [
        (game_addr(MEMORY_RVA) as u32, 0x20),
        (game_addr(FREE_SPACE_RVA) as u32, 4),
        (game_addr(TEXT_RVA) as u32, 0x10),
    ];
    let saved = capture(&written);
    let before = Regs::SENTINEL;
    let run = |rust: bool| unsafe {
        let from = std::fs::metadata(LOG_FILE)
            .map(|file| file.len() as usize)
            .ok()?;
        game_write(FREE_SPACE_RVA, 0xAAAA_AAAAu32);
        frame_test::begin(u32::MAX, 0);
        let mut deletes = 0usize;
        let done = if rust {
            start_log(&mut || deletes += 1).is_ok()
        } else {
            let mut ro = before;
            call_game(&mut ro, copy.addr());
            deletes = frame_test::notes().len();
            ro.preserved_except(&before, 0)
        };
        Some((
            done,
            deletes,
            game_read::<u32>(FREE_SPACE_RVA),
            added(from)?,
        ))
    };
    let (theirs, mine) = (run(false), run(true));
    // Free space is the disk's, which other programs (and the game's threads) change between
    // the two runs: equal within 64 MB, and its line equal but for the number.
    let free_line = |line: &String| line.starts_with("free space on partition=");
    let same = match (&theirs, &mine) {
        (Some(o), Some(r)) => {
            o.0 == r.0
                && o.1 == r.1
                && o.2.abs_diff(r.2) < 0x1_0000
                && o.3.len() == r.3.len()
                && o.3
                    .iter()
                    .zip(&r.3)
                    .all(|(a, b)| a == b || (free_line(a) && free_line(b)))
        }
        _ => false,
    };
    t.check(
        same && theirs
            .as_ref()
            .is_some_and(|run| run.0 && run.1 == 1 && run.3.len() > 10),
        || format!("the log's head: original {theirs:?}; rust {mine:?}"),
    );
    restore(&written, &saved);
    t.finish()
}
