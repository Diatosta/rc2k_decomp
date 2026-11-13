//! Begins the game's log anew. Nothing if the log is off. Else `var\logfile.txt` is deleted and
//! the head is written: "Log started" with `logfile::date_text`, the version, a second text if
//! there is one (`SECOND_TEXT`), then what `start` has found out by then: the processor
//! (`cpu=`, `clk=`, `fpu=`, `mmx=`, `amdk3d=`, `katmai=`), the keyboard type (`machinetype=`),
//! the Windows version (`platform=`), the memory and the free space of the current drive in Kb
//! (`FREE_SPACE_KB`).

use crate::errors::BadInput;
use crate::logfile::date_text::write_date_text;
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_int::log_int;
use crate::logfile::write_line::{LOG_PATH, level};
use crate::session::PlainDword;
use crate::system::owner::System;

/// The version's text.
pub const VERSION: &[u8] = b"$VER: Rally 5.30 (4.8.2000)\0";
/// `StartTexts.second_text`: a second text, logged when not empty (it is empty).
pub const SECOND_TEXT: &[u8] = b"\0";
/// What the log's head says of the current drive. Nothing reads it.
pub static FREE_SPACE_KB: crate::session::GameThread<crate::session::PlainDword> =
    crate::session::GameThread::new(crate::session::PlainDword::new(0));

/// The free space of the current drive in Kb, or 0xFFFFFFFF if it cannot be asked or is 2^42
/// bytes or more.
pub fn free_space_kb() -> u32 {
    let Some([cluster_sectors, sector_bytes, free, _]) = crate::os::disk_free(None) else {
        return u32::MAX;
    };
    // The cluster's bytes as 32 bits, times the free clusters as 64.
    rc2k_core::files::log::free_kb(cluster_sectors, sector_bytes, free)
}

fn text(text: &[u8], newline: bool) {
    debug_assert!(text.ends_with(&[0]));
    crate::logfile::log_write::log_nul(text, newline, 1);
}

/// Refuses, before doing anything, a start date the date's text refuses.
pub fn start_log(system: &System) -> Result<(), BadInput> {
    if level() == 0 {
        return Ok(());
    }
    crate::logfile::date_text::date_text(system)?;
    let _ = std::fs::remove_file(crate::files::host_path(&LOG_PATH[..LOG_PATH.len() - 1]));
    use rc2k_core::files::log::{Machine, Message, head};
    let machine = system.machine();
    let le_bytes = |words: &[PlainDword]| -> Vec<u8> {
        words.iter().flat_map(|w| w.get().to_le_bytes()).collect()
    };
    let runs_bytes =
        |runs: &[&[PlainDword]]| -> Vec<u8> { runs.iter().flat_map(|run| le_bytes(run)).collect() };
    // The vendor's text is read on through the fields after it, to the record's end.
    let vendor_run = runs_bytes(&[
        &machine.cpu_vendor[..],
        core::slice::from_ref(&machine.gap_e4),
        core::slice::from_ref(&machine.cpu_mhz),
        core::slice::from_ref(&machine.has_tsc),
        core::slice::from_ref(&machine.has_cmov),
        core::slice::from_ref(&machine.found_mmx),
        core::slice::from_ref(&machine.found_sse),
        core::slice::from_ref(&machine.found_3dnow),
        core::slice::from_ref(&machine.keyboard_type),
        &machine.version[..],
        core::slice::from_ref(&machine.exception_resume),
        core::slice::from_ref(&machine.instance),
    ]);
    // The date in the numbers' scratch buffer (the log's lock is not held: no thread logs yet).
    let date = write_date_text(system, &system.work().numbers)?;
    // `Buffer`, written and read here only: a local.
    let memory = crate::os::memory();
    let free = free_space_kb();
    FREE_SPACE_KB.get().set(free);
    let m = Machine {
        date,
        version: VERSION[..VERSION.len() - 1].to_vec(), // Rust's own text
        second: (SECOND_TEXT[0] != 0).then(|| SECOND_TEXT[..SECOND_TEXT.len() - 1].to_vec()),
        cpu_family: system.machine().cpu_family.get() as i32,
        cpu_flags: system.machine().cpu_signature.get(),
        // Twelve characters; the end follows the field (read on to the record's end).
        cpu_name: crate::files::cstring_in(&vendor_run)?,
        clock: system.machine().cpu_mhz.get() as i32,
        fpu: system.session.host.has_fpu.get() as i32,
        mmx: system.machine().found_mmx.get() as i32,
        has_3dnow: system.machine().found_3dnow.get() as i32,
        katmai: system.machine().found_sse.get() as i32,
        machine_type: system.machine().keyboard_type.get() as i32,
        platform: [
            system.machine().version[4].get() as i32,
            system.machine().version[1].get() as i32,
            system.machine().version[2].get() as i32,
            (system.machine().version[3].get() & 0xFFFF) as i32,
        ],
        // The Windows version record's text, inside the record.
        platform_text: crate::files::cstring_in(&le_bytes(&machine.version[5..]))?,
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
