//! `race::load_stage`'s "loadlevel_init" (`sub_44426F`): its small steps (IDA,
//! 2026-10-16). The course memory (`alloc coursemem`): 0x1300000 bytes from
//! `memory::allocate` on `stru_5C6D50`, `g_courseMemory` (0x5C6F94) its start and
//! `g_courseInfo` (0x5C6FA0) its last 0x400000 bytes, where the texture and mark info
//! files are read.

use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::race::owner::Race;
use crate::race::owner::race;
use crate::race::owner_course::Course;
use crate::race::owner_frame::Frame;
use crate::race::trap::Trap;
use crate::scene::file_numbers_store::texts;
use crate::scene::owner::scene;

const FIRST_PART: u32 = 0xF0_0000;
const INFO_PART: u32 = 0x40_0000;
/// The path strings the course's number is written into: the tens' and the units' places.
pub fn number_places() -> [crate::place::Place; 9] {
    let file_numbers = &scene().file_numbers;
    use crate::place::Place;
    let p = &race().course.level_paths;
    [
        Place::from(&p.edit_path).add(0xD),
        Place::from(&p.cdat_src).add(0xD),
        Place::from(&p.cdat_files).add(0x12),
        Place::from(&p.cdat_src_2).add(0xD),
        Place::from(&p.localdat_src).add(0xD),
        Place::from(&p.localdat_files).add(0x12),
        file_numbers.text_ptr(texts::NUMBER_PLACE_A).into(),
        file_numbers.text_ptr(texts::NUMBER_PLACE_B).into(),
        file_numbers.text_ptr(texts::NUMBER_PLACE_C).into(),
    ]
}
const TEXTURES: u32 = 0x100;

/// `race::allocate_course_memory` (0x41F8C4): the course memory taken (`memory::allocate`
/// on the list `list`).
pub fn allocate_course_memory(
    course: &Course,
    list: &[crate::session::Dword; 4],
) -> Result<(), Refused> {
    course.lists.first_part.set(FIRST_PART);
    course.lists.course_bytes_total.set(
        course
            .lists
            .course_bytes_total
            .get()
            .wrapping_add(FIRST_PART + INFO_PART),
    );
    // Counted whole, as the original's one block; the info part is Rust's own store
    // (`race::course_places::INFO`), named by its place.
    let memory = take_course_memory(
        crate::pointers::GamePtr::to(list),
        FIRST_PART + INFO_PART,
        FIRST_PART,
    )?;
    course.lists.course_memory.set(memory);
    course
        .lists
        .course_info
        .set(crate::race::course_places::take_info(INFO_PART)?);
    Ok(())
}

/// The course memory (`memory::allocate` on the course's list in the original): Rust's
/// bytes since phase 13 (2026-10-27), counted on the list as the game's block was and lent
/// (the stage's files are read into it and parsed in place; its faces, records and scene
/// points then move out, `race::course_places`); given back with the list
/// (`stage_free::COURSE`).
struct CourseMemory {
    _bytes: crate::pointers::LentBytes,
}

static COURSE_MEMORY: std::sync::Mutex<Option<CourseMemory>> = std::sync::Mutex::new(None);

/// `bytes` of course memory counted on `list`, the first `kept` of them held: their game
/// address.
fn take_course_memory(
    list: crate::pointers::GamePtr,
    bytes: u32,
    kept: u32,
) -> Result<u32, Refused> {
    let mut memory = crate::memory::owned::counted(list, bytes)?;
    memory.truncate(kept as usize);
    memory.shrink_to_fit();
    let memory = crate::pointers::LentBytes::from_vec(memory);
    let at = memory.addr();
    *COURSE_MEMORY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(CourseMemory { _bytes: memory });
    Ok(at)
}

/// The place at game address `at` inside the course memory, bounded by the memory's bytes;
/// an address outside it, or no course memory, is refused.
pub fn course_memory_at(at: u32) -> Result<crate::pointers::GamePtr, BadInput> {
    let guard = COURSE_MEMORY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let memory = &guard.as_ref().ok_or(BadInput("no course memory"))?._bytes;
    let off = at
        .checked_sub(memory.addr())
        .ok_or(BadInput("a place before the course memory"))?;
    Ok(memory.ptr().offset(off))
}

