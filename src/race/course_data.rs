//! Reads the course's data (`cdat00c`): its first 0x100 bytes are the header (`course_header`: the
//! counts of the layout in `race/norms.rs`), the 0x20-byte records are grouped by kind, the faces'
//! lights reset, then the best line is built. This file holds the small steps.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_physics::Physics;
use rc2k_core::race::loading::{FIRST_PART, HEADER_WORDS};

const KINDS: u32 = 0x100;
/// The records a course file's room (`FIRST_PART` bytes) can hold.
pub const MOST_RECORDS: u32 = 0x7_8000;

/// Writes `number`'s two digits (its low byte's div 10, + '0') into both cdat paths.
pub(super) fn set_cdat_number(course: &Course, number: u32) {
    let digits = rc2k_core::race::stage::two_digits(number as u8);
    let p = course.level_paths();
    p.cdat_src.set(0x14, digits[0]);
    p.cdat_src.set(0x15, digits[1]);
    p.cdat_files.set(0x19, digits[0]);
    p.cdat_files.set(0x1A, digits[1]);
}

/// Stores the course file's header dwords in `course_header` and the fields after it, in their
/// order: the counts, the start place and the fields nothing reads.
fn store_course_header(course: &Course, header: &[u32; HEADER_WORDS]) {
    let c = course.course.get();
    let fields: [&[crate::memory::session::PlainDword]; 18] = [
        std::slice::from_ref(&c.course_header),
        std::slice::from_ref(&c.course_record_count),
        std::slice::from_ref(&c.course_face_count),
        std::slice::from_ref(&c.scene_point_count),
        std::slice::from_ref(&c.unused_134),
        std::slice::from_ref(&c.place_face),
        &c.unused_13c,
        std::slice::from_ref(&c.camera_first),
        &c.unused_154,
        std::slice::from_ref(&c.place_heading),
        &c.unused_174,
        std::slice::from_ref(&c.flat_depth),
        &c.unused_194,
        std::slice::from_ref(&c.place_x),
        &c.unused_1b4,
        std::slice::from_ref(&c.place_y),
        &c.unused_1d4,
        std::slice::from_ref(&c.place_z),
    ];
    let mut words = header.iter();
    for dword in fields.into_iter().flatten().chain(&c.unused_1f4) {
        if let Some(&word) = words.next() {
            dword.set(word);
        }
    }
}

/// Sets `lit_face_count` to 0.
fn clear_lit_faces(physics: &Physics) {
    physics.contact.get().lit_face_count.set(0u32);
}

/// Gives each face of the lit kind (0xFF) its lit normals and light marks 0
/// (`faces::reset_lights`); then `clear_lit_faces`. Refused: no faces or 0x100000 or more.
pub fn reset_face_lights(race: &Race) -> Result<(), Refused> {
    let course = &race.course;
    let physics = &race.physics;
    {
        let count: u32 = course.course.get().course_face_count.get();
        rc2k_core::race::faces::check_face_count(count)?;
        // Each record is read inside the faces; it is written back only when changed
        // (`rc2k_core::race::faces::reset_lights`).
        const FACE: usize = rc2k_core::race::faces::FACE_BYTES;
        let all = crate::race::course_places::faces_table(course, count)?;
        for (n, bytes) in all.as_chunks::<FACE>().0.iter().enumerate() {
            let mut face = *bytes;
            let kept = face;
            rc2k_core::race::faces::reset_lights(&mut face);
            if face != kept {
                crate::race::course_places::write_face_number(
                    course,
                    n as u32,
                    rc2k_core::race::faces::Face(face),
                )?;
            }
        }
        clear_lit_faces(physics);
    }
    Ok(())
}

/// Puts the course's 0x20-byte records in order of kind (low four bits of the first dword, + bits
/// 11 .. 14 times 16), each kind's in the order met, by swapping each one found into the next
/// place; `first_records` and `last_records` get each kind's first and last number, -1
/// for both with none. Refused: more records than a course file's room holds.
fn group_records_by_kind(course: &Course) -> Result<(), Refused> {
    // The records are read whole: every one is swapped into place, so all of them are read
    // (`rc2k_core::race::stage::group_by_kind`).
    {
        let count: u32 = course.course.get().course_record_count.get();
        if count > MOST_RECORDS {
            return Err(BadInput("more records than a course file holds").into());
        }
        let bytes = crate::race::course_places::records_table(course.reads(), count)?;
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
        crate::race::course_places::write_records_table(course, &out)?;
        for kind in 0..KINDS as usize {
            course.kind_records().first_records[kind].set(grouped.first[kind]);
            course.kind_records().last_records[kind].set(grouped.last[kind]);
        }
    }
    Ok(())
}

/// The log's words: "loadlevel", "done", "next", "this", "then", "ok".
const LOAD_LOG: [&std::ffi::CStr; 6] = [c"loadlevel", c"done", c"next", c"this", c"then", c"ok"];
/// What the player is told when the stage's data cannot be read: the race is left and the
/// game goes back to the main menu.
const NOT_READ: crate::errors::Recover =
    crate::errors::Recover("The data of the stage could not be read (see rc2k.log).");

/// A log line ("loadlevel" .. "ok").
fn load_log(n: usize) {
    crate::logfile::log_write::log_nul(&LOAD_LOG[n], false, 1);
}

/// Reads the course data (cdat 0) into a buffer of `FIRST_PART` bytes (a failed read is a trouble
/// the game goes back to the menu from), takes it apart (`rc2k_core::race::loading::read_course`),
/// stores the header and the parts, lays the course out and resets the faces' lights, with
/// "loadlevel", "done", "next", "this", "then", "ok" in the log.
pub(super) fn load_level(race: &Race) -> Result<(), Refused> {
    let course = &race.course;
    use crate::files::read_level_file;
    set_cdat_number(course, 0);
    load_log(0);
    let mut data = vec![0u8; FIRST_PART];
    crate::logfile::load_times::mark("read_level_file");
    let read = read_level_file::read_level_file(&course.level_paths().cdat_src.text(), &mut data)?;
    if read.is_none() {
        // The path named, for the reports.
        crate::program_log::log(&format!(
            "race::load_level: the course data \"{}\" could not be read",
            String::from_utf8_lossy(&course.level_paths().cdat_src.text())
        ));
        return Err(NOT_READ.into());
    }
    load_log(1);
    load_log(2);
    crate::logfile::load_times::mark("read_course, store header and parts");
    let parts =
        rc2k_core::race::loading::read_course(&data).map_err(|e| crate::errors::Recover(e.0))?;
    drop(data);
    store_course_header(course, &parts.header);
    crate::race::scene_points::take(&course.stage.course, &parts.points)?;
    crate::race::course_places::take(&course.stage.course, &parts.faces, &parts.records)?;
    load_log(3);
    load_log(4);
    crate::logfile::load_times::mark("reset_face_lights");
    reset_face_lights(race)?;
    load_log(5);
    Ok(())
}

/// Sets `cdat_number` 0 and runs `load_level`, `group_records_by_kind` and `build_best_line`;
/// the stage's "locacurrcourse" step.
pub(super) fn locate_current_course(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let course = &race.course;
    course.stage_prefs.get().cdat_number.set(0u32);
    crate::logfile::load_times::mark("load_level");
    load_level(race)?;
    crate::logfile::load_times::mark("group_records_by_kind");
    group_records_by_kind(course)?;
    crate::logfile::load_times::mark("build_best_line");
    crate::race::best_line::build_best_line(race, scene)
}
