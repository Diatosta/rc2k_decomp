//! `race::loadlevel_init`'s "locacurrcourse" (`sub_4394CB`, IDA 2026-10-16): the course's
//! data (`cdat00c`) read into the course memory, its first 0x100 bytes the header
//! (`g_courseHeader`, 0x5CCD14: the counts of `race/norms.rs`' layout), the 0x20-byte
//! records grouped by kind, the faces' lights reset, then the best line. Here the small
//! steps.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

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
pub const KIND_TABLES_LEN: usize = 0x800;
const KINDS: u32 = 0x100;
pub const MOST_FACES: u32 = 0x10_0000;
/// The records the course memory's first part (0xF00000 bytes) can hold.
pub const MOST_RECORDS: u32 = 0x7_8000;

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

/// `race::set_cdat_number` (0x4456DD): al's two digits (`div 10`, + '0') into both cdat
/// paths. Leaves eax the tens' digit, edx the units', ebx 10.
pub unsafe fn set_cdat_number(entry: &Regs) -> Regs {
    let mut g = *entry;
    let n = g.eax & 0xFF;
    g.eax = n / 10 + 0x30;
    g.edx = n % 10 + 0x30;
    g.ebx = 10;
    for at in CDAT_DIGITS {
        unsafe { game_write(at, [g.eax as u8, g.edx as u8]) };
    }
    g
}

/// `race::copy_course_header` (0x445736): the course memory's first 0x100 bytes to
/// `g_courseHeader`. Leaves esi and edi past them, ecx 0.
pub unsafe fn copy_course_header(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        g.esi = game_read(COURSE_MEMORY_RVA);
        g.edi = game_addr(HEADER_RVA) as u32;
        std::ptr::copy(g.esi as *const u8, g.edi as *mut u8, HEADER_LEN);
    }
    g.esi = g.esi.wrapping_add(HEADER_LEN as u32);
    g.edi += HEADER_LEN as u32;
    g.ecx = 0;
    g
}

/// `race::clear_lit_faces` (0x45E9D5): `g_litFaceCount` 0.
pub unsafe fn clear_lit_faces() {
    unsafe { game_write(LIT_COUNT_RVA, 0u32) };
}

/// `race::reset_face_lights` (0x45ED70): each face whose first byte is 0xFF gets its
/// +0x30 .. +0x38 and bytes +0x11 .. +0x13 0; then `clear_lit_faces`. Leaves ebx the face
/// count, esi the last face. Refuses no faces (the original counts round) or 0x100000 or
/// more.
pub unsafe fn reset_face_lights(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let count: u32 = game_read(FACE_COUNT_RVA);
        if count == 0 || count >= MOST_FACES {
            return Err(
                BadInput("no faces, or 0x100000 or more: the original counts round").into(),
            );
        }
        let faces: u32 = game_read(FACES_RVA);
        for n in 0..count {
            g.esi = faces.wrapping_add(n << 6);
            if *(g.esi as *const u8) == 0xFF {
                for o in [0x30, 0x34, 0x38] {
                    put(g.esi + o, 0);
                }
                std::ptr::write_bytes((g.esi + 0x11) as *mut u8, 0, 3);
            }
        }
        g.ebx = count;
        clear_lit_faces();
    }
    Ok(g)
}

