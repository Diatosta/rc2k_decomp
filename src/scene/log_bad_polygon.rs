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
//! Takes: the vertices
//!
//! Caller: `scene::show_bad_polygon` (0x443AEC). Refuses (before anything) more than 64
//! vertices (the original's loop would run through memory).

use crate::errors::BadInput;
use crate::pointers::Store;
use crate::scene::owner::scene;

const HEX8: u32 = 1;
/// The value column: what the line shows.
type Value = fn(&crate::scene::polygon_work::PolygonWork) -> u32;
const FLOAT: u32 = 2;
const INT: u32 = 3;
const INT_NL: u32 = 4;
const NEWLINE: u32 = 5;
const MOST: u32 = 64;

/// (label, kind, value): the head's lines before the mask's clearing, and after.
const HEAD: [(&std::ffi::CStr, u32, Value); 10] = [
    (c"videox", HEX8, |_| {
        crate::system::owner::system().session.screen.width.get()
    }), // "videox" g_screenWidth
    (c"videoy", HEX8, |_| {
        crate::system::owner::system().session.screen.height.get()
    }), // "videoy" g_screenHeight
    (c"videoy", HEX8, |_| {
        crate::system::owner::system().session.screen.height.get()
    }), // "videoy" g_screenHeight
    (c"r", FLOAT, |_| scene().drawing.scene.clip_screen[3].bits()), // "r" g_clipScreenRight
    (c"l", FLOAT, |_| scene().drawing.scene.clip_screen[2].bits()), // "l" g_clipScreenLeft
    (c"b", FLOAT, |_| scene().drawing.scene.clip_screen[1].bits()), // "b" g_clipScreenBottom
    (c"t", FLOAT, |_| scene().drawing.scene.clip_screen[0].bits()), // "t" g_clipScreenTop
    (c"donezcut", INT, |poly| poly.depth_clipped.bits()),           // "donezcut"
    (c"donesuth", INT, |poly| poly.screen_clipped.bits()),          // "donesuth"
    (c"cuttest", INT_NL, |poly| poly.polygon_clip_mask.bits()),     // "cuttest" g_polygonClipMask
];
const AFTER: [(&std::ffi::CStr, u32, Value); 2] = [
    (c"tpage", HEX8, |poly| poly.texture_page.bits()), // "tpage"
    (c"points", INT, |poly| poly.polygon_vertex_count.bits()), // "points" g_polygonVertexCount
];
const POLYGON_NUMBER: [(&std::ffi::CStr, u32, Value); 1] = [(c"polynum", INT, |poly| {
    crate::race::view_scratch::of_polygon(poly).map_or(0, |v| v.face.course_face.bits())
})]; // "polynum"
const OBJECT: [(&std::ffi::CStr, u32, Value); 2] = [
    // The view's whose polygon this is (every polygon work is a view slot's).
    (c"objdraw", INT, |poly| {
        crate::race::view_scratch::of_polygon(poly).map_or(0, |v| v.cars.object_number.bits())
    }), // "objdraw" g_objectNumber
    (c"objfaced", INT, |poly| {
        crate::race::view_scratch::of_polygon(poly).map_or(0, |v| v.face.face_number.bits())
    }), // "objfaced" g_faceNumber
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

/// The Rust version: the dump of the `g_polygonVertexCount` vertices at `vertices` (at most 64,
/// held to the vertex buffer: the longest read is the last vertex's 0x20 bytes of its 0x800).
pub fn log_bad_polygon(
    poly: &crate::scene::polygon_work::PolygonWork,
    vertices: &dyn Store,
) -> Result<(), BadInput> {
    let count: u32 = poly.polygon_vertex_count.bits();
    if count > MOST {
        return Err(BadInput("more than 64 vertices"));
    }
    for (label, kind, at) in HEAD {
        write(label);
        log(kind, at(poly));
    }
    poly.polygon_clip_mask.set_bits(0u32);
    let object = crate::race::view_scratch::of_polygon(poly)
        .is_some_and(|v| v.object.drawing_object.bits() == 1);
    let tail: &[(&std::ffi::CStr, u32, Value)] = if object { &OBJECT } else { &POLYGON_NUMBER };
    for &(label, kind, at) in AFTER.iter().chain(tail) {
        write(label);
        log(kind, at(poly));
    }
    log(NEWLINE, 0);
    if count == 0 {
        return Ok(());
    }
    for n in 0..count {
        let mut words = [0u8; 0x20];
        vertices.load(0x20 * n as usize, &mut words)?;
        write(PT);
        log(INT, n);
        for (label, kind, offset) in VERTEX {
            write(label);
            log(
                kind,
                u32::from_le_bytes(
                    words[offset as usize..offset as usize + 4]
                        .try_into()
                        .unwrap(),
                ),
            );
        }
        log(NEWLINE, 0);
    }
    log(NEWLINE, 0);
    Ok(())
}

/// A label, at level 1.
fn write(label: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(label.to_bytes(), false, 1);
}

/// One log call at level 1: a number (a float with two decimals) or a line's end.
fn log(kind: u32, value: u32) {
    use crate::logfile::{log_float::log_float, log_hex8::log_hex8, log_int::log_int};
    match kind {
        HEX8 => log_hex8(value, false, 1),
        FLOAT => log_float(f32::from_bits(value), 2, false, 1),
        INT => log_int(value as i32, false, 1),
        INT_NL => log_int(value as i32, true, 1),
        _ => crate::logfile::log_write::log_nul(&c"", true, 1),
    }
}
