//! `race::load_stage`'s "loadlevel_init" (`sub_44426F`): its small steps (IDA,
//! 2026-10-16). The course memory (`alloc coursemem`): 0x1300000 bytes from
//! `memory::allocate` on `stru_5C6D50`, `g_courseMemory` (0x5C6F94) its start and
//! `g_courseInfo` (0x5C6FA0) its last 0x400000 bytes, where the texture and mark info
//! files are read.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, rva};
use crate::physics::split_tables_store::SPLIT_TABLES;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::level_grid_store::LEVEL_GRID;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::trap::{Trap, span};
use crate::race::vars_store::RACE_VARS;
use crate::scene::file_numbers_store::{FILE_NUMBERS, texts};

/// Where the steps' variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct Vars {
    pub course_info: u32,
    pub course_memory: u32,
    pub first_part: u32,
    pub list_total: u32,
    pub texture_kinds: u32,
    pub texture_infos: u32,
    pub checkpoint_kinds: u32,
    pub checkpoint_count: u32,
    pub checkpoint_table: u32,
    pub checkpoint_records: u32,
    pub checkpoints: u32,
    pub checkpoints_found: u32,
    pub refresh: [u32; 2],
}

impl Vars {
    pub fn game() -> Self {
        let _a = |v: usize| game_addr(rva(v));
        Self {
            course_info: (COURSE_LISTS.course_info.addr()),
            course_memory: (COURSE_LISTS.course_memory.addr()),
            first_part: (COURSE_LISTS.first_part.addr()),
            list_total: (COURSE_LISTS.course_bytes_total.addr()),
            texture_kinds: (STAGE_PREFS.texture_kinds[0].addr()),
            texture_infos: crate::scene::polygon::texture_infos_addr(),
            checkpoint_kinds: (RACE_VARS.checkpoint_count.addr()),
            checkpoint_count: (RACE_VARS.last_checkpoint.addr()),
            checkpoint_table: crate::race::checkpoints_store::CHECKPOINTS.checkpoint_faces[0]
                .addr(),
            checkpoint_records: (COURSE.course_faces.addr()),
            checkpoints: (SPLIT_TABLES.checkpoints[0].addr()),
            checkpoints_found: (SPLIT_TABLES.split_colour_count.addr()),
            refresh: [(LEVEL_GRID.grid_x.addr()), (LEVEL_GRID.grid_z.addr())],
        }
    }
}

const FIRST_PART: u32 = 0xF0_0000;
const INFO_PART: u32 = 0x40_0000;
/// The path strings the course's number is written into: the tens' and the units' places.
pub fn number_places() -> [usize; 9] {
    let p = &crate::race::level_paths_store::LEVEL_PATHS;
    [
        (p.edit_path.addr() + 0xD) as usize,
        (p.cdat_src.addr() + 0xD) as usize,
        (p.cdat_files.addr() + 0x12) as usize,
        (p.cdat_src_2.addr() + 0xD) as usize,
        (p.localdat_src.addr() + 0xD) as usize,
        (p.localdat_files.addr() + 0x12) as usize,
        FILE_NUMBERS.text_addr(texts::NUMBER_PLACE_A) as usize,
        FILE_NUMBERS.text_addr(texts::NUMBER_PLACE_B) as usize,
        FILE_NUMBERS.text_addr(texts::NUMBER_PLACE_C) as usize,
    ]
}
const TEXTURES: u32 = 0x100;
fn get(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read("a course variable")
}

fn put(at: u32, value: u32) -> Result<(), BadInput> {
    GamePtr::new(at).write(value, "a course variable")
}

/// `files::read_whole_file` (inside `pusha` in every caller: what it leaves is read by
/// nobody).
pub fn read(path: u32, buffer: u32, count: u32) -> Result<(), Refused> {
    span(buffer, count as usize, "a file's buffer")?;
    crate::files::read_whole_file::read_whole_file(
        crate::pointers::host(path) as *const u8,
        crate::pointers::host(buffer),
        count,
    )?;
    Ok(())
}

/// `race::allocate_course_memory` (0x41F8C4): the course memory taken (`memory::allocate`
/// on the list `list`).
pub fn allocate_course_memory(v: &Vars, list: u32) -> Result<(), Refused> {
    put(v.first_part, FIRST_PART)?;
    put(
        v.list_total,
        get(v.list_total)?.wrapping_add(FIRST_PART + INFO_PART),
    )?;
    let memory = {
        crate::memory::allocate::allocate(
            crate::pointers::host(list) as *mut _,
            FIRST_PART + INFO_PART,
        )
    }?;
    put(v.course_memory, crate::pointers::game_address(memory))?;
    put(
        v.course_info,
        crate::pointers::game_address(memory).wrapping_add(FIRST_PART),
    )?;
    Ok(())
}

/// `race::copy_texture_info_tail` (0x439AE5): 0x800 bytes of the course info from
/// +0x17E800 to +0x17F000.
pub fn copy_texture_info_tail(v: &Vars) -> Result<(), BadInput> {
    let info = get(v.course_info)?;
    GamePtr::new(info.wrapping_add(0x17_E800)).copy_to(
        GamePtr::new(info.wrapping_add(0x17_F000)),
        0x800,
        "the course info's texture tail",
    )
}

