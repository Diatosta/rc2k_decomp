//! `race::load_stage`'s "loadlevel_init" (`sub_44426F`): its small steps (IDA,
//! 2026-10-16). The course memory (`alloc coursemem`): 0x1300000 bytes from
//! `memory::allocate` on `stru_5C6D50`, `g_courseMemory` (0x5C6F94) its start and
//! `g_courseInfo` (0x5C6FA0) its last 0x400000 bytes, where the texture and mark info
//! files are read.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};

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
/// A call: 0 `memory::allocate` (eax the bytes, ebx the list: eax the memory), 1
/// `files::read_whole_file` (eax the path, ebx the buffer, ecx the count; `pusha` around
/// it in every caller), 2 `scene::allocate_point_records`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

/// A read, its registers kept as `pusha`/`popa` keep them.
fn read(g: &Regs, path: u32, buffer: u32, count: u32, part: Part) -> Result<(), Refused> {
    let mut r = Regs {
        eax: path,
        ebx: buffer,
        ecx: count,
        ..*g
    };
    part(1, &mut r)
}

/// `race::allocate_course_memory` (0x41F8C4). Leaves eax the end of the memory and ebx the
/// list, the rest as `memory::allocate` leaves them.
pub unsafe fn allocate_course_memory(
    entry: &Regs,
    v: &Vars,
    list: u32,
    part: Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        put(v.first_part, FIRST_PART);
        g.eax = FIRST_PART + INFO_PART;
        g.ebx = list;
        put(v.list_total, get(v.list_total).wrapping_add(g.eax));
        part(0, &mut g)?;
        put(v.course_memory, g.eax);
        g.eax = g.eax.wrapping_add(FIRST_PART);
        put(v.course_info, g.eax);
        g.eax = g.eax.wrapping_add(INFO_PART);
    }
    Ok(g)
}

/// `race::copy_texture_info_tail` (0x439AE5): 0x800 bytes of the course info from
/// +0x17E800 to +0x17F000. Leaves esi and edi past them, ecx 0.
pub unsafe fn copy_texture_info_tail(entry: &Regs, v: &Vars) -> Regs {
    let mut g = *entry;
    unsafe {
        let info = get(v.course_info);
        g.esi = info.wrapping_add(0x17_E800);
        g.edi = info.wrapping_add(0x17_F000);
        std::ptr::copy(g.esi as *const u8, g.edi as *mut u8, 0x800);
        g.esi += 0x800;
        g.edi += 0x800;
        g.ecx = 0;
    }
    g
}

