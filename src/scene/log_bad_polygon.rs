//! `scene::log_bad_polygon` (0x443AF8): a polygon Direct3D was not given, dumped to the
//! log at level 1: "videox" `g_screenWidth`, "videoy" `g_screenHeight` (twice), "r" "l" "b"
//! "t" the clip edges (`g_clipScreenRight` .. `g_clipScreenTop`, two decimals), "donezcut"
//! `SceneDraw.depth_clipped`, "donesuth" `SceneDraw.screen_clipped`, "cuttest" `g_polygonClipMask` (with a line's
//! end; then the mask 0), "tpage" the texture page (hex8), "points" the polygon's vertex count;
//! "polynum" the current course face, or with the drawing-object flag 1 "objdraw" the
//! current object and "objfaced" the face number; a line's end. Then with vertices, each
//! (0x20 bytes): "pt" its number, "x" +0, "yzuwv" +4, "zuwv" +8, "uwv" +0x18,
//! "wv" +0xC, "v" +0x1C (floats, two decimals; the labels are the tails of one string),
//! "co" +0x10 and "sp" +0x14 (hex8), a line's end; and a last line's end.
//!
//! Takes: the vertices
//!
//! Caller: `scene::show_bad_polygon` (0x443AEC). Refuses (before anything) more than 64
//! vertices (the original's loop would run through memory).

use crate::cells::Store;
use crate::errors::BadInput;

/// A value as the dump shows it.
#[derive(Clone, Copy)]
enum Shown {
    /// Eight hex digits.
    Hex8(u32),
    /// A float with two decimals.
    Float(f32),
    /// A whole number.
    Int(i32),
    /// A whole number, then a line's end.
    IntNewline(i32),
    /// A line's end.
    Newline,
}

/// The value column: what the line shows.
type Value = fn(&crate::scene::polygon_work::PolygonWork, &crate::session::Screen) -> Shown;
const MOST: u32 = 64;

/// (label, value): the head's lines before the mask's clearing, and after.
const HEAD: [(&std::ffi::CStr, Value); 10] = [
    (c"videox", |_, screen| Shown::Hex8(screen.width.get())), // "videox" g_screenWidth
    (c"videoy", |_, screen| Shown::Hex8(screen.height.get())), // "videoy" g_screenHeight
    (c"videoy", |_, screen| Shown::Hex8(screen.height.get())), // "videoy" g_screenHeight
    (c"r", |poly, _| {
        Shown::Float(poly.window.clip_screen[3].get())
    }), // "r" g_clipScreenRight
    (c"l", |poly, _| {
        Shown::Float(poly.window.clip_screen[2].get())
    }), // "l" g_clipScreenLeft
    (c"b", |poly, _| {
        Shown::Float(poly.window.clip_screen[1].get())
    }), // "b" g_clipScreenBottom
    (c"t", |poly, _| {
        Shown::Float(poly.window.clip_screen[0].get())
    }), // "t" g_clipScreenTop
    (c"donezcut", |poly, _| {
        Shown::Int(poly.depth_clipped.get() as i32)
    }), // "donezcut"
    (c"donesuth", |poly, _| {
        Shown::Int(poly.screen_clipped.get() as i32)
    }), // "donesuth"
    (c"cuttest", |poly, _| {
        Shown::IntNewline(poly.polygon_clip_mask.get() as i32)
    }), // "cuttest" g_polygonClipMask
];
const AFTER: [(&std::ffi::CStr, Value); 2] = [
    (c"tpage", |poly, _| Shown::Hex8(poly.texture_page.get())), // "tpage"
    (c"points", |poly, _| {
        Shown::Int(poly.polygon_vertex_count.get() as i32)
    }), // "points" g_polygonVertexCount
];
const POLYGON_NUMBER: [(&std::ffi::CStr, Value); 1] = [(c"polynum", |poly, _| {
    Shown::Int(
        crate::race::view_scratch::of_polygon(poly).map_or(0, |v| v.face.course_face.get()) as i32,
    )
})]; // "polynum"
const OBJECT: [(&std::ffi::CStr, Value); 2] = [
    // The view's whose polygon this is (every polygon work is a view slot's).
    (c"objdraw", |poly, _| {
        Shown::Int(
            crate::race::view_scratch::of_polygon(poly).map_or(0, |v| v.cars.object_number.get())
                as i32,
        )
    }), // "objdraw" g_objectNumber
    (c"objfaced", |poly, _| {
        Shown::Int(
            crate::race::view_scratch::of_polygon(poly).map_or(0, |v| v.face.face_number.get())
                as i32,
        )
    }), // "objfaced" g_faceNumber
];
/// How a vertex's dword is shown.
#[derive(Clone, Copy)]
enum Part {
    Float,
    Hex8,
}