/// `race::load_texture_info` (0x4398FA): `files\a\textinfo` (0x1F4000 bytes at most) into
/// the course info, `files\m\markinfo` (0x8000) at +0x1C2000, `files\b\textinfo`
/// (0x1F4000) at +0x1F4000; three line feeds at +0x5BD21; `copy_texture_info_tail`. Refuses
/// no course info.
pub fn load_texture_info(v: &Vars) -> Result<(), Refused> {
    let info = get(v.course_info)?;
    if info == 0 {
        return Err(BadInput("no course memory: the original reads into 0").into());
    }
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
    GamePtr::new(info + 0x5_BD21).fill(0x0A, 3, "the course info's line feeds")?;
    copy_texture_info_tail(v)?;
    Ok(())
}

/// `race::set_course_number` (0x43944F): al's two digits (tens, units, as `div 10` of the
/// low byte) into the nine path strings (the stage's files) that hold the course's number.
pub fn set_course_number(eax: u32) -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::stage::two_digits`.
    let [tens, units] = rc2k_core::race::stage::two_digits(eax as u8);
    for place in number_places() {
        GamePtr::new(place as u32).write_bytes(&[tens, units], "a path's course number")?;
    }
    Ok(())
}

/// `race::load_error_list` (0x439A92): the file named at 0x60AE02, 0x2000 bytes at most, to
/// 0x637660.
pub fn load_error_list() -> Result<(), Refused> {
    read(
        FILE_NUMBERS.text_addr(texts::ERROR_LIST_PATH),
        crate::race::error_list_store::ERROR_LIST.list.addr(),
        0x2000,
    )
}

/// `race::build_texture_infos` (0x4393A0): nothing without course info; else for each of
/// the 256 textures, from its kind k (`dword_608DB4`) and the info's record +k * 0x80, a
/// record of 0x40 at 0x979F30: +0x24 k & 0xF, +0x20 k >> 4, +0x28 that | 0x80000000,
/// +0x10/+0x14/+0x18 the record's +0x20/+0x10/+0x14, +0x1C its byte +0x18, +0x2C..+0x2E its
/// bytes +0x24..+0x26, +0 .. +0xC its first four. Refuses a kind whose record lies past
/// the info.
pub fn build_texture_infos(v: &Vars) -> Result<(), Refused> {
    {
        let info = get(v.course_info)?;
        if info == 0 {
            return Ok(());
        }
        for n in 0..TEXTURES {
            if get(v.texture_kinds + 4 * n)? >= INFO_PART / 0x80 {
                return Err(BadInput("a texture kind past the course info").into());
            }
        }
        // Converted: `rc2k_core::race::stage::texture_info`.
        for n in 0..TEXTURES {
            let out = v.texture_infos + n * 0x40;
            let kind = get(v.texture_kinds + 4 * n)?;
            let rec = info + kind * 0x80;
            let trap = Trap::new();
            let t = rc2k_core::race::stage::texture_info(
                kind,
                |o| trap.read::<u32>(rec + o, "a course info record"),
                |o| trap.read::<u8>(rec + o, "a course info record"),
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
pub fn reset_refresh(v: &Vars) -> Result<(), BadInput> {
    put(v.refresh[0], 0x8000_0000)?;
    put(v.refresh[1], 0x8000_0000)
}

/// `race::init_checkpoints` (0x45A98F): `dword_601830` 0; unless `dword_623640` - 2 is
/// below 0, the table entries 1 .. `dword_623644` - 1 (`dword_9E3738[n]`) whose record
/// (`dword_5CCCBC` + it * 0x40) has its byte +0x26 set are listed in `dword_60102C`, and
/// `dword_601830` = their count - 1. Refuses an end below 2 (the original counts round), or
/// more found than the list's 0x201 (a real stage's end is past 0x201: refused wrongly on
/// 2026-10-16, when the end was checked).
pub fn init_checkpoints(v: &Vars) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::checkpoints::listed_checkpoints`.
    {
        let kinds = get(v.checkpoint_kinds)?;
        if (kinds.wrapping_sub(2) as i32) < 0 {
            put(v.checkpoints_found, 0)?;
            return Ok(());
        }
        let end = get(v.checkpoint_count)?;
        let records = get(v.checkpoint_records)?;
        let trap = Trap::new();
        let listed = rc2k_core::race::checkpoints::listed_checkpoints(end, |n| {
            let face = trap.read::<u32>(v.checkpoint_table + 4 * n, "the checkpoint table");
            let record = (face << 6).wrapping_add(records);
            trap.course::<u8>(record + 0x26, "a checkpoint face") != 0
        })?;
        trap.finish()?;
        put(v.checkpoints_found, 0)?;
        for (k, &n) in listed.iter().enumerate() {
            put(v.checkpoints + 4 * k as u32, n)?;
        }
        put(v.checkpoints_found, (listed.len() as u32).wrapping_sub(1))?;
    }
    Ok(())
}
