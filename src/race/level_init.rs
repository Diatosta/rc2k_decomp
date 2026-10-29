//! `race::load_stage`'s "loadlevel_init" (`sub_44426F`): its small steps (IDA,
//! 2026-10-16). The course memory (`alloc coursemem`): 0x1300000 bytes from
//! `memory::allocate` on `stru_5C6D50`, `g_courseMemory` (0x5C6F94) its start and
//! `g_courseInfo` (0x5C6FA0) its last 0x400000 bytes, where the texture and mark info
//! files are read.

use crate::errors::{BadInput, Refused};
use crate::physics::split_tables_store::SPLIT_TABLES;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::level_grid_store::LEVEL_GRID;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::trap::Trap;
use crate::race::vars_store::RACE_VARS;
use crate::scene::file_numbers_store::{FILE_NUMBERS, texts};

const FIRST_PART: u32 = 0xF0_0000;
const INFO_PART: u32 = 0x40_0000;
/// The path strings the course's number is written into: the tens' and the units' places.
pub fn number_places() -> [crate::place::Place; 9] {
    use crate::place::Place;
    let p = &crate::race::level_paths_store::LEVEL_PATHS;
    [
        Place::from(&p.edit_path).add(0xD),
        Place::from(&p.cdat_src).add(0xD),
        Place::from(&p.cdat_files).add(0x12),
        Place::from(&p.cdat_src_2).add(0xD),
        Place::from(&p.localdat_src).add(0xD),
        Place::from(&p.localdat_files).add(0x12),
        FILE_NUMBERS.text_addr(texts::NUMBER_PLACE_A).into(),
        FILE_NUMBERS.text_addr(texts::NUMBER_PLACE_B).into(),
        FILE_NUMBERS.text_addr(texts::NUMBER_PLACE_C).into(),
    ]
}
const TEXTURES: u32 = 0x100;
fn put(at: u32, value: u32) -> Result<(), BadInput> {
    GamePtr::new(at).write(value, "a course variable")
}

/// `race::allocate_course_memory` (0x41F8C4): the course memory taken (`memory::allocate`
/// on the list `list`).
pub fn allocate_course_memory(list: u32) -> Result<(), Refused> {
    COURSE_LISTS.first_part.set(FIRST_PART);
    COURSE_LISTS.course_bytes_total.set(
        COURSE_LISTS
            .course_bytes_total
            .get()
            .wrapping_add(FIRST_PART + INFO_PART),
    );
    // Counted whole, as the original's one block; the info part is Rust's own store
    // (`race::course_places::INFO`), named by its place.
    let memory = take_course_memory(list, FIRST_PART + INFO_PART, FIRST_PART)?;
    COURSE_LISTS.course_memory.set(memory);
    COURSE_LISTS
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
    // Declared first: the region goes before the bytes (`pointers::Lent`).
    _lent: crate::pointers::Lent,
    _bytes: Vec<u8>,
}

static COURSE_MEMORY: std::sync::Mutex<Option<CourseMemory>> = std::sync::Mutex::new(None);

/// `bytes` of course memory counted on `list`, the first `kept` of them held: their game
/// address.
fn take_course_memory(list: u32, bytes: u32, kept: u32) -> Result<u32, Refused> {
    let mut memory = crate::memory::owned::counted(
        crate::pointers::host(list) as *mut crate::memory::MemoryList,
        bytes,
    )?;
    memory.truncate(kept as usize);
    memory.shrink_to_fit();
    let lent = crate::pointers::Lent::new(&memory);
    let at = lent.addr();
    *COURSE_MEMORY.lock().unwrap_or_else(|e| e.into_inner()) = Some(CourseMemory {
        _lent: lent,
        _bytes: memory,
    });
    Ok(at)
}