/// `race::group_records_by_kind` (0x421E42): the course's 0x20-byte records put in order
/// of kind (low four bits of the first dword, + bits 11 .. 14 times 16), each kind's in
/// the order met, by swapping each one found into the next place; `g_kindFirstRecords`
/// and `g_kindLastRecords` each kind's first and last number, -1 for both with none.
/// Leaves ecx 0x100, edx and eax as the last kind leaves them (the records, the last - 1),
/// ebx the count, esi the last record looked at (past it after a swap), edi past the last
/// placed, ebp the last kind's high part. Refuses more records than the course memory's
/// first part holds.
pub unsafe fn group_records_by_kind(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let count: u32 = game_read(RECORD_COUNT_RVA);
        if count > MOST_RECORDS {
            return Err(BadInput("more records than the course memory holds").into());
        }
        let records: u32 = game_read(RECORDS_RVA);
        let first = game_addr(KIND_TABLES_RVA) as u32;
        let last = first + 4 * KINDS;
        std::ptr::write_bytes(first as *mut u8, 0xFF, KIND_TABLES_LEN);
        g.esi = records;
        g.edi = records;
        g.edx = 0;
        for kind in 0..KINDS {
            g.ebx = g.edx;
            put(first + 4 * kind, g.edx);
            if g.ebx != count {
                loop {
                    g.esi = records.wrapping_add(g.ebx << 5);
                    let d = get(g.esi);
                    g.ebp = ((d >> 11) & 0xF) << 4;
                    g.eax = (d & 0xF) + g.ebp;
                    if g.eax == kind {
                        for _ in 0..8 {
                            g.eax = get(g.esi);
                            let other = get(g.edi);
                            put(g.edi, g.eax);
                            put(g.esi, other);
                            g.esi += 4;
                            g.edi += 4;
                        }
                        g.edx += 1;
                    }
                    g.ebx += 1;
                    if g.ebx == count {
                        break;
                    }
                }
            }
            g.eax = g.edx.wrapping_sub(1);
            put(last + 4 * kind, g.eax);
            if (g.eax.wrapping_sub(get(first + 4 * kind)) as i32) < 0 {
                put(first + 4 * kind, u32::MAX);
                put(last + 4 * kind, u32::MAX);
            }
        }
        g.ecx = KINDS;
    }
    Ok(g)
}

// ---- for the self-tests ----

/// Faces or records of a test's in the game's pointer and count until dropped.
pub struct TestTable {
    pub memory: Vec<u8>,
    initial: Vec<u8>,
    vars: [crate::service::harness::Region; 2],
    saved: Vec<u8>,
}

impl TestTable {
    /// `count` entries of `size` bytes (room for `room`), filled by `fill(n, bytes)`.
    pub fn new(
        pointer: usize,
        counter: usize,
        size: usize,
        count: u32,
        room: usize,
        fill: &dyn Fn(usize, &mut [u8]),
    ) -> Self {
        use crate::service::harness::{at, capture};
        let vars = [at(pointer), at(counter)];
        let saved = capture(&vars);
        let mut memory = vec![0u8; size * room];
        for (n, entry) in memory.chunks_mut(size).enumerate() {
            fill(n, entry);
        }
        unsafe {
            game_write(pointer, memory.as_ptr() as u32);
            game_write(counter, count);
        }
        Self {
            initial: memory.clone(),
            memory,
            vars,
            saved,
        }
    }

    /// The entries as `new` filled them, before each run.
    pub fn reset(&self) {
        let at = self.memory.as_ptr() as *mut u8;
        unsafe { std::ptr::copy_nonoverlapping(self.initial.as_ptr(), at, self.initial.len()) };
    }

    pub fn region(&self) -> crate::service::harness::Region {
        (self.memory.as_ptr() as u32, self.memory.len())
    }
}

impl Drop for TestTable {
    fn drop(&mut self) {
        crate::service::harness::restore(&self.vars, &self.saved);
    }
}

/// `race::load_level`'s callees by place, as IDA shows them: `set_cdat_number`,
/// `files::read_level_file`, `copy_course_header`, `race::lay_out_course`,
/// `reset_face_lights`, `logfile::log_write_l1`.
pub const LOAD_CALLEES: &[(usize, u32)] = &[
    (0x4456DD, 0),
    (0x41BA2E, 1),
    (0x445736, 2),
    (0x421BE3, 3),
    (0x45ED70, 4),
    (0x401427, 5),
];

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

const COURSE_PATH_RVA: usize = rva(0x60B02B); // "srclvls\level01\cdat00c"
const FIRST_PART_RVA: usize = rva(0x5C6FC8); // dword_5C6FC8: the course memory's first part
pub const READ_BYTES_RVA: usize = rva(0x60B2CC); // dword_60B2CC
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

