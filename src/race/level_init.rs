//! `race::load_stage`'s "loadlevel_init" (`sub_44426F`): its small steps (IDA,
//! 2026-10-16). The course memory (`alloc coursemem`): 0x1300000 bytes from
//! `memory::allocate` on `stru_5C6D50`, `g_courseMemory` (0x5C6F94) its start and
//! `g_courseInfo` (0x5C6FA0) its last 0x400000 bytes, where the texture and mark info
//! files are read.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, rva};

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
        let a = |v: usize| game_addr(rva(v)) as u32;
        Self {
            course_info: a(0x5C6FA0),
            course_memory: a(0x5C6F94),
            first_part: a(0x5C6FC8),
            list_total: a(0x5C6F90),
            texture_kinds: a(0x608DB4),
            texture_infos: a(0x979F30),
            checkpoint_kinds: a(0x623640),
            checkpoint_count: a(0x623644),
            checkpoint_table: a(0x9E3738),
            checkpoint_records: a(0x5CCCBC),
            checkpoints: a(0x60102C),
            checkpoints_found: a(0x601830),
            refresh: [a(0x63E174), a(0x63E178)],
        }
    }
}

pub const COURSE_LIST_RVA: usize = rva(0x5C6D50); // stru_5C6D50: the course's MemoryList
const FIRST_PART: u32 = 0xF0_0000;
const INFO_PART: u32 = 0x40_0000;
const TEXTURE_INFO_A_RVA: usize = rva(0x60AE36); // "files\a\textinfo"
const MARK_INFO_RVA: usize = rva(0x60AE47); // "files\m\markinfo"
const TEXTURE_INFO_B_RVA: usize = rva(0x60AE58); // "files\b\textinfo"
const ERROR_LIST_PATH_RVA: usize = rva(0x60AE02);
const ERROR_LIST_RVA: usize = rva(0x637660);
/// The path strings the course's number is written into: the tens' and the units' places.
pub const NUMBER_PLACES: [usize; 9] = [
    0x60B01F, 0x60B038, 0x60B08C, 0x60B0A7, 0x60B050, 0x60B06E, 0x60AE0F, 0x60AE1D, 0x60AE2F,
];
const TEXTURES: u32 = 0x100;
unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

/// `files::read_whole_file` (inside `pusha` in every caller: what it leaves is read by
/// nobody).
pub unsafe fn read(path: u32, buffer: u32, count: u32) -> Result<(), Refused> {
    unsafe {
        crate::files::read_whole_file::read_whole_file(
            path as *const u8,
            buffer as *mut u8,
            count,
        )?;
    }
    Ok(())
}

/// `race::allocate_course_memory` (0x41F8C4): the course memory taken (`memory::allocate`
/// on the list `list`).
pub unsafe fn allocate_course_memory(v: &Vars, list: u32) -> Result<(), Refused> {
    unsafe {
        put(v.first_part, FIRST_PART);
        put(
            v.list_total,
            get(v.list_total).wrapping_add(FIRST_PART + INFO_PART),
        );
        let memory = crate::memory::allocate::allocate(list as *mut _, FIRST_PART + INFO_PART)?;
        put(v.course_memory, memory as u32);
        put(v.course_info, (memory as u32).wrapping_add(FIRST_PART));
    }
    Ok(())
}

/// `race::copy_texture_info_tail` (0x439AE5): 0x800 bytes of the course info from
/// +0x17E800 to +0x17F000.
pub unsafe fn copy_texture_info_tail(v: &Vars) {
    unsafe {
        let info = get(v.course_info);
        std::ptr::copy(
            info.wrapping_add(0x17_E800) as *const u8,
            info.wrapping_add(0x17_F000) as *mut u8,
            0x800,
        );
    }
}

/// `race::load_texture_info` (0x4398FA): `files\a\textinfo` (0x1F4000 bytes at most) into
/// the course info, `files\m\markinfo` (0x8000) at +0x1C2000, `files\b\textinfo`
/// (0x1F4000) at +0x1F4000; three line feeds at +0x5BD21; `copy_texture_info_tail`. Refuses
/// no course info.
pub unsafe fn load_texture_info(v: &Vars) -> Result<(), Refused> {
    unsafe {
        let info = get(v.course_info);
        if info == 0 {
            return Err(BadInput("no course memory: the original reads into 0").into());
        }
        read(game_addr(TEXTURE_INFO_A_RVA) as u32, info, 0x1F_4000)?;
        read(game_addr(MARK_INFO_RVA) as u32, info + 0x1C_2000, 0x8000)?;
        read(
            game_addr(TEXTURE_INFO_B_RVA) as u32,
            info + 0x1F_4000,
            0x1F_4000,
        )?;
        std::ptr::write_bytes((info + 0x5_BD21) as *mut u8, 0x0A, 3);
        copy_texture_info_tail(v);
        Ok(())
    }
}

