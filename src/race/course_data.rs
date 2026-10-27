//! `race::loadlevel_init`'s "locacurrcourse" (`sub_4394CB`, IDA 2026-10-16): the course's
//! data (`cdat00c`) read into the course memory, its first 0x100 bytes the header
//! (`g_courseHeader`, 0x5CCD14: the counts of `race/norms.rs`' layout), the 0x20-byte
//! records grouped by kind, the faces' lights reset, then the best line. Here the small
//! steps.

use crate::errors::{BadInput, Refused};
use crate::physics::contact_store::CONTACT_WORK;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::kind_records_store::KIND_RECORDS;
use crate::race::prefs_store::STAGE_PREFS;

/// `files\levels\levelxx\cdat00c`'s two digits, in `srclvls\..` (0x60B02B) and
/// `\levels\..` (0x60B07F): tens, units.
pub const HEADER_LEN: usize = 0x100;
const KINDS: u32 = 0x100;
/// The records the course memory's first part (0xF00000 bytes) can hold.
pub const MOST_RECORDS: u32 = 0x7_8000;

/// `race::set_cdat_number` (0x4456DD): `number`'s two digits (its low byte's `div 10`, +
/// '0') into both cdat paths.
pub fn set_cdat_number(number: u32) {
    // Converted: `rc2k_core::race::stage::two_digits`.
    let digits = rc2k_core::race::stage::two_digits(number as u8);
    let p = &crate::race::level_paths_store::LEVEL_PATHS;
    p.cdat_src.set(0x14, digits[0]);
    p.cdat_src.set(0x15, digits[1]);
    p.cdat_files.set(0x19, digits[0]);
    p.cdat_files.set(0x1A, digits[1]);
}

/// `race::copy_course_header` (0x445736): the course memory's first 0x100 bytes to
/// `g_courseHeader`.
pub fn copy_course_header() -> Result<(), BadInput> {
    let from: u32 = COURSE_LISTS.course_memory.bits();
    GamePtr::new(from).copy_to(
        GamePtr::new(COURSE.course_header.addr()),
        HEADER_LEN,
        "the course memory's header",
    )
}

/// `race::clear_lit_faces` (0x45E9D5): `g_litFaceCount` 0.
pub fn clear_lit_faces() {
    CONTACT_WORK.lit_face_count.set_bits(0u32);
}