/// A log line's call, inside `pusha`.
fn load_log(n: usize, g: &Regs, part: Part) -> Result<(), Refused> {
    let mut r = Regs {
        eax: game_addr(LOAD_LOG[n]) as u32,
        ..*g
    };
    part(5, &mut r)
}

/// `race::load_level` (0x4455FC): the course data (cdat 0) read into the course memory's
/// first part (`files::read_level_file`, the bytes in `dword_60B2CC`; a failed read is
/// fatal error 0x80650210, without a message), its header copied, laid out, the faces'
/// lights reset, with "loadlevel", "done", "next", "this", "then", "ok" in the log. Leaves
/// eax and edx the digits, ecx 0, ebx the face count, esi the last face, edi past the
/// header.
pub unsafe fn load_level(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = Regs { eax: 0, ..*entry };
    unsafe {
        part(0, &mut g)?;
        game_write(READ_BYTES_RVA, 0u32);
        g.esi = game_read(COURSE_MEMORY_RVA);
        load_log(0, &g, part)?;
        let mut r = Regs {
            eax: game_addr(COURSE_PATH_RVA) as u32,
            ebx: game_read(COURSE_MEMORY_RVA),
            ecx: game_read(FIRST_PART_RVA),
            ..g
        };
        part(1, &mut r)?;
        game_write(READ_BYTES_RVA, r.eax);
        if r.carry() {
            return Err(NOT_READ.into());
        }
        load_log(1, &g, part)?;
        load_log(2, &g, part)?;
        part(2, &mut g)?;
        load_log(3, &g, part)?;
        part(3, &mut g)?;
        load_log(4, &g, part)?;
        part(4, &mut g)?;
        load_log(5, &g, part)?;
    }
    Ok(g)
}

/// `load_level`'s callees in the game: the Rust versions.
pub fn load_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::files::{read_level_file, read_packed_file};
    unsafe {
        match place {
            0 => *r = set_cdat_number(r),
            1 => read_level_file::read_level_file(
                r,
                &mut read_level_file::game_exists,
                &mut read_packed_file::game_read,
            )?
            .apply(r),
            2 => *r = copy_course_header(r),
            3 => crate::race::norms::lay_out_course(),
            4 => *r = reset_face_lights(r)?,
            _ => crate::logfile::log_write::log_write(r.eax as *const u8, false, 1),
        }
    }
    Ok(())
}

/// `race::locate_current_course`'s callees by place, as IDA shows them: `load_level`,
/// `race::lay_out_course`, `group_records_by_kind`, `race::build_best_line`.
pub const LOCATE_CALLEES: &[(usize, u32)] =
    &[(0x4455FC, 0), (0x421BE3, 1), (0x421E42, 2), (0x445772, 3)];

const CDAT_NUMBER_RVA: usize = rva(0x609380); // dword_609380

/// `race::locate_current_course` (0x4394CB, "locacurrcourse"): `dword_609380` 0 (and eax
/// it), `load_level`, `lay_out_course` (its test of `dword_621514` jumps to the next
/// instruction either way: section 16), `group_records_by_kind`, `build_best_line`. Leaves
/// the registers as the last does.
pub unsafe fn locate_current_course(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        game_write(CDAT_NUMBER_RVA, 0u32);
        g.eax = 0;
        for place in 0..4 {
            part(place, &mut g)?;
        }
    }
    Ok(g)
}

/// `locate_current_course`'s callees in the game: the Rust versions.
pub fn locate_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => *r = load_level(r, &mut load_part)?,
            1 => crate::race::norms::lay_out_course(),
            2 => *r = group_records_by_kind(r)?,
            _ => {
                *r = crate::race::best_line::build_best_line(
                    r,
                    &mut crate::race::best_line::build_part,
                )?
            }
        }
    }
    Ok(())
}