/// A vertex's: (label, how shown, offset).
const VERTEX: [(&std::ffi::CStr, Part, u32); 8] = [
    (c"x", Part::Float, 0),    // "x"
    (c"y", Part::Float, 4),    // "yzuwv"
    (c"z", Part::Float, 8),    // "zuwv"
    (c"u", Part::Float, 0x18), // "uwv"
    (c"w", Part::Float, 0xC),  // "wv"
    (c"v", Part::Float, 0x1C), // "v"
    (c"co", Part::Hex8, 0x10), // "co"
    (c"sp", Part::Hex8, 0x14), // "sp"
];
const PT: &std::ffi::CStr = c"pt"; // "pt"

/// The Rust version: the dump of the `g_polygonVertexCount` vertices at `vertices` (at most 64,
/// held to the vertex buffer: the longest read is the last vertex's 0x20 bytes of its 0x800).
pub fn log_bad_polygon(
    poly: &crate::scene::polygon_work::PolygonWork,
    screen: &crate::session::Screen,
    vertices: &dyn Store,
) -> Result<(), BadInput> {
    let count: u32 = poly.polygon_vertex_count.get();
    if count > MOST {
        return Err(BadInput("more than 64 vertices"));
    }
    for (label, at) in HEAD {
        write(label);
        log(at(poly, screen));
    }
    poly.polygon_clip_mask.set(0u32);
    let object = crate::race::view_scratch::of_polygon(poly)
        .is_some_and(|v| v.object.drawing_object.get() == 1);
    let tail: &[(&std::ffi::CStr, Value)] = if object { &OBJECT } else { &POLYGON_NUMBER };
    for &(label, at) in AFTER.iter().chain(tail) {
        write(label);
        log(at(poly, screen));
    }
    log(Shown::Newline);
    if count == 0 {
        return Ok(());
    }
    for n in 0..count {
        let mut words = [0u8; 0x20];
        vertices.load(0x20 * n as usize, &mut words)?;
        write(PT);
        log(Shown::Int(n as i32));
        for (label, part, offset) in VERTEX {
            write(label);
            let dword = u32::from_le_bytes(
                words[offset as usize..offset as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            log(match part {
                Part::Float => Shown::Float(f32::from_bits(dword)),
                Part::Hex8 => Shown::Hex8(dword),
            });
        }
        log(Shown::Newline);
    }
    log(Shown::Newline);
    Ok(())
}

/// A label, at level 1.
fn write(label: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(label.to_bytes(), false, 1);
}

/// One log call at level 1: a number (a float with two decimals) or a line's end.
fn log(value: Shown) {
    use crate::logfile::{log_float::log_float, log_hex8::log_hex8, log_int::log_int};
    match value {
        Shown::Hex8(v) => log_hex8(v, false, 1),
        Shown::Float(v) => log_float(v, 2, false, 1),
        Shown::Int(v) => log_int(v, false, 1),
        Shown::IntNewline(v) => log_int(v, true, 1),
        Shown::Newline => crate::logfile::log_write::log_nul(&c"", true, 1),
    }
}
