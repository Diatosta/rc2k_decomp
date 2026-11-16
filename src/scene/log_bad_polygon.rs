//! `scene::log_bad_polygon`: a polygon Direct3D was not given, dumped to the log at level 1:
//! "videox" the screen's width, "videoy" the screen's height (twice), "r" "l" "b" "t" the clip
//! edges (two decimals), "donezcut" `PolygonWork.depth_clipped`, "donesuth"
//! `PolygonWork.screen_clipped`, "cuttest" `PolygonWork.polygon_clip_mask` (with a line's end; then
//! the mask 0), "tpage" the texture page (hex8), "points" the polygon's vertex count; "polynum" the
//! current course face, or with the drawing-object flag 1 "objdraw" the current object and
//! "objfaced" the face number; a line's end. Then with vertices, each (0x20 bytes): "pt" its
//! number, "x" +0, "yzuwv" +4, "zuwv" +8, "uwv" +0x18, "wv" +0xC, "v" +0x1C (floats, two decimals;
//! the labels are the tails of one string), "co" +0x10 and "sp" +0x14 (hex8), a line's end; and a
//! last line's end. Refuses (before anything) more than 64 vertices.

use crate::errors::BadInput;
use crate::memory::cells::Store;

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
type Value = fn(&crate::race::view_scratch::ViewScratch, &crate::memory::session::Screen) -> Shown;
const MOST: u32 = 64;

/// (label, value): the head's lines before the mask's clearing, and after.
const HEAD: [(&std::ffi::CStr, Value); 10] = [
    (c"videox", |_, screen| Shown::Hex8(screen.width.get())), // "videox"
    (c"videoy", |_, screen| Shown::Hex8(screen.height.get())), // "videoy"
    (c"videoy", |_, screen| Shown::Hex8(screen.height.get())), // "videoy"
    (c"r", |view, _| {
        Shown::Float(view.polygon.window.clip_screen[3].get())
    }),
    (c"l", |view, _| {
        Shown::Float(view.polygon.window.clip_screen[2].get())
    }),
    (c"b", |view, _| {
        Shown::Float(view.polygon.window.clip_screen[1].get())
    }),
    (c"t", |view, _| {
        Shown::Float(view.polygon.window.clip_screen[0].get())
    }),
    (c"donezcut", |view, _| {
        Shown::Int(view.polygon.depth_clipped.get() as i32)
    }),
    (c"donesuth", |view, _| {
        Shown::Int(view.polygon.screen_clipped.get() as i32)
    }),
    (c"cuttest", |view, _| {
        Shown::IntNewline(view.polygon.polygon_clip_mask.get() as i32)
    }),
];
const AFTER: [(&std::ffi::CStr, Value); 2] = [
    (c"tpage", |view, _| {
        Shown::Hex8(view.polygon.texture_page.get())
    }),
    (c"points", |view, _| {
        Shown::Int(view.polygon.polygon_vertex_count.get() as i32)
    }),
];
const POLYGON_NUMBER: [(&std::ffi::CStr, Value); 1] = [(c"polynum", |view, _| {
    Shown::Int(view.face.course_face.get() as i32)
})];
const OBJECT: [(&std::ffi::CStr, Value); 2] = [
    // The view's whose polygon this is (every polygon work is a view slot's).
    (c"objdraw", |view, _| {
        Shown::Int(view.cars.object_number.get() as i32)
    }),
    (c"objfaced", |view, _| {
        Shown::Int(view.face.face_number.get() as i32)
    }),
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
const PT: &std::ffi::CStr = c"pt";

/// The dump of the `polygon_vertex_count` vertices at `vertices` (at most 64, held to the
/// vertex buffer: the longest read is the last vertex's 0x20 bytes of its 0x800).
pub fn log_bad_polygon(
    view: &crate::race::view_scratch::ViewScratch,
    screen: &crate::memory::session::Screen,
    vertices: &dyn Store,
) -> Result<(), BadInput> {
    let count: u32 = view.polygon.polygon_vertex_count.get();
    if count > MOST {
        return Err(BadInput("more than 64 vertices"));
    }
    for (label, at) in HEAD {
        write(label);
        log(at(view, screen));
    }
    view.polygon.polygon_clip_mask.set(0u32);
    let object = view.object.drawing_object.get() == 1;
    let tail: &[(&std::ffi::CStr, Value)] = if object { &OBJECT } else { &POLYGON_NUMBER };
    for &(label, at) in AFTER.iter().chain(tail) {
        write(label);
        log(at(view, screen));
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
