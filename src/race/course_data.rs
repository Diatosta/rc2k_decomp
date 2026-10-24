//! `race::loadlevel_init`'s "locacurrcourse" (`sub_4394CB`, IDA 2026-10-16): the course's
//! data (`cdat00c`) read into the course memory, its first 0x100 bytes the header
//! (`g_courseHeader`, 0x5CCD14: the counts of `race/norms.rs`' layout), the 0x20-byte
//! records grouped by kind, the faces' lights reset, then the best line. Here the small
//! steps.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};

/// `files\levels\levelxx\cdat00c`'s two digits, in `srclvls\..` (0x60B02B) and
/// `\levels\..` (0x60B07F): tens, units.
pub const CDAT_DIGITS: [usize; 2] = [rva(0x60B03F), rva(0x60B093)];
pub const COURSE_MEMORY_RVA: usize = rva(0x5C6F94); // g_courseMemory
pub const HEADER_RVA: usize = rva(0x5CCD14); // g_courseHeader
pub const HEADER_LEN: usize = 0x100;
pub const LIT_COUNT_RVA: usize = rva(0x6D3620); // g_litFaceCount
pub const FACES_RVA: usize = rva(0x5CCCBC); // g_courseFaces
pub const FACE_COUNT_RVA: usize = rva(0x5CCD1C); // g_courseFaceCount
pub const RECORDS_RVA: usize = rva(0x5CCCB8); // g_courseRecords
pub const RECORD_COUNT_RVA: usize = rva(0x5CCD18); // g_courseRecordCount
/// `g_kindFirstRecords` (0x100 dwords), followed by `g_kindLastRecords` (0x5CC7E4).
pub const KIND_TABLES_RVA: usize = rva(0x5CC3E4);
const KINDS: u32 = 0x100;
/// The records the course memory's first part (0xF00000 bytes) can hold.
pub const MOST_RECORDS: u32 = 0x7_8000;

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

/// `race::set_cdat_number` (0x4456DD): `number`'s two digits (its low byte's `div 10`, +
/// '0') into both cdat paths.
pub unsafe fn set_cdat_number(number: u32) {
    // Converted: `rc2k_core::race::stage::two_digits`.
    let digits = rc2k_core::race::stage::two_digits(number as u8);
    for at in CDAT_DIGITS {
        unsafe { game_write(at, digits) };
    }
}

/// `race::copy_course_header` (0x445736): the course memory's first 0x100 bytes to
/// `g_courseHeader`.
pub unsafe fn copy_course_header() {
    unsafe {
        let from: u32 = game_read(COURSE_MEMORY_RVA);
        std::ptr::copy(
            from as *const u8,
            game_addr(HEADER_RVA) as *mut u8,
            HEADER_LEN,
        );
    }
}

/// `race::clear_lit_faces` (0x45E9D5): `g_litFaceCount` 0.
pub unsafe fn clear_lit_faces() {
    unsafe { game_write(LIT_COUNT_RVA, 0u32) };
}

/// `race::reset_face_lights` (0x45ED70): each face whose first byte is 0xFF gets its
/// +0x30 .. +0x38 and bytes +0x11 .. +0x13 0; then `clear_lit_faces`. Refuses no faces
/// (the original counts round) or 0x100000 or more.
pub unsafe fn reset_face_lights() -> Result<(), Refused> {
    unsafe {
        let count: u32 = game_read(FACE_COUNT_RVA);
        rc2k_core::race::faces::check_face_count(count)?;
        // Converted: `rc2k_core::race::faces::reset_lights` (each record inside the
        // faces the original walks; written back only when changed).
        let faces: u32 = game_read(FACES_RVA);
        for n in 0..count {
            let at = faces.wrapping_add(n << 6) as *mut [u8; rc2k_core::race::faces::FACE_BYTES];
            let mut face = at.read_unaligned();
            let kept = face;
            rc2k_core::race::faces::reset_lights(&mut face);
            if face != kept {
                at.write_unaligned(face);
            }
        }
        clear_lit_faces();
    }
    Ok(())
}