/// The course memory given back (the list's `free_list` counts it down).
pub fn give_back_course_memory() {
    *COURSE_MEMORY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

/// `race::copy_texture_info_tail` (0x439AE5): 0x800 bytes of the course info from
/// +0x17E800 to +0x17F000.
pub fn copy_texture_info_tail() -> Result<(), BadInput> {
    use crate::race::course_places::{info_bytes, write_info};
    let what = "the course info's texture tail";
    write_info(0x17_F000, &info_bytes(0x17_E800, 0x800, what)?, what)
}

/// `race::load_texture_info` (0x4398FA): `files\a\textinfo` (0x1F4000 bytes at most) into
/// the course info, `files\m\markinfo` (0x8000) at +0x1C2000, `files\b\textinfo`
/// (0x1F4000) at +0x1F4000; three line feeds at +0x5BD21; `copy_texture_info_tail`. Refuses
/// no course info.
pub fn load_texture_info(course: &Course) -> Result<(), Refused> {
    let file_numbers = &scene().file_numbers;
    let info = course.lists.course_info.get();
    if info == 0 {
        return Err(BadInput("no course memory: the original reads into 0").into());
    }
    let read = crate::race::course_places::read_file_into_info;
    read(file_numbers.text_ptr(texts::TEXTURE_INFO_A), 0, 0x1F_4000)?;
    read(file_numbers.text_ptr(texts::MARK_INFO), 0x1C_2000, 0x8000)?;
    read(
        file_numbers.text_ptr(texts::TEXTURE_INFO_B),
        0x1F_4000,
        0x1F_4000,
    )?;
    crate::race::course_places::write_info(0x5_BD21, &[0x0A; 3], "the course info's line feeds")?;
    copy_texture_info_tail()?;
    Ok(())
}

/// `race::set_course_number` (0x43944F): al's two digits (tens, units, as `div 10` of the
/// low byte) into the nine path strings (the stage's files) that hold the course's number.
pub fn set_course_number(eax: u32) -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::stage::two_digits`.
    let [tens, units] = rc2k_core::race::stage::two_digits(eax as u8);
    for place in number_places() {
        place.write(&[tens, units], "a path's course number")?;
    }
    Ok(())
}

/// `race::load_error_list` (0x439A92): the file named at 0x60AE02, 0x2000 bytes at most, to
/// 0x637660.
pub fn load_error_list(frame: &Frame) -> Result<(), Refused> {
    crate::files::read_whole_file::read_whole_file_bytes(
        scene().file_numbers.text_ptr(texts::ERROR_LIST_PATH),
        &frame.error_list.list,
    )?;
    Ok(())
}

/// `race::build_texture_infos` (0x4393A0): nothing without course info; else for each of
/// the 256 textures, from its kind k (`dword_608DB4`) and the info's record +k * 0x80, a
/// record of 0x40 at 0x979F30: +0x24 k & 0xF, +0x20 k >> 4, +0x28 that | 0x80000000,
/// +0x10/+0x14/+0x18 the record's +0x20/+0x10/+0x14, +0x1C its byte +0x18, +0x2C..+0x2E its
/// bytes +0x24..+0x26, +0 .. +0xC its first four. Refuses a kind whose record lies past
/// the info.
pub fn build_texture_infos(course: &Course) -> Result<(), Refused> {
    {
        let info = course.lists.course_info.get();
        if info == 0 {
            return Ok(());
        }
        for n in 0..TEXTURES {
            if course.stage_prefs.texture_kinds[n as usize].get() >= INFO_PART / 0x80 {
                return Err(BadInput("a texture kind past the course info").into());
            }
        }
        // Converted: `rc2k_core::race::stage::texture_info`.
        for n in 0..TEXTURES {
            let out = &crate::scene::polygon_store::POLYGON_WORK.texture_infos[n as usize].words;
            let kind = course.stage_prefs.texture_kinds[n as usize].get();
            let rec = kind * 0x80;
            let trap = Trap::new();
            use crate::race::course_places::info_field as field;
            let t = rc2k_core::race::stage::texture_info(
                kind,
                |o| trap.keep_read(field::<u32>(rec + o, "a course info record")),
                |o| trap.keep_read(field::<u8>(rec + o, "a course info record")),
            );
            trap.finish()?;
            out[9].set(t.row);
            out[8].set(t.group);
            for (k, &value) in t.values.iter().enumerate() {
                out[4 + k].set(value);
            }
            // The record's byte at +0x1C and three at +0x2C are bytes inside their dwords.
            GamePtr::to(&out[7]).write(t.byte, "a texture info")?;
            GamePtr::to(&out[11]).write_bytes(&t.bytes, "a texture info")?;
            for (k, &value) in t.first.iter().enumerate() {
                out[k].set(value);
            }
            out[10].set(t.marked);
        }
    }
    Ok(())
}

/// `race::reset_refresh` (0x45C53B): `dword_63E174` and `dword_63E178` 0x80000000.
pub fn reset_refresh(course: &Course) {
    course.level_grid.grid_x.set(0x8000_0000);
    course.level_grid.grid_z.set(0x8000_0000);
}

/// `race::init_checkpoints` (0x45A98F): `dword_601830` 0; unless `dword_623640` - 2 is
/// below 0, the table entries 1 .. `dword_623644` - 1 (`dword_9E3738[n]`) whose record
/// (`dword_5CCCBC` + it * 0x40) has its byte +0x26 set are listed in `dword_60102C`, and
/// `dword_601830` = their count - 1. Refuses an end below 2 (the original counts round), or
/// more found than the list's 0x201 (a real stage's end is past 0x201: refused wrongly on
/// 2026-10-16, when the end was checked).
pub fn init_checkpoints(race: &Race) -> Result<(), Refused> {
    let physics = &race.physics;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::checkpoints::listed_checkpoints`.
    {
        let kinds = shared.vars.checkpoint_count.get();
        if (kinds.wrapping_sub(2) as i32) < 0 {
            physics.split_tables.split_colour_count.set(0);
            return Ok(());
        }
        let end = shared.vars.last_checkpoint.get();
        let table = &crate::race::checkpoints_store::CHECKPOINTS.checkpoint_faces;
        let trap = Trap::new();
        let listed = rc2k_core::race::checkpoints::listed_checkpoints(end, |n| {
            let face = trap.keep_read(crate::session::read_on_in(
                &shared.buffers,
                table,
                n as usize,
                "the checkpoint table",
            ));
            trap.keep_read(crate::race::course_places::face_field::<u8>(
                face,
                0x26,
                "a checkpoint face",
            )) != 0
        })?;
        trap.finish()?;
        physics.split_tables.split_colour_count.set(0);
        // By address: the list runs on past its 0x101 entries as the original's does.
        let list = &physics.split_tables.checkpoints;
        for (k, &n) in listed.iter().enumerate() {
            crate::session::write_on_in(&physics.split_tables, list, k, n, "a course variable")?;
        }
        physics
            .split_tables
            .split_colour_count
            .set((listed.len() as u32).wrapping_sub(1));
    }
    Ok(())
}