/// `race::set_course_number` (0x43944F): al's two digits (tens, units, as `div 10` of the
/// low byte) into the nine path strings (the stage's files) that hold the course's number.
pub unsafe fn set_course_number(eax: u32) {
    // Converted: `rc2k_core::race::stage::two_digits`.
    let [tens, units] = rc2k_core::race::stage::two_digits(eax as u8);
    for place in NUMBER_PLACES {
        let at = game_addr(rva(place)) as *mut u8;
        unsafe {
            *at = tens;
            *at.add(1) = units;
        }
    }
}

/// `race::load_error_list` (0x439A92): the file named at 0x60AE02, 0x2000 bytes at most, to
/// 0x637660.
pub unsafe fn load_error_list() -> Result<(), Refused> {
    unsafe {
        read(
            game_addr(ERROR_LIST_PATH_RVA) as u32,
            game_addr(ERROR_LIST_RVA) as u32,
            0x2000,
        )
    }
}

/// `race::build_texture_infos` (0x4393A0): nothing without course info; else for each of
/// the 256 textures, from its kind k (`dword_608DB4`) and the info's record +k * 0x80, a
/// record of 0x40 at 0x979F30: +0x24 k & 0xF, +0x20 k >> 4, +0x28 that | 0x80000000,
/// +0x10/+0x14/+0x18 the record's +0x20/+0x10/+0x14, +0x1C its byte +0x18, +0x2C..+0x2E its
/// bytes +0x24..+0x26, +0 .. +0xC its first four. Refuses a kind whose record lies past
/// the info.
pub unsafe fn build_texture_infos(v: &Vars) -> Result<(), Refused> {
    unsafe {
        let info = get(v.course_info);
        if info == 0 {
            return Ok(());
        }
        for n in 0..TEXTURES {
            if get(v.texture_kinds + 4 * n) >= INFO_PART / 0x80 {
                return Err(BadInput("a texture kind past the course info").into());
            }
        }
        // Converted: `rc2k_core::race::stage::texture_info`.
        for n in 0..TEXTURES {
            let out = v.texture_infos + n * 0x40;
            let kind = get(v.texture_kinds + 4 * n);
            let rec = info + kind * 0x80;
            let t = rc2k_core::race::stage::texture_info(
                kind,
                |o| get(rec + o),
                |o| *((rec + o) as *const u8),
            );
            put(out + 0x24, t.row);
            put(out + 0x20, t.group);
            for (k, &value) in t.values.iter().enumerate() {
                put(out + 0x10 + 4 * k as u32, value);
            }
            *((out + 0x1C) as *mut u8) = t.byte;
            for (k, &b) in t.bytes.iter().enumerate() {
                *((out + 0x2C + k as u32) as *mut u8) = b;
            }
            for (k, &value) in t.first.iter().enumerate() {
                put(out + 4 * k as u32, value);
            }
            put(out + 0x28, t.marked);
        }
    }
    Ok(())
}

/// `race::reset_refresh` (0x45C53B): `dword_63E174` and `dword_63E178` 0x80000000.
pub unsafe fn reset_refresh(v: &Vars) {
    unsafe {
        put(v.refresh[0], 0x8000_0000);
        put(v.refresh[1], 0x8000_0000);
    }
}

/// `race::init_checkpoints` (0x45A98F): `dword_601830` 0; unless `dword_623640` - 2 is
/// below 0, the table entries 1 .. `dword_623644` - 1 (`dword_9E3738[n]`) whose record
/// (`dword_5CCCBC` + it * 0x40) has its byte +0x26 set are listed in `dword_60102C`, and
/// `dword_601830` = their count - 1. Refuses an end below 2 (the original counts round), or
/// more found than the list's 0x201 (a real stage's end is past 0x201: refused wrongly on
/// 2026-10-16, when the end was checked).
pub unsafe fn init_checkpoints(v: &Vars) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::checkpoints::listed_checkpoints`.
    unsafe {
        let kinds = get(v.checkpoint_kinds);
        if (kinds.wrapping_sub(2) as i32) < 0 {
            put(v.checkpoints_found, 0);
            return Ok(());
        }
        let end = get(v.checkpoint_count);
        let records = get(v.checkpoint_records);
        let listed = rc2k_core::race::checkpoints::listed_checkpoints(end, |n| {
            let record = (get(v.checkpoint_table + 4 * n) << 6).wrapping_add(records);
            *((record + 0x26) as *const u8) != 0
        })?;
        put(v.checkpoints_found, 0);
        for (k, &n) in listed.iter().enumerate() {
            put(v.checkpoints + 4 * k as u32, n);
        }
        put(v.checkpoints_found, (listed.len() as u32).wrapping_sub(1));
    }
    Ok(())
}
