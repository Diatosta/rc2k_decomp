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
use crate::logfile::date_text::write_date_text;
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_int::log_int;
use crate::logfile::write_line::{LOG_PATH, level};
use crate::session::MACHINE as M;
use std::sync::atomic::{AtomicU32, Ordering};

// ---- game data ----

/// `aVerRally530482` (0x4E0000): the version's text.
pub const VERSION: &[u8] = b"$VER: Rally 5.30 (4.8.2000)\0";
/// `byte_4E0077`: a second text, logged when not empty (it is empty).
pub const SECOND_TEXT: &[u8] = b"\0";
/// `g_freeSpaceKb` (0x4E073A in the image until 2026-10-24): what the log's head says of
/// the current drive. Nothing reads it.
pub static FREE_SPACE_KB: AtomicU32 = AtomicU32::new(0);

// ---- Rust version ----

/// `sub_40317D` with eax = 0: the free space of the current drive in Kb, or 0xFFFFFFFF if
/// it cannot be asked or is 2^42 bytes or more. The original asks for the four numbers in
/// the scratch block at 0x93E8B0 (`g_pngInfo`, written before read by each of its users):
/// here locals.
pub fn free_space_kb() -> u32 {
    let Some([cluster_sectors, sector_bytes, free, _]) = crate::os::disk_free(None) else {
        return u32::MAX;
    };
    // The cluster's bytes as 32 bits, times the free clusters as 64.
    // Converted: `rc2k_core::files::log::free_kb`.
    rc2k_core::files::log::free_kb(cluster_sectors, sector_bytes, free)
}

fn text(text: &[u8], newline: bool) {
    debug_assert!(text.ends_with(&[0]));
    crate::logfile::log_write::log_nul(text, newline, 1);
}

/// The Rust version (see the header). Refuses, before doing anything, a start date the
/// date's text refuses.
pub fn start_log() -> Result<(), BadInput> {
    if level() == 0 {
        return Ok(());
    }
    crate::logfile::date_text::date_text()?;
    let _ = std::fs::remove_file(crate::files::host_path(&LOG_PATH[..LOG_PATH.len() - 1]));
    // Converted: `rc2k_core::files::log::head`; the values gathered (the date text,
    // the memory and the drive's numbers left in game memory as the original leaves
    // them), then each message logged as the original's call.
    use rc2k_core::files::log::{Machine, Message, head};
    let c_text = |p: crate::pointers::GamePtr| crate::files::cstring(p, "a start-up text");
    // The date in the numbers' scratch buffer, as the original makes it (the log's
    // lock is not held: no thread logs yet).
    let scratch = crate::pointers::GamePtr::to(&crate::logfile::numeric::SCRATCH);
    write_date_text(scratch)?;
    // `Buffer` (0x4E0558), written and read here only: a local.
    let memory = crate::os::memory();
    let free = free_space_kb();
    FREE_SPACE_KB.store(free, Ordering::Relaxed);
    let m = Machine {
        date: c_text(scratch)?,
        version: VERSION[..VERSION.len() - 1].to_vec(), // Rust's own text
        second: (SECOND_TEXT[0] != 0).then(|| SECOND_TEXT[..SECOND_TEXT.len() - 1].to_vec()),
        cpu_family: M.cpu_family.get() as i32,
        cpu_flags: M.cpu_signature.get(),
        // Twelve characters; the end follows the field.
        cpu_name: c_text(crate::pointers::GamePtr::to(&M.cpu_vendor).unbounded())?,
        clock: M.cpu_mhz.get() as i32,
        fpu: M.has_fpu.get() as i32,
        mmx: M.found_mmx.get() as i32,
        amd_3d: M.found_3dnow.get() as i32,
        katmai: M.found_sse.get() as i32,
        machine_type: M.keyboard_type.get() as i32,
        platform: [
            M.version[4].get() as i32,
            M.version[1].get() as i32,
            M.version[2].get() as i32,
            (M.version[3].get() & 0xFFFF) as i32,
        ],
        platform_text: c_text(crate::pointers::GamePtr::to(&M.version).offset(0x14))?,
        memory,
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

#[cfg(test)]
mod tests {
    use crate::moved::image_bytes;

    #[test]
    fn texts_are_the_games() {
        assert_eq!(image_bytes(0x4E0000, super::VERSION.len()), super::VERSION);
        assert_eq!(image_bytes(0x4E0077, 1), super::SECOND_TEXT);
    }
}