/// `race::reset_face_lights` (0x45ED70): each face whose first byte is 0xFF gets its
/// +0x30 .. +0x38 and bytes +0x11 .. +0x13 0; then `clear_lit_faces`. Refuses no faces
/// (the original counts round) or 0x100000 or more.
pub fn reset_face_lights() -> Result<(), Refused> {
    {
        let count: u32 = COURSE.course_face_count.bits();
        rc2k_core::race::faces::check_face_count(count)?;
        // Converted: `rc2k_core::race::faces::reset_lights` (each record inside the
        // faces the original walks; written back only when changed).
        let faces: u32 = COURSE.course_faces.bits();
        const FACE: usize = rc2k_core::race::faces::FACE_BYTES;
        let all =
            crate::race::course_places::bytes(faces, count as usize * FACE, "the course faces")?;
        for (n, bytes) in all.as_chunks::<FACE>().0.iter().enumerate() {
            let mut face = *bytes;
            let kept = face;
            rc2k_core::race::faces::reset_lights(&mut face);
            if face != kept {
                crate::race::course_places::write(
                    faces.wrapping_add((n as u32) << 6),
                    face,
                    "a course face",
                )?;
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
pub fn group_records_by_kind() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::stage::group_by_kind`; the records read whole (every
    // one is swapped into place, so the original reads them all).
    {
        let count: u32 = COURSE.course_record_count.bits();
        if count > MOST_RECORDS {
            return Err(BadInput("more records than the course memory holds").into());
        }
        let records: u32 = COURSE.course_records.bits();
        let bytes =
            crate::race::course_places::bytes(records, count as usize * 32, "the course records")?;
        let mut list: Vec<[u32; 8]> = bytes
            .as_chunks::<32>()
            .0
            .iter()
            .map(|r| {
                std::array::from_fn(|k| {
                    u32::from_le_bytes([r[4 * k], r[4 * k + 1], r[4 * k + 2], r[4 * k + 3]])
                })
            })
            .collect();
        let grouped = rc2k_core::race::stage::group_by_kind(&mut list, |r| r[0]);
        let out: Vec<u8> = list
            .iter()
            .flat_map(|r| r.iter().flat_map(|v| v.to_le_bytes()))
            .collect();
        crate::race::course_places::write_bytes(records, &out, "the course records")?;
        for kind in 0..KINDS as usize {
            KIND_RECORDS.first_records[kind].set(grouped.first[kind]);
            KIND_RECORDS.last_records[kind].set(grouped.last[kind]);
        }
    }
    Ok(())
}

/// "loadlevel", "done", "next", "this", "then", "ok".
const LOAD_LOG: [&std::ffi::CStr; 6] = [c"loadlevel", c"done", c"next", c"this", c"then", c"ok"];
const NOT_READ: crate::errors::FatalError = crate::errors::FatalError {
    code: 0x8065_0210,
    message: "",
};

/// A log line ("loadlevel" .. "ok").
fn load_log(n: usize) {
    unsafe { crate::logfile::log_write::log_write(LOAD_LOG[n].as_ptr().cast(), false, 1) };
}

/// The scene points moved out of the course memory (`race::scene_points`): from where the
/// layout puts them up to the first part's end (where the course info was), the room the
/// game writes new points into.
fn take_scene_points() -> Result<(), BadInput> {
    let layout = rc2k_core::race::faces::CourseLayout::of(
        COURSE_LISTS.course_memory.bits(),
        COURSE.course_record_count.bits(),
        COURSE.course_face_count.bits(),
        COURSE.scene_point_count.bits(),
    );
    // The course info followed the first part in the original's block; it is a store of
    // its own now (`race::course_places::INFO`): the first part's end.
    let end: u32 = COURSE_LISTS
        .course_memory
        .bits()
        .wrapping_add(COURSE_LISTS.first_part.bits());
    if end <= layout.points {
        return Err(BadInput("scene points past the course info"));
    }
    crate::race::scene_points::take_from(layout.points, end - layout.points)?;
    crate::race::course_places::take_from(
        layout.faces,
        layout.points - layout.faces,
        layout.records,
        layout.faces - layout.records,
    )
}

/// `race::load_level` (0x4455FC): the course data (cdat 0) read into the course memory's
/// first part (`files::read_level_file`, the bytes in `dword_60B2CC`; a failed read is
/// fatal error 0x80650210, without a message), its header copied, laid out, the faces'
/// lights reset, with "loadlevel", "done", "next", "this", "then", "ok" in the log.
pub fn load_level() -> Result<(), Refused> {
    use crate::files::{read_level_file, read_packed_file};
    set_cdat_number(0);
    crate::files::state::BYTES_READ.store(0, std::sync::atomic::Ordering::Relaxed);
    load_log(0);
    let out = read_level_file::read_level_file(
        crate::race::level_paths_store::LEVEL_PATHS.cdat_src.addr(),
        COURSE_LISTS.course_memory.bits(),
        COURSE_LISTS.first_part.bits(),
        &mut read_level_file::game_exists,
        &mut read_packed_file::game_read,
    )?;
    crate::files::state::BYTES_READ.store(out.eax, std::sync::atomic::Ordering::Relaxed);
    if out.carry {
        return Err(NOT_READ.into());
    }
    load_log(1);
    load_log(2);
    copy_course_header()?;
    take_scene_points()?;
    load_log(3);
    crate::race::norms::lay_out_course();
    load_log(4);
    reset_face_lights()?;
    load_log(5);
    Ok(())
}

/// `race::locate_current_course` (0x4394CB, "locacurrcourse"): `dword_609380` 0,
/// `load_level`, `lay_out_course` (its test of `dword_621514` jumps to the next
/// instruction either way: section 16), `group_records_by_kind`, `build_best_line`.
pub fn locate_current_course() -> Result<(), Refused> {
    STAGE_PREFS.cdat_number.set_bits(0u32);
    load_level()?;
    crate::race::norms::lay_out_course();
    group_records_by_kind()?;
    crate::race::best_line::build_best_line()
}
