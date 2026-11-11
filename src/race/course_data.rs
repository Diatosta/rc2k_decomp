//! `race::loadlevel_init`'s "locacurrcourse" (`race::locate_current_course`): the
//! course's data (`cdat00c`) read into the course memory, its first 0x100 bytes the header
//! (`g_courseHeader`, 0x5CCD14: the counts of `race/norms.rs`' layout), the 0x20-byte
//! records grouped by kind, the faces' lights reset, then the best line. Here the small
//! steps.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_physics::Physics;

/// `files\levels\levelxx\cdat00c`'s two digits, in `srclvls\..` (0x60B02B) and
/// `\levels\..` (0x60B07F): tens, units.
pub const HEADER_LEN: usize = 0x100;
const KINDS: u32 = 0x100;
/// The records the course memory's first part (0xF00000 bytes) can hold.
pub const MOST_RECORDS: u32 = 0x7_8000;

/// `race::set_cdat_number` (0x4456DD): `number`'s two digits (its low byte's `div 10`, + '0') into
/// both cdat paths.
pub fn set_cdat_number(course: &Course, number: u32) {
    // Converted: `rc2k_core::race::stage::two_digits`.
    let digits = rc2k_core::race::stage::two_digits(number as u8);
    let p = &course.level_paths;
    p.cdat_src.set(0x14, digits[0]);
    p.cdat_src.set(0x15, digits[1]);
    p.cdat_files.set(0x19, digits[0]);
    p.cdat_files.set(0x1A, digits[1]);
}

/// `race::copy_course_header` (0x445736): the course memory's first 0x100 bytes to
/// `g_courseHeader`.
pub fn copy_course_header(course: &Course) -> Result<(), BadInput> {
    let from: u32 = course.lists.course_memory.get();
    let header = crate::race::level_init::read_course_memory(
        from,
        HEADER_LEN,
        "the course memory's header",
    )?;
    // The header runs over the course store's fields from the header's own to the shape radius's:
    // the counts, the start place and the fields nothing reads, in their order.
    let c = &course.course;
    let fields: [&[crate::session::Dword]; 18] = [
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
    let mut words = header
        .as_chunks::<4>()
        .0
        .iter()
        .map(|w| u32::from_le_bytes(*w));
    for dword in fields.into_iter().flatten().chain(&c.unused_1f4) {
        if let Some(word) = words.next() {
            dword.set(word);
        }
    }
    Ok(())
}

/// `race::clear_lit_faces` (0x45E9D5): `g_litFaceCount` 0.
pub fn clear_lit_faces(physics: &Physics) {
    physics.contact.lit_face_count.set(0u32);
}

/// `race::reset_face_lights` (0x45ED70): each face whose first byte is 0xFF gets its +0x30 .. +0x38
/// and bytes +0x11 .. +0x13 0; then `clear_lit_faces`. Refuses no faces (the original counts round)
/// or 0x100000 or more.
pub fn reset_face_lights(race: &Race) -> Result<(), Refused> {
    let course = &race.course;
    let physics = &race.physics;
    {
        let count: u32 = course.course.course_face_count.get();
        rc2k_core::race::faces::check_face_count(count)?;
        // Converted: `rc2k_core::race::faces::reset_lights` (each record inside the
        // faces the original walks; written back only when changed).
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

/// `race::group_records_by_kind` (0x421E42): the course's 0x20-byte records put in order of kind
/// (low four bits of the first dword, + bits 11 .. 14 times 16), each kind's in the order met, by
/// swapping each one found into the next place; `g_kindFirstRecords` and `g_kindLastRecords` each
/// kind's first and last number, -1 for both with none. Refuses more records than the course
/// memory's first part holds.
pub fn group_records_by_kind(course: &Course) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::stage::group_by_kind`; the records read whole (every
    // one is swapped into place, so the original reads them all).
    {
        let count: u32 = course.course.course_record_count.get();
        if count > MOST_RECORDS {
            return Err(BadInput("more records than the course memory holds").into());
        }
        let bytes = crate::race::course_places::records_table(course, count)?;
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
            course.kind_records.first_records[kind].set(grouped.first[kind]);
            course.kind_records.last_records[kind].set(grouped.last[kind]);
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
    crate::logfile::log_write::log_nul(&LOAD_LOG[n], false, 1);
}

/// The scene points moved out of the course memory (`race::scene_points`): from where the
/// layout puts them up to the first part's end (where the course info was), the room the
/// game writes new points into.
fn take_scene_points(course: &Course) -> Result<(), BadInput> {
    let layout = rc2k_core::race::faces::CourseLayout::of(
        course.lists.course_memory.get(),
        course.course.course_record_count.get(),
        course.course.course_face_count.get(),
        course.course.scene_point_count.get(),
    );
    // The course info followed the first part in the original's block; it is a store of
    // its own now (`race::course_places::INFO`): the first part's end.
    let end: u32 = course
        .lists
        .course_memory
        .get()
        .wrapping_add(course.lists.first_part.get());
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

/// `race::load_level` (0x4455FC): the course data (cdat 0) read into the course memory's first part
/// (`files::read_level_file`, the bytes in `dword_60B2CC`; a failed read is fatal error 0x80650210,
/// without a message), its header copied, laid out, the faces' lights reset, with "loadlevel",
/// "done", "next", "this", "then", "ok" in the log.
pub fn load_level(race: &Race) -> Result<(), Refused> {
    let course = &race.course;
    use crate::files::read_level_file;
    set_cdat_number(course, 0);
    crate::files::state::BYTES_READ.set(0);
    load_log(0);
    // The course memory is a region of its own (`level_init::take_course_memory`).
    let memory = course.lists.course_memory.get();
    let mut data = crate::race::level_init::read_course_memory(
        memory,
        course.lists.first_part.get() as usize,
        "the course data",
    )?;
    let read = read_level_file::read_level_file(&course.level_paths.cdat_src.text(), &mut data);
    // What the file's bytes reached stays, also when the unpacking is refused.
    crate::race::level_init::write_course_memory(memory, &data, "the course data")?;
    let out = read?;
    crate::files::state::BYTES_READ.set(out.eax);
    if out.carry {
        // The original says nothing more: the path named, for the reports.
        crate::framework::log(&format!(
            "race::load_level: the course data \"{}\" could not be read",
            String::from_utf8_lossy(&course.level_paths.cdat_src.text())
        ));
        return Err(NOT_READ.into());
    }
    load_log(1);
    load_log(2);
    copy_course_header(course)?;
    take_scene_points(course)?;
    load_log(3);
    crate::race::norms::lay_out_course(course);
    load_log(4);
    reset_face_lights(race)?;
    load_log(5);
    Ok(())
}

/// `race::locate_current_course` (0x4394CB, "locacurrcourse"): `cdat_number` 0,
/// `load_level`, `lay_out_course` (its test of `editor_mode` jumps to the next
/// instruction either way: section 16), `group_records_by_kind`, `build_best_line`.
pub fn locate_current_course(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let course = &race.course;
    course.stage_prefs.cdat_number.set(0u32);
    load_level(race)?;
    crate::race::norms::lay_out_course(course);
    group_records_by_kind(course)?;
    crate::race::best_line::build_best_line(race, scene)
}
