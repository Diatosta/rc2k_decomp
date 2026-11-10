//! `effects::clip_quad` (0x4B8A0B): a quad (four vertices of 0x20 bytes at ebp: +0 x, +4 y,
//! +8 .. +0x14 kept, +0x18 / +0x1C texture) clipped against one screen edge by the
//! settings' +8: 4 x at least `g_marksScreenMiddleX`, 3 x at most `g_marksScreenMiddleXLess`,
//! 2 y at least `g_marksScreenMiddleY`, 1 y at most `g_marksScreenMiddleYLess` (each test a
//! signed compare of the floats' bits, as the original's `sub; jl`). All in: CF clear,
//! nothing changed. All out: CF set. Else the run of inside vertices (from the first after
//! an outside one to the last) and the two crossings (the far edge's, then the near
//! edge's: the edge's coordinate the line, the other and the texture interpolated on the
//! x87, +8 .. +0x14 the outside vertex's) are built in `g_marksClipVertices` (`TLVertex`s) and copied over
//! the quad; `g_polygonVertexCount` and `dword_93CC94` their number; CF clear. Other
//! settings: CF clear, nothing changed.
//!
//! Eight callers, each `jb` after it. Refuses null settings (a null quad, refused before,
//! cannot come: the quad is the vertices' own).

use crate::d3d::vertices_store::Vertex;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::session::Float;
/// A vertex's coordinate the edge is tested on.
type Axis = fn(&Vertex) -> &Float;

fn x_of(v: &Vertex) -> &Float {
    &v.x
}

fn y_of(v: &Vertex) -> &Float {
    &v.y
}

/// Per mode 1 .. 4: the line's variable, its coordinate, the other coordinate, whether
/// inside is at or below it.
const EDGES: [(usize, Axis, Axis, bool); 4] = [
    (3, y_of, x_of, true),  // 1: g_marksScreenMiddleYLess
    (2, y_of, x_of, false), // 2: g_marksScreenMiddleY
    (1, x_of, y_of, true),  // 3: g_marksScreenMiddleXLess
    (0, x_of, y_of, false), // 4: g_marksScreenMiddleX
];

/// Copies a whole vertex, field by field.
fn copy_vertex(from: &Vertex, to: &Vertex) {
    to.x.set_bits(from.x.bits());
    to.y.set_bits(from.y.bits());
    to.z.set_bits(from.z.bits());
    to.rhw.set_bits(from.rhw.bits());
    to.colour.set(from.colour.get());
    to.specular.set(from.specular.get());
    to.u.set_bits(from.u.bits());
    to.v.set_bits(from.v.bits());
}

/// Whether all the quad is outside (the original's CF).
/// The quad is the polygon vertices' first four (`g_polygonVertices`, every caller's);
/// the original stepped its vertex offsets by 0x20 and masked them with 0x7F, here the
/// vertex numbers wrap at 4.
pub fn clip_quad(effects: &crate::effects::owner::Effects) -> Result<bool, Refused> {
    {
        let settings: u32 = effects.setup.settings.is_set.get();
        if settings == 0 {
            return Err(BadInput("null marks settings").into());
        }
        let q = &crate::d3d::vertices_store::VERTICES.vertices;
        let marks = crate::effects::setup::effects_settings()?;
        let mode = marks.marks_layout.get();
        if !(1..=4).contains(&mode) {
            return Ok(false);
        }
        let (edge, coord, other, below) = EDGES[mode as usize - 1];
        let line: u32 = effects.clipping.clip.edges[edge].bits();
        // The test of vertex k: inside.
        let t = |k: usize| -> bool {
            let v = coord(&q[k]).bits() as i32;
            if below {
                line as i32 >= v
            } else {
                v >= line as i32
            }
        };
        let (esi, edi);
        if !t(0) {
            let Some(first) = (1..=3).find(|&k| t(k)) else {
                return Ok(true);
            };
            edi = first;
            esi = if t(3) {
                3
            } else if t(2) {
                2
            } else {
                1
            };
        } else if !t(1) {
            esi = 0;
            edi = if t(2) {
                2
            } else if t(3) {
                3
            } else {
                0
            };
        } else if !t(2) {
            esi = 1;
            edi = if t(3) { 3 } else { 0 };
        } else if !t(3) {
            esi = 2;
            edi = 0;
        } else {
            return Ok(false);
        }
        let work = &effects.clipping.clip.vertices;
        let mut built = 0;
        let mut k = edi;
        loop {
            copy_vertex(&q[k], &work[built]);
            built += 1;
            if k == esi {
                break;
            }
            k = (k + 1) & 3;
        }
        let c = (esi + 1) & 3;
        let d = edi.wrapping_sub(1) & 3;
        for (inside, outside) in [(esi, c), (edi, d)] {
            let (s, o) = (&q[inside], &q[outside]);
            let pairs = [
                [other(s).bits(), other(o).bits()],
                [s.u.bits(), o.u.bits()],
                [s.v.bits(), o.v.bits()],
            ];
            let v = x87::clip_lerp3(line, coord(o).bits(), coord(s).bits(), pairs);
            let w = &work[built];
            coord(w).set_bits(line);
            other(w).set_bits(v[0]);
            w.u.set_bits(v[1]);
            w.v.set_bits(v[2]);
            w.z.set_bits(o.z.bits());
            w.rhw.set_bits(o.rhw.bits());
            w.colour.set(o.colour.get());
            w.specular.set(o.specular.get());
            built += 1;
        }
        for (from, to) in work.iter().zip(q.iter()).take(built) {
            copy_vertex(from, to);
        }
        let count = built as u32;
        crate::scene::polygon_work::front_end()
            .polygon_vertex_count
            .set_bits(count);
        effects.clipping.glow.clip_count.set(count);
    }
    Ok(false)
}

/// `scene::draw_fan` (0x44EB32) of `g_polygonVertices` (plain since the scene's
/// conversion, 2026-10-23).
pub fn draw_fan() {
    crate::scene::draw_fan::draw_fan(crate::scene::polygon_work::front_end());
}