/// `race::load_texture_info` (0x4398FA): `files\a\textinfo` (0x1F4000 bytes at most) into
/// the course info, `files\m\markinfo` (0x8000) at +0x1C2000, `files\b\textinfo`
/// (0x1F4000) at +0x1F4000; three line feeds at +0x5BD21; `copy_texture_info_tail`. Refuses
/// no course info.
pub unsafe fn load_texture_info(entry: &Regs, v: &Vars, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let info = get(v.course_info);
        if info == 0 {
            return Err(BadInput("no course memory: the original reads into 0").into());
        }
        g.esi = info;
        read(
            &g,
            game_addr(TEXTURE_INFO_A_RVA) as u32,
            info,
            0x1F_4000,
            part,
        )?;
        g.esi = info + 0x1C_2000;
        read(&g, game_addr(MARK_INFO_RVA) as u32, g.esi, 0x8000, part)?;
        g.esi = info + 0x1F_4000;
        read(
            &g,
            game_addr(TEXTURE_INFO_B_RVA) as u32,
            g.esi,
            0x1F_4000,
            part,
        )?;
        g.esi = info;
        std::ptr::write_bytes((info + 0x5_BD21) as *mut u8, 0x0A, 3);
        part(3, &mut g)?;
        Ok(g)
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
pub fn load_error_list(entry: &Regs, part: Part) -> Result<(), Refused> {
    read(
        entry,
        game_addr(ERROR_LIST_PATH_RVA) as u32,
        game_addr(ERROR_LIST_RVA) as u32,
        0x2000,
        part,
    )
}

/// `race::build_texture_infos` (0x4393A0): nothing without course info; else for each of
/// the 256 textures, from its kind k (`dword_608DB4`) and the info's record +k * 0x80, a
/// record of 0x40 at 0x979F30: +0x24 k & 0xF, +0x20 k >> 4, +0x28 that | 0x80000000,
/// +0x10/+0x14/+0x18 the record's +0x20/+0x10/+0x14, +0x1C its byte +0x18, +0x2C..+0x2E its
/// bytes +0x24..+0x26, +0 .. +0xC its first four. Leaves ecx 0x100, esi the last record,
/// edi and eax as the last pass leaves them. Refuses a kind whose record lies past the info.
pub unsafe fn build_texture_infos(entry: &Regs, v: &Vars) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let info = get(v.course_info);
        if info == 0 {
            return Ok(g);
        }
        for n in 0..TEXTURES {
            if get(v.texture_kinds + 4 * n) >= INFO_PART / 0x80 {
                return Err(BadInput("a texture kind past the course info").into());
            }
        }
        for n in 0..TEXTURES {
            let out = v.texture_infos + n * 0x40;
            let kind = get(v.texture_kinds + 4 * n);
            put(out + 0x24, kind & 0xF);
            put(out + 0x20, kind >> 4);
            let rec = info + kind * 0x80;
            put(out + 0x10, get(rec + 0x20));
            put(out + 0x14, get(rec + 0x10));
            put(out + 0x18, get(rec + 0x14));
            let byte = |at: u32| *(at as *const u8);
            *((out + 0x1C) as *mut u8) = byte(rec + 0x18);
            for k in 0..3 {
                *((out + 0x2C + k) as *mut u8) = byte(rec + 0x24 + k);
            }
            for k in 0..4 {
                put(out + 4 * k, get(rec + 4 * k));
            }
            let marked = (kind >> 4) | 0x8000_0000;
            put(out + 0x28, marked);
            g.esi = out;
            g.edi = rec;
            g.eax = marked;
        }
        g.ecx = TEXTURES;
    }
    Ok(g)
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
/// `dword_601830` = their count - 1. Leaves edx the end, ebp the count - 1, ebx and esi
/// the last entry's, al its byte. Refuses an end below 2 (the original counts round), or
/// more found than the list's 0x201 (a real stage's end is past 0x201: refused wrongly on
/// 2026-10-16, when the end was checked).
pub unsafe fn init_checkpoints(entry: &Regs, v: &Vars) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::checkpoints::listed_checkpoints`.
    let mut g = *entry;
    unsafe {
        let kinds = get(v.checkpoint_kinds);
        if (kinds.wrapping_sub(2) as i32) < 0 {
            put(v.checkpoints_found, 0);
            return Ok(g);
        }
        let end = get(v.checkpoint_count);
        let records = get(v.checkpoint_records);
        let listed = rc2k_core::race::checkpoints::listed_checkpoints(end, |n| {
            g.ebx = get(v.checkpoint_table + 4 * n);
            g.esi = (g.ebx << 6).wrapping_add(records);
            let byte = *((g.esi + 0x26) as *const u8);
            g.eax = g.eax & !0xFF | byte as u32;
            byte != 0
        })?;
        put(v.checkpoints_found, 0);
        for (k, &n) in listed.iter().enumerate() {
            put(v.checkpoints + 4 * k as u32, n);
        }
        g.edx = end;
        g.ebp = (listed.len() as u32).wrapping_sub(1);
        put(v.checkpoints_found, g.ebp);
    }
    Ok(g)
}

/// The game's calls: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => {
                r.eax = crate::memory::allocate::allocate(r.ebx as *mut _, r.eax)? as u32;
                r.set_carry(false);
            }
            1 => {
                let left = crate::files::read_whole_file::read_whole_file(
                    r.eax as *const u8,
                    r.ebx as *mut u8,
                    r.ecx,
                    r.ebx,
                )?;
                left.apply(r);
            }
            2 => {
                crate::scene::allocate_point_records::allocate_point_records_in_game()?.apply(r);
            }
            _ => *r = copy_texture_info_tail(r, &Vars::game()),
        }
    }
    Ok(())
}

// ---- for the self-tests ----

/// The test's variables and memory: every `Vars` field a dword of `words`, the course
/// info (0x180000 bytes) and the checkpoint records (0x1000) of their own.
pub struct TestMemory {
    pub words: Box<[u32; 32]>,
    pub info: Vec<u8>,
    pub records: Vec<u8>,
    pub tables: Box<[u32; 0x1000]>,
}

/// The variables' addresses as IDA shows them, in the order of `TestMemory::vars`' slots.
pub const VARIABLES: [usize; 16] = [
    0x5C6FA0, 0x5C6F94, 0x5C6FC8, 0x5C6F90, 0x608DB4, 0x979F30, 0x623640, 0x623644, 0x9E3738,
    0x5CCCBC, 0x60102C, 0x601830, 0x63E174, 0x63E178, 0x5C6D50, 0,
];

impl TestMemory {
    pub fn new() -> Self {
        Self {
            words: Box::new([0; 32]),
            info: vec![0; 0x18_0000],
            records: vec![0; 0x1000],
            tables: Box::new([0; 0x1000]),
        }
    }

    fn slot(&self, n: usize) -> u32 {
        self.words.as_ptr() as u32 + 4 * n as u32
    }

    /// Where the tables are in `tables`: the texture kinds at +0, the checkpoint table at
    /// +0x1000 (0x800 dwords), the list at +0x3000 (0x400); the texture records need 0x4000: they go
    /// in the info's last part.
    pub fn vars(&self) -> Vars {
        let t = self.tables.as_ptr() as u32;
        Vars {
            course_info: self.slot(0),
            course_memory: self.slot(1),
            first_part: self.slot(2),
            list_total: self.slot(3),
            texture_kinds: t,
            texture_infos: self.info.as_ptr() as u32 + 0x17_0000,
            checkpoint_kinds: self.slot(6),
            checkpoint_count: self.slot(7),
            checkpoint_table: t + 0x1000,
            checkpoint_records: self.slot(9),
            checkpoints: t + 0x3000,
            checkpoints_found: self.slot(11),
            refresh: [self.slot(12), self.slot(13)],
        }
    }

    /// The test's address of each variable, for `test_copy::make`.
    pub fn patches(&self) -> Vec<(usize, u32)> {
        let v = self.vars();
        let list = self.slot(14);
        let all = [
            v.course_info,
            v.course_memory,
            v.first_part,
            v.list_total,
            v.texture_kinds,
            v.texture_infos,
            v.checkpoint_kinds,
            v.checkpoint_count,
            v.checkpoint_table,
            v.checkpoint_records,
            v.checkpoints,
            v.checkpoints_found,
            v.refresh[0],
            v.refresh[1],
            list,
        ];
        VARIABLES[..15].iter().copied().zip(all).collect()
    }

    /// Everything the steps can write, for comparing.
    pub fn snapshot(&self) -> Vec<u8> {
        let mut s: Vec<u8> = self.words.iter().flat_map(|w| w.to_le_bytes()).collect();
        s.extend(self.tables.iter().flat_map(|w| w.to_le_bytes()));
        s.extend_from_slice(&self.info);
        s.extend_from_slice(&self.records);
        s
    }
}

impl Default for TestMemory {
    fn default() -> Self {
        Self::new()
    }
}

static CALLED: std::sync::Mutex<Vec<(u32, u32, u32, u32)>> = std::sync::Mutex::new(Vec::new());
/// What the allocation stand-in gives.
pub static GIVES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The stand-ins: each call noted with eax, ebx, ecx; the allocation gives `GIVES`.
pub fn note(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((place, r.eax, r.ebx, r.ecx));
    if place == 0 {
        r.eax = GIVES.load(std::sync::atomic::Ordering::SeqCst);
    }
}

pub fn take_called() -> Vec<(u32, u32, u32, u32)> {
    std::mem::take(&mut *CALLED.lock().unwrap())
}

/// The parts' callees by place, as IDA shows them: `memory::allocate`,
/// `files::read_whole_file`, `scene::allocate_point_records`, `race::copy_texture_info_tail`.
const CALLEES: [usize; 4] = [0x4028A7, 0x421B54, 0x41F976, 0x439AE5];

/// A copy of a step whose variables are the test's and whose calls go to stand-ins by
/// place.
pub fn copy_of(
    m: &TestMemory,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    references: usize,
) -> Result<crate::redirect::RedirectedCopy, String> {
    crate::install::stand_ins::set_handler(note);
    crate::audio::test_copy::make(
        &crate::race::create_controls::HOOK,
        &crate::audio::test_copy::Copy {
            address,
            len,
            entry,
            calls,
            jumps_out: &[],
        },
        &|c| {
            CALLEES
                .iter()
                .position(|&k| rva(k) == c)
                .map(|n| crate::install::stand_ins::stand_in(n as u32))
        },
        &m.patches(),
        references,
    )
}

/// One case: `prepare` sets the test's memory, the copy runs, `prepare` again, the Rust
/// version runs; the registers left (but esp and the flags), the calls noted and the
/// test's memory compared.
pub fn both(
    t: &mut crate::framework::SelfTest,
    what: &str,
    copy: &crate::redirect::RedirectedCopy,
    m: &TestMemory,
    prepare: &dyn Fn(),
    rust: &dyn Fn(&Regs) -> Result<Regs, Refused>,
) {
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    let before = Regs::SENTINEL;
    prepare();
    take_called();
    let mut ro = before;
    unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
    let theirs = (Some(regs(&ro)), take_called(), m.snapshot());
    prepare();
    let mine = rust(&before);
    let mine = (mine.as_ref().ok().map(regs), take_called(), m.snapshot());
    t.check(theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
}

/// The Rust side's part in the tests: the stand-ins' noting.
pub fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}
