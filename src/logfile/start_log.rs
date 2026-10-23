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
//! One caller, `start`, which calls it directly (2026-10-23).
//! Shadow mode runs the original alone (the file can be deleted and begun once).
//! Self-test, with the game running since it logs: a redirected copy that deletes
//! nothing; both versions add a head to the log, and what each added is compared but for
//! the two lines about memory, whose numbers move.

use crate::errors::BadInput;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::logfile::date_text::write_date_text;
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::logfile::write_line::LOG_LEVEL_RVA;
use crate::settings::TEXT_RVA;
use windows_sys::Win32::Storage::FileSystem::{DeleteFileA, GetDiskFreeSpaceA};
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatus, MEMORYSTATUS};

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
// sub_402E8C

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
        // Converted: `rc2k_core::files::log::free_kb`.
        rc2k_core::files::log::free_kb(
            cluster_sectors.read_unaligned(),
            sector_bytes.read_unaligned(),
            free.read_unaligned(),
        )
    }
}

unsafe fn text(text: &[u8], newline: bool) {
    debug_assert!(text.ends_with(&[0]));
    unsafe { log_write(text.as_ptr(), newline, 1) };
}

/// The Rust version (see the header). Refuses, before doing anything, a start date the
/// date's text refuses.
pub unsafe fn start_log() -> Result<(), BadInput> {
    unsafe {
        if game_read::<u32>(LOG_LEVEL_RVA) == 0 {
            return Ok(());
        }
        crate::logfile::date_text::date_text()?;
        DeleteFileA(game_addr(LOG_PATH_RVA) as *const u8);
        // Converted: `rc2k_core::files::log::head`; the values gathered (the date text,
        // the memory and the drive's numbers left in game memory as the original leaves
        // them), then each message logged as the original's call.
        use rc2k_core::files::log::{Machine, Message, head};
        let c_text = |p: *const u8| {
            std::ffi::CStr::from_ptr(p as *const std::ffi::c_char)
                .to_bytes()
                .to_vec()
        };
        let scratch = game_addr(SCRATCH_RVA) as *mut u8;
        write_date_text(scratch)?;
        let memory = game_addr(MEMORY_RVA) as *mut MEMORYSTATUS;
        (*memory).dwLength = 0x20;
        GlobalMemoryStatus(memory);
        let free = free_space_kb();
        game_write(FREE_SPACE_RVA, free);
        let m = Machine {
            date: c_text(scratch),
            version: c_text(game_addr(VERSION_RVA) as *const u8),
            second: (game_read::<u8>(SECOND_TEXT_RVA) != 0)
                .then(|| c_text(game_addr(SECOND_TEXT_RVA) as *const u8)),
            cpu_family: game_read(CPU_FAMILY_RVA),
            cpu_flags: game_read(CPU_FLAGS_RVA),
            cpu_name: c_text(game_addr(CPU_NAME_RVA) as *const u8),
            clock: game_read(CLOCK_RVA),
            fpu: game_read(FPU_RVA),
            mmx: game_read(MMX_RVA),
            amd_3d: game_read(AMD_3D_RVA),
            katmai: game_read(KATMAI_RVA),
            machine_type: game_read(KEYBOARD_TYPE_RVA),
            platform: [
                game_read(VERSION_INFO_RVA + 0x10),
                game_read(VERSION_INFO_RVA + 4),
                game_read(VERSION_INFO_RVA + 8),
                game_read::<u16>(VERSION_INFO_RVA + 0xC) as i32,
            ],
            platform_text: c_text(game_addr(VERSION_INFO_RVA + 0x14) as *const u8),
            memory: [
                (*memory).dwAvailPhys as u32,
                (*memory).dwTotalPhys as u32,
                (*memory).dwAvailPageFile as u32,
                (*memory).dwTotalPageFile as u32,
            ],
            free_kb: free,
        };
        for (message, newline) in head(&m) {
            match message {
                Message::Text(mut t) => {
                    t.push(0);
                    text(&t, newline);
                }
                Message::Int(v) => log_int(v, newline, 1),
                Message::Hex8(v) => log_hex8(v, newline, 1),
            }
        }
        Ok(())
    }
}