/// The course memory given back (the list's `free_list` counts it down).
pub fn give_back_course_memory() {
    *COURSE_MEMORY.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

/// `race::copy_texture_info_tail` (0x439AE5): 0x800 bytes of the course info from
/// +0x17E800 to +0x17F000.
pub fn copy_texture_info_tail() -> Result<(), BadInput> {
    let info = COURSE_LISTS.course_info.get();
    let what = "the course info's texture tail";
    let tail = crate::race::course_places::bytes(info.wrapping_add(0x17_E800), 0x800, what)?;
    crate::race::course_places::write_bytes(info.wrapping_add(0x17_F000), &tail, what)
}

/// `race::load_texture_info` (0x4398FA): `files\a\textinfo` (0x1F4000 bytes at most) into
/// the course info, `files\m\markinfo` (0x8000) at +0x1C2000, `files\b\textinfo`
/// (0x1F4000) at +0x1F4000; three line feeds at +0x5BD21; `copy_texture_info_tail`. Refuses
/// no course info.
pub fn load_texture_info() -> Result<(), Refused> {
    let info = COURSE_LISTS.course_info.get();
    if info == 0 {
        return Err(BadInput("no course memory: the original reads into 0").into());
    }
    let read = crate::race::course_places::read_file_into_info;
    read(
        FILE_NUMBERS.text_addr(texts::TEXTURE_INFO_A),
        info,
        0x1F_4000,
    )?;
    read(
        FILE_NUMBERS.text_addr(texts::MARK_INFO),
        info + 0x1C_2000,
        0x8000,
    )?;
    read(
        FILE_NUMBERS.text_addr(texts::TEXTURE_INFO_B),
        info + 0x1F_4000,
        0x1F_4000,
    )?;
    crate::race::course_places::write_bytes(
        info + 0x5_BD21,
        &[0x0A; 3],
        "the course info's line feeds",
    )?;
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
pub fn load_error_list() -> Result<(), Refused> {
    crate::files::read_whole_file::read_whole_file_bytes(
        crate::pointers::host(FILE_NUMBERS.text_addr(texts::ERROR_LIST_PATH)),
        &crate::race::error_list_store::ERROR_LIST.list,
    )?;
    Ok(())
}

/// `race::build_texture_infos` (0x4393A0): nothing without course info; else for each of
/// the 256 textures, from its kind k (`dword_608DB4`) and the info's record +k * 0x80, a
/// record of 0x40 at 0x979F30: +0x24 k & 0xF, +0x20 k >> 4, +0x28 that | 0x80000000,
/// +0x10/+0x14/+0x18 the record's +0x20/+0x10/+0x14, +0x1C its byte +0x18, +0x2C..+0x2E its
/// bytes +0x24..+0x26, +0 .. +0xC its first four. Refuses a kind whose record lies past
/// the info.
pub fn build_texture_infos() -> Result<(), Refused> {
    {
        let info = COURSE_LISTS.course_info.get();
        if info == 0 {
            return Ok(());
        }
        for n in 0..TEXTURES {
            if STAGE_PREFS.texture_kinds[n as usize].get() >= INFO_PART / 0x80 {
                return Err(BadInput("a texture kind past the course info").into());
            }
        }
        // Converted: `rc2k_core::race::stage::texture_info`.
        for n in 0..TEXTURES {
            let out = crate::scene::polygon::texture_infos_addr() + n * 0x40;
            let kind = STAGE_PREFS.texture_kinds[n as usize].get();
            let rec = info + kind * 0x80;
            let trap = Trap::new();
            let t = rc2k_core::race::stage::texture_info(
                kind,
                |o| trap.course::<u32>(rec + o, "a course info record"),
                |o| trap.course::<u8>(rec + o, "a course info record"),
            );
            trap.finish()?;
            put(out + 0x24, t.row)?;
            put(out + 0x20, t.group)?;
            for (k, &value) in t.values.iter().enumerate() {
                put(out + 0x10 + 4 * k as u32, value)?;
            }
            GamePtr::new(out + 0x1C).write(t.byte, "a texture info")?;
            GamePtr::new(out + 0x2C).write_bytes(&t.bytes, "a texture info")?;
            for (k, &value) in t.first.iter().enumerate() {
                put(out + 4 * k as u32, value)?;
            }
            put(out + 0x28, t.marked)?;
        }
    }
    Ok(())
}

/// `race::reset_refresh` (0x45C53B): `dword_63E174` and `dword_63E178` 0x80000000.
pub fn reset_refresh() {
    LEVEL_GRID.grid_x.set(0x8000_0000);
    LEVEL_GRID.grid_z.set(0x8000_0000);
}

/// `race::init_checkpoints` (0x45A98F): `dword_601830` 0; unless `dword_623640` - 2 is
/// below 0, the table entries 1 .. `dword_623644` - 1 (`dword_9E3738[n]`) whose record
/// (`dword_5CCCBC` + it * 0x40) has its byte +0x26 set are listed in `dword_60102C`, and
/// `dword_601830` = their count - 1. Refuses an end below 2 (the original counts round), or
/// more found than the list's 0x201 (a real stage's end is past 0x201: refused wrongly on
/// 2026-10-16, when the end was checked).
pub fn init_checkpoints() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::checkpoints::listed_checkpoints`.
    {
        let kinds = RACE_VARS.checkpoint_count.get();
        if (kinds.wrapping_sub(2) as i32) < 0 {
            SPLIT_TABLES.split_colour_count.set(0);
            return Ok(());
        }
        let end = RACE_VARS.last_checkpoint.get();
        let records = COURSE.course_faces.get();
        let table = crate::race::checkpoints_store::CHECKPOINTS.checkpoint_faces[0].addr();
        let trap = Trap::new();
        let listed = rc2k_core::race::checkpoints::listed_checkpoints(end, |n| {
            let face = trap.read::<u32>(table + 4 * n, "the checkpoint table");
            let record = (face << 6).wrapping_add(records);
            trap.course::<u8>(record + 0x26, "a checkpoint face") != 0
        })?;
        trap.finish()?;
        SPLIT_TABLES.split_colour_count.set(0);
        // By address: the list runs on past its 0x101 entries as the original's does.
        let list = SPLIT_TABLES.checkpoints[0].addr();
        for (k, &n) in listed.iter().enumerate() {
            put(list + 4 * k as u32, n)?;
        }
        SPLIT_TABLES
            .split_colour_count
            .set((listed.len() as u32).wrapping_sub(1));
    }
    Ok(())
}
