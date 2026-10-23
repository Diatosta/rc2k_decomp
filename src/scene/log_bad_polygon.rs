//! `scene::log_bad_polygon` (0x443AF8): a polygon Direct3D was not given, dumped to the
//! log at level 1: "videox" `g_screenWidth`, "videoy" `g_screenHeight` (twice), "r" "l" "b"
//! "t" the clip edges (`g_clipScreenRight` .. `g_clipScreenTop`, two decimals), "donezcut"
//! `dword_624124`, "donesuth" `dword_624120`, "cuttest" `g_polygonClipMask` (with a line's
//! end; then the mask 0), "tpage" `dword_6240C4` (hex8), "points" `g_polygonVertexCount`;
//! "polynum" `g_currentCourseFace`, or with `g_drawingObject` 1 "objdraw"
//! `g_objectNumber` and "objfaced" `g_faceNumber`; a line's end. Then with vertices, each
//! (0x20 bytes from edi): "pt" its number, "x" +0, "yzuwv" +4, "zuwv" +8, "uwv" +0x18,
//! "wv" +0xC, "v" +0x1C (floats, two decimals; the labels are the tails of one string),
//! "co" +0x10 and "sp" +0x14 (hex8), a line's end; and a last line's end.
//!
//! In:  edi = the vertices
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `scene::show_bad_polygon` (0x443AEC). The log calls through `part` (0
//! `log_write_l1`, 1 `log_hex8_l1`, 2 `log_float_l1` with ecx 2, 3 `log_int_l1`, 4
//! `log_int_l1_nl`, 5 `log_newline_l1`; eax the text or value), in the game the Rust log
//! functions. Refuses (before anything) more than 64 vertices (the original's loop would
//! run through memory). Shadow mode runs the original alone (it writes the log).
//! Self-test: a copy whose calls go to stand-ins, against the Rust version: no vertex,
//! three, the object's and the course's labels; the mask put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::parts::{Part, parted_entry};

const WRITE: u32 = 0;
const HEX8: u32 = 1;
const FLOAT: u32 = 2;
const INT: u32 = 3;
const INT_NL: u32 = 4;
const NEWLINE: u32 = 5;
const MASK_RVA: usize = rva(0x624118); // g_polygonClipMask
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const OBJECT_RVA: usize = rva(0x62DA44); // g_drawingObject
const MOST: u32 = 64;

/// (label, kind, value): the head's lines before the mask's clearing, and after.
const HEAD: [(usize, u32, usize); 10] = [
    (0x4DBCB6, HEX8, 0x4F494C),   // "videox" g_screenWidth
    (0x4DBCBD, HEX8, 0x4F4950),   // "videoy" g_screenHeight
    (0x4DBCC4, HEX8, 0x4F4950),   // "videoy" g_screenHeight
    (0x4DBCCB, FLOAT, 0x6241F0),  // "r" g_clipScreenRight
    (0x4DBCCD, FLOAT, 0x6241EC),  // "l" g_clipScreenLeft
    (0x4DBCCF, FLOAT, 0x6241E8),  // "b" g_clipScreenBottom
    (0x4DBCD1, FLOAT, 0x6241E4),  // "t" g_clipScreenTop
    (0x4DBCD3, INT, 0x624124),    // "donezcut"
    (0x4DBCDC, INT, 0x624120),    // "donesuth"
    (0x4DBCE5, INT_NL, 0x624118), // "cuttest" g_polygonClipMask
];
const AFTER: [(usize, u32, usize); 2] = [
    (0x4DBCED, HEX8, 0x6240C4), // "tpage"
    (0x4DBCF3, INT, 0x6240F8),  // "points" g_polygonVertexCount
];
const COURSE: [(usize, u32, usize); 1] = [(0x4DBCFA, INT, 0x5CCCD0)]; // "polynum"
const OBJECT: [(usize, u32, usize); 2] = [
    (0x4DBD02, INT, 0x6214FC), // "objdraw" g_objectNumber
    (0x4DBD0A, INT, 0x623620), // "objfaced" g_faceNumber
];
/// A vertex's: (label, kind, offset).
const VERTEX: [(usize, u32, u32); 8] = [
    (0x4DBD16, FLOAT, 0),    // "x"
    (0x4DBD18, FLOAT, 4),    // "yzuwv"
    (0x4DBD1A, FLOAT, 8),    // "zuwv"
    (0x4DBD1C, FLOAT, 0x18), // "uwv"
    (0x4DBD1E, FLOAT, 0xC),  // "wv"
    (0x4DBD20, FLOAT, 0x1C), // "v"
    (0x4DBD22, HEX8, 0x10),  // "co"
    (0x4DBD25, HEX8, 0x14),  // "sp"
];
const PT: usize = 0x4DBD13; // "pt"

/// The Rust version: the registers it leaves.
pub unsafe fn log_bad_polygon(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let count: u32 = unsafe { game_read(COUNT_RVA) };
    if count > MOST {
        return Err(BadInput("more than 64 vertices").into());
    }
    // Each call gets the registers the loop has then (ecx the vertex's number, edi the
    // vertex), eax the text or value and, for a float, ecx 2.
    let mut base = *entry;
    let mut call = |base: &Regs, place: u32, eax: u32| {
        let mut r = Regs { eax, ..*base };
        if place == FLOAT {
            r.ecx = 2;
        }
        part(place, &mut r)
    };
    let text = |a: usize| game_addr(rva(a)) as u32;
    let value = |a: usize| unsafe { game_read::<u32>(rva(a)) };
    for (label, kind, at) in HEAD {
        call(&base, WRITE, text(label))?;
        call(&base, kind, value(at))?;
    }
    unsafe { game_write(MASK_RVA, 0u32) };
    let object = unsafe { game_read::<u32>(OBJECT_RVA) } == 1;
    let tail: &[(usize, u32, usize)] = if object { &OBJECT } else { &COURSE };
    for &(label, kind, at) in AFTER.iter().chain(tail) {
        call(&base, WRITE, text(label))?;
        call(&base, kind, value(at))?;
    }
    call(&base, NEWLINE, entry.eax)?;
    if count == 0 {
        return Ok(*entry);
    }
    for n in 0..count {
        base.ecx = n;
        base.edi = entry.edi.wrapping_add(0x20 * n);
        let vertex = base.edi;
        call(&base, WRITE, text(PT))?;
        call(&base, INT, n)?;
        for (label, kind, offset) in VERTEX {
            call(&base, WRITE, text(label))?;
            call(&base, kind, unsafe {
                ((vertex + offset) as *const u32).read_unaligned()
            })?;
        }
        call(&base, NEWLINE, entry.eax)?;
    }
    base.ecx = count;
    base.edi = entry.edi.wrapping_add(0x20 * count);
    call(&base, NEWLINE, entry.eax)?;
    Ok(*entry)
}

/// In the game: the Rust log functions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::log_write::log_write;
    use crate::logfile::{log_float::log_float, log_hex8::log_hex8, log_int::log_int};
    unsafe {
        match place {
            WRITE => log_write(r.eax as *const u8, false, 1),
            HEX8 => log_hex8(r.eax, false, 1),
            FLOAT => log_float(f32::from_bits(r.eax), 2, false, 1),
            INT => log_int(r.eax as i32, false, 1),
            INT_NL => log_int(r.eax as i32, true, 1),
            _ => log_write(game_addr(rva(0x4E00B7)) as *const u8, true, 1),
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    parted_entry(r, &|r, part| unsafe { log_bad_polygon(r, part) }, game_part)
}
