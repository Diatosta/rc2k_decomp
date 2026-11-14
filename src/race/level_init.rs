//! The small steps of a stage's level initialisation (`race::loadlevel_init`). The course memory:
//! 0x1300000 bytes from `memory::allocate` on `CourseLists.course_memory_list[0]`, `course_memory`
//! its start and `course_info` its last 0x400000 bytes, where the texture and mark info files are
//! read.

use crate::cells::Store;
use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_frame::Frame;
use crate::race::trap::Trap;
use crate::scene::file_numbers_store::texts;
use crate::session::TextPlace;

const FIRST_PART: u32 = 0xF0_0000;
const INFO_PART: u32 = 0x40_0000;
/// The path strings the course's number is written into: the tens' and the units' places.
pub fn number_places(
    p: &'static crate::race::level_paths_store::LevelPaths,
) -> [crate::place::BytePlace; 6] {
    use crate::place::BytePlace;
    [
        BytePlace::from(&p.edit_path).add(0xD),
        BytePlace::from(&p.cdat_src).add(0xD),
        BytePlace::from(&p.cdat_files).add(0x12),
        BytePlace::from(&p.cdat_src_copy).add(0xD),
        BytePlace::from(&p.localdat_src).add(0xD),
        BytePlace::from(&p.localdat_files).add(0x12),
    ]
}
const TEXTURES: u32 = 0x100;

/// Takes the course memory (`memory::allocate` on the list `list`).
pub fn allocate_course_memory(
    course: &Course,
    list: &crate::memory::ListCells,
) -> Result<(), Refused> {
    course.lists.get().first_part.set(FIRST_PART);
    course.lists.get().course_bytes_total.set(
        course
            .lists
            .get()
            .course_bytes_total
            .get()
            .wrapping_add(FIRST_PART + INFO_PART),
    );
    // Counted whole as one block; the info part is Rust's own store (`race::course_places::INFO`),
    // named by its place.
    let memory = take_course_memory(list, FIRST_PART + INFO_PART, FIRST_PART)?;
    course.lists.get().course_memory.set(memory);
    course
        .lists
        .get()
        .course_info
        .set(crate::race::course_places::take_info(INFO_PART)?);
    Ok(())
}

/// The course memory (`memory::allocate` on the course's list): Rust's bytes, counted on the list.
/// The stage's files are read into it and parsed in place; its faces, records and scene points
/// then move out (`race::course_places`). Given back with the list (`stage_free::COURSE`).
static COURSE_MEMORY: std::sync::Mutex<Option<Vec<u8>>> = std::sync::Mutex::new(None);

/// The course memory by handle: `COURSE_TAG | offset` names the byte at `offset` of the course
/// memory, 0 none. `course_memory` holds `COURSE_TAG` (offset 0), and the layout's addresses
/// (`lay_out_course`) are `COURSE_TAG` plus their offset from the block's start;
/// `course_memory_offset` turns one back into an offset. No host address is kept.
pub const COURSE_TAG: u32 = 0xCF00_0000;

/// `bytes` of course memory counted on `list`, the first `kept` of them held: their handle
/// (`COURSE_TAG`).
fn take_course_memory(
    list: &crate::memory::ListCells,
    bytes: u32,
    kept: u32,
) -> Result<u32, Refused> {
    let mut memory = crate::memory::owned::counted(list, bytes)?;
    memory.truncate(kept as usize);
    memory.shrink_to_fit();
    *COURSE_MEMORY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(memory);
    Ok(COURSE_TAG)
}

/// The course memory's offset of handle `at` (`COURSE_TAG | offset`); one before the memory is
/// refused.
fn course_memory_offset(at: u32) -> Result<u32, BadInput> {
    at.checked_sub(COURSE_TAG)
        .ok_or(BadInput("a place before the course memory"))
}

/// `len` bytes of the course memory from handle `at` (`what` names them in a refusal); a handle
/// outside the memory, or no course memory, is refused.
pub fn read_course_memory(at: u32, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
    let guard = COURSE_MEMORY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let memory = guard.as_ref().ok_or(BadInput("no course memory"))?;
    let offset = course_memory_offset(at)?;
    let range = within(memory, offset, len, what)?;
    Ok(memory[range].to_vec())
}