/// `race::group_records_by_kind` (0x421E42): the course's 0x20-byte records put in order
/// of kind (low four bits of the first dword, + bits 11 .. 14 times 16), each kind's in
/// the order met, by swapping each one found into the next place; `g_kindFirstRecords`
/// and `g_kindLastRecords` each kind's first and last number, -1 for both with none.
/// Refuses more records than the course memory's first part holds.
pub unsafe fn group_records_by_kind() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::stage::group_by_kind`; the records read whole (every
    // one is swapped into place, so the original reads them all).
    unsafe {
        let count: u32 = game_read(RECORD_COUNT_RVA);
        if count > MOST_RECORDS {
            return Err(BadInput("more records than the course memory holds").into());
        }
        let records: u32 = game_read(RECORDS_RVA);
        let at = records as *mut [u32; 8];
        let mut list: Vec<[u32; 8]> = (0..count as usize)
            .map(|n| at.add(n).read_unaligned())
            .collect();
        let grouped = rc2k_core::race::stage::group_by_kind(&mut list, |r| r[0]);
        for (n, r) in list.iter().enumerate() {
            at.add(n).write_unaligned(*r);
        }
        let first = game_addr(KIND_TABLES_RVA) as u32;
        let last = first + 4 * KINDS;
        for kind in 0..KINDS {
            put(first + 4 * kind, grouped.first[kind as usize]);
            put(last + 4 * kind, grouped.last[kind as usize]);
        }
    }
    Ok(())
}

// ---- for the self-tests ----

const COURSE_PATH_RVA: usize = rva(0x60B02B); // "srclvls\level01\cdat00c"
const FIRST_PART_RVA: usize = rva(0x5C6FC8); // dword_5C6FC8: the course memory's first part
/// "loadlevel", "done", "next", "this", "then", "ok".
const LOAD_LOG: [usize; 6] = [
    rva(0x4DC487),
    rva(0x4DC491),
    rva(0x4DC496),
    rva(0x4DC49B),
    rva(0x4DC4A0),
    rva(0x4DC4A5),
];
const NOT_READ: crate::errors::FatalError = crate::errors::FatalError {
    code: 0x8065_0210,
    message: "",
};

/// A log line ("loadlevel" .. "ok").
fn load_log(n: usize) {
    unsafe { crate::logfile::log_write::log_write(game_addr(LOAD_LOG[n]) as *const u8, false, 1) };
}

/// `race::load_level` (0x4455FC): the course data (cdat 0) read into the course memory's
/// first part (`files::read_level_file`, the bytes in `dword_60B2CC`; a failed read is
/// fatal error 0x80650210, without a message), its header copied, laid out, the faces'
/// lights reset, with "loadlevel", "done", "next", "this", "then", "ok" in the log.
pub unsafe fn load_level() -> Result<(), Refused> {
    use crate::files::{read_level_file, read_packed_file};
    unsafe {
        set_cdat_number(0);
        crate::files::state::BYTES_READ.store(0, std::sync::atomic::Ordering::Relaxed);
        load_log(0);
        let out = read_level_file::read_level_file(
            game_addr(COURSE_PATH_RVA) as u32,
            game_read(COURSE_MEMORY_RVA),
            game_read(FIRST_PART_RVA),
            &mut read_level_file::game_exists,
            &mut read_packed_file::game_read,
        )?;
        crate::files::state::BYTES_READ.store(out.eax, std::sync::atomic::Ordering::Relaxed);
        if out.carry {
            return Err(NOT_READ.into());
        }
        load_log(1);
        load_log(2);
        copy_course_header();
        load_log(3);
        crate::race::norms::lay_out_course();
        load_log(4);
        reset_face_lights()?;
        load_log(5);
    }
    Ok(())
}

const CDAT_NUMBER_RVA: usize = rva(0x609380); // dword_609380

/// `race::locate_current_course` (0x4394CB, "locacurrcourse"): `dword_609380` 0,
/// `load_level`, `lay_out_course` (its test of `dword_621514` jumps to the next
/// instruction either way: section 16), `group_records_by_kind`, `build_best_line`.
pub unsafe fn locate_current_course() -> Result<(), Refused> {
    unsafe {
        game_write(CDAT_NUMBER_RVA, 0u32);
        load_level()?;
        crate::race::norms::lay_out_course();
        group_records_by_kind()?;
        crate::race::best_line::build_best_line()
    }
}
