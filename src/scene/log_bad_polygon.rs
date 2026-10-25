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
//! Caller: `scene::show_bad_polygon` (0x443AEC). Refuses (before anything) more than 64
//! vertices (the original's loop would run through memory).

use crate::errors::BadInput;
use crate::framework::{game_read, game_write, rva};

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
/// Stand-ins in the value column for the session's screen size (not addresses).
const SCREEN_WIDTH: usize = 1;
const SCREEN_HEIGHT: usize = 2;

/// (label, kind, value): the head's lines before the mask's clearing, and after.
const HEAD: [(&std::ffi::CStr, u32, usize); 10] = [
    (c"videox", HEX8, SCREEN_WIDTH),  // "videox" g_screenWidth
    (c"videoy", HEX8, SCREEN_HEIGHT), // "videoy" g_screenHeight
    (c"videoy", HEX8, SCREEN_HEIGHT), // "videoy" g_screenHeight
    (c"r", FLOAT, 0x6241F0),          // "r" g_clipScreenRight
    (c"l", FLOAT, 0x6241EC),          // "l" g_clipScreenLeft
    (c"b", FLOAT, 0x6241E8),          // "b" g_clipScreenBottom
    (c"t", FLOAT, 0x6241E4),          // "t" g_clipScreenTop
    (c"donezcut", INT, 0x624124),     // "donezcut"
    (c"donesuth", INT, 0x624120),     // "donesuth"
    (c"cuttest", INT_NL, 0x624118),   // "cuttest" g_polygonClipMask
];
const AFTER: [(&std::ffi::CStr, u32, usize); 2] = [
    (c"tpage", HEX8, 0x6240C4), // "tpage"
    (c"points", INT, 0x6240F8), // "points" g_polygonVertexCount
];
const COURSE: [(&std::ffi::CStr, u32, usize); 1] = [(c"polynum", INT, 0x5CCCD0)]; // "polynum"
const OBJECT: [(&std::ffi::CStr, u32, usize); 2] = [
    (c"objdraw", INT, 0x6214FC),  // "objdraw" g_objectNumber
    (c"objfaced", INT, 0x623620), // "objfaced" g_faceNumber
];
/// A vertex's: (label, kind, offset).
const VERTEX: [(&std::ffi::CStr, u32, u32); 8] = [
    (c"x", FLOAT, 0),    // "x"
    (c"y", FLOAT, 4),    // "yzuwv"
    (c"z", FLOAT, 8),    // "zuwv"
    (c"u", FLOAT, 0x18), // "uwv"
    (c"w", FLOAT, 0xC),  // "wv"
    (c"v", FLOAT, 0x1C), // "v"
    (c"co", HEX8, 0x10), // "co"
    (c"sp", HEX8, 0x14), // "sp"
];
const PT: &std::ffi::CStr = c"pt"; // "pt"

/// The Rust version: the dump of the `g_polygonVertexCount` vertices at `vertices`.
pub unsafe fn log_bad_polygon(vertices: u32) -> Result<(), BadInput> {
    let count: u32 = unsafe { game_read(COUNT_RVA) };
    if count > MOST {
        return Err(BadInput("more than 64 vertices"));
    }
    let text = |s: &std::ffi::CStr| s.as_ptr() as u32;
    let value = |a: usize| match a {
        SCREEN_WIDTH => crate::session::SCREEN.width.get(),
        SCREEN_HEIGHT => crate::session::SCREEN.height.get(),
        a => unsafe { game_read::<u32>(rva(a)) },
    };
    for (label, kind, at) in HEAD {
        log(WRITE, text(label));
        log(kind, value(at));
    }
    unsafe { game_write(MASK_RVA, 0u32) };
    let object = unsafe { game_read::<u32>(OBJECT_RVA) } == 1;
    let tail: &[(&std::ffi::CStr, u32, usize)] = if object { &OBJECT } else { &COURSE };
    for &(label, kind, at) in AFTER.iter().chain(tail) {
        log(WRITE, text(label));
        log(kind, value(at));
    }
    log(NEWLINE, 0);
    if count == 0 {
        return Ok(());
    }
    for n in 0..count {
        let vertex = vertices.wrapping_add(0x20 * n);
        log(WRITE, text(PT));
        log(INT, n);
        for (label, kind, offset) in VERTEX {
            log(WRITE, text(label));
            log(kind, unsafe {
                ((vertex + offset) as *const u32).read_unaligned()
            });
        }
        log(NEWLINE, 0);
    }
    log(NEWLINE, 0);
    Ok(())
}

/// One log call at level 1: a text, a number (a float with two decimals) or a line's end.
fn log(kind: u32, value: u32) {
    use crate::logfile::log_write::log_write;
    use crate::logfile::{log_float::log_float, log_hex8::log_hex8, log_int::log_int};
    unsafe {
        match kind {
            WRITE => log_write(value as *const u8, false, 1),
            HEX8 => log_hex8(value, false, 1),
            FLOAT => log_float(f32::from_bits(value), 2, false, 1),
            INT => log_int(value as i32, false, 1),
            INT_NL => log_int(value as i32, true, 1),
            _ => log_write(c"".as_ptr().cast(), true, 1),
        }
    }
}