/// The range of `len` bytes from `offset` in `memory`; one past its end is refused.
fn within(
    memory: &[u8],
    offset: u32,
    len: usize,
    what: &str,
) -> Result<std::ops::Range<usize>, BadInput> {
    let start = offset as usize;
    match start.checked_add(len).filter(|&end| end <= memory.len()) {
        Some(end) => Ok(start..end),
        None => {
            crate::framework::log(&format!(
                "pointers: {what}: {len} bytes at offset {offset:#x}, past the {:#x} bytes",
                memory.len()
            ));
            Err(BadInput("past the end of a field"))
        }
    }
}

/// `data` written into the course memory from handle `at`.
pub fn write_course_memory(at: u32, data: &[u8], what: &str) -> Result<(), BadInput> {
    let mut guard = COURSE_MEMORY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let memory = guard.as_mut().ok_or(BadInput("no course memory"))?;
    let range = within(memory, course_memory_offset(at)?, data.len(), what)?;
    memory[range].copy_from_slice(data);
    Ok(())
}

/// The course memory given back (the list's `free_list` counts it down).
pub fn give_back_course_memory() {
    *COURSE_MEMORY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

/// Copies 0x800 bytes of the course info from +0x17E800 to +0x17F000.
pub fn copy_texture_info_tail() -> Result<(), BadInput> {
    use crate::race::course_places::{info_bytes, write_info};
    write_info(0x17_F000, &info_bytes(0x17_E800, 0x800)?)
}

/// Reads `files\a\textinfo` (0x1F4000 bytes at most) into the course info, `files\m\markinfo`
/// (0x8000) at +0x1C2000 and `files\b\textinfo` at +0x1F4000, puts three line feeds at +0x5BD21,
/// then `copy_texture_info_tail`. Refuses no course info.
pub fn load_texture_info(
    course: &Course,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let file_numbers = &scene.file_numbers();
    let info = course.lists.get().course_info.get();
    if info == 0 {
        return Err(BadInput("no course memory: the original reads into 0").into());
    }
    let read = |texts_at: usize, offset: u32, count: u32| {
        let path = file_numbers.text_at(texts_at).text("a course info path")?;
        crate::race::course_places::read_file_into_info(&path, offset, count)
    };
    read(texts::TEXTURE_INFO_A, 0, 0x1F_4000)?;
    read(texts::MARK_INFO, 0x1C_2000, 0x8000)?;
    read(texts::TEXTURE_INFO_B, 0x1F_4000, 0x1F_4000)?;
    crate::race::course_places::write_info(0x5_BD21, &[0x0A; 3])?;
    copy_texture_info_tail()?;
    Ok(())
}

/// Writes the course's two digits (tens, units, from the low byte divided by 10) into the nine
/// path strings (the stage's files) that hold the course's number.
pub fn set_course_number(
    eax: u32,
    race: &'static Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    // Logic in `rc2k_core::race::stage::two_digits`.
    let [tens, units] = rc2k_core::race::stage::two_digits(eax as u8);
    for place in number_places(race.course.level_paths()) {
        place.write(&[tens, units], "a path's course number")?;
    }
    for text in [
        texts::NUMBER_PLACE_A,
        texts::NUMBER_PLACE_B,
        texts::NUMBER_PLACE_C,
    ] {
        scene.file_numbers().text_at(text).write_text_at(
            0,
            &[tens, units],
            "a path's course number",
        )?;
    }
    Ok(())
}

/// Reads the named file, 0x2000 bytes at most, into the error list.
pub fn load_error_list(frame: &Frame, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    crate::files::read_whole_file::read_whole_file_plain_bytes(
        &scene
            .file_numbers()
            .text_at(texts::ERROR_LIST_PATH)
            .text("the error list's path")?,
        &frame.error_list().list,
    )?;
    Ok(())
}

/// Builds the texture infos: nothing without course info; else for each of the 256 textures, from
/// its kind k (`texture_kinds`) and the info's record at k * 0x80, a record of 0x40: +0x24 k & 0xF,
/// +0x20 k >> 4, +0x28 that | 0x80000000, +0x10 / +0x14 / +0x18 the record's +0x20 / +0x10 /
/// +0x14, +0x1C its byte +0x18, +0x2C ..= +0x2E its bytes +0x24 ..= +0x26, +0 ..= +0xC its first
/// four. Refuses a kind whose record lies past the info.
pub fn build_texture_infos(race: &Race) -> Result<(), Refused> {
    let course = &race.course;
    {
        let info = course.lists.get().course_info.get();
        if info == 0 {
            return Ok(());
        }
        for n in 0..TEXTURES {
            if course.stage_prefs.get().texture_kinds[n as usize].get() >= INFO_PART / 0x80 {
                return Err(BadInput("a texture kind past the course info").into());
            }
        }
        // Logic in `rc2k_core::race::stage::texture_info`.
        for n in 0..TEXTURES {
            let out = &race.polygon_tables().texture_infos[n as usize].words;
            let kind = course.stage_prefs.get().texture_kinds[n as usize].get();
            let rec = kind * 0x80;
            let trap = Trap::new();
            use crate::race::course_places::info_field as field;
            let t = rc2k_core::race::stage::texture_info(
                kind,
                |o| trap.keep_read(field::<u32>(rec + o)),
                |o| trap.keep_read(field::<u8>(rec + o)),
            );
            trap.finish()?;
            out[9].set(t.row);
            out[8].set(t.group);
            for (k, &value) in t.values.iter().enumerate() {
                out[4 + k].set(value);
            }
            // The record's byte at +0x1C and three at +0x2C are bytes inside their dwords.
            out[7].store(0, &[t.byte])?;
            out[11].store(0, &t.bytes)?;
            for (k, &value) in t.first.iter().enumerate() {
                out[k].set(value);
            }
            out[10].set(t.marked);
        }
    }
    Ok(())
}

/// Resets `grid_x` and `grid_z`, the course grid's corner (each view keeps its own,
/// `race::view_grid`), so every view makes its grid again.
pub fn reset_refresh() {
    crate::race::view_grid::reset_corners();
}

/// Lists the checkpoints: `split_colour_count` 0; unless `checkpoint_count` - 2 is below 0, the
/// table entries 1 .. `last_checkpoint` - 1 (`Buffers.checkpoints.checkpoint_faces[0][n]`) whose
/// record (`course_faces` + it * 0x40) has its byte +0x26 set go into `checkpoints`, and
/// `split_colour_count` becomes their count - 1. Refuses an end below 2, or more found than the
/// list's 0x201 (a real stage's end is past 0x201).
pub fn init_checkpoints(race: &Race) -> Result<(), Refused> {
    let physics = &race.physics;
    let shared = &race.shared;
    // Logic in `rc2k_core::race::checkpoints::listed_checkpoints`.
    {
        let kinds = shared.vars.get().checkpoint_count.get();
        if (kinds.wrapping_sub(2) as i32) < 0 {
            physics.split_tables().split_colour_count.set(0);
            return Ok(());
        }
        let end = shared.vars.get().last_checkpoint.get();
        let table = &race.checkpoints().checkpoint_faces;
        let trap = Trap::new();
        let listed = rc2k_core::race::checkpoints::listed_checkpoints(end, |n| {
            // n < `last_checkpoint`, which `race::best_line` sets to the chain's length - 1 after
            // writing the chain into the table's 5146 entries: inside the table.
            let face = trap.keep_read(
                table
                    .get(n as usize)
                    .map(crate::session::PlainDword::get)
                    .ok_or(BadInput("a checkpoint past the checkpoint table")),
            );
            trap.keep_read(crate::race::course_places::face_field::<u8>(
                race.course.reads(),
                face,
                0x26,
            )) != 0
        })?;
        trap.finish()?;
        physics.split_tables().split_colour_count.set(0);
        // The list holds 0x101 entries but may run on for up to 0x201 (`MOST_LISTED`), over the
        // stage's numbers that follow it (0x100 entries: the two fields end together at 0x201).
        // Such a stage is refused later by `race::init_overlay` (more than 0x40 parts).
        let tables = physics.split_tables();
        for (k, &n) in listed.iter().enumerate() {
            let entry = tables
                .checkpoints
                .get(k)
                .or_else(|| tables.stage_numbers.get(k - tables.checkpoints.len()));
            entry
                .ok_or(BadInput(
                    "a split past the split list and the stage's numbers",
                ))?
                .set(n);
        }
        physics
            .split_tables()
            .split_colour_count
            .set((listed.len() as u32).wrapping_sub(1));
    }
    Ok(())
}
