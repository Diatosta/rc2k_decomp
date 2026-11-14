//! `effects::clip_quad`: a quad (four vertices of 0x20 bytes: +0 x, +4 y, +8 .. +0x14 kept,
//! +0x18 / +0x1C texture) clipped against one screen edge chosen by the settings' +8: 4 x
//! at least the screen's middle x, 3 x at most its middle x less one, 2 y at least the
//! middle y, 1 y at most the middle y less one (each test a signed compare of the floats'
//! bits). All in: returns false, nothing changed. All out: returns true. Else the run of
//! inside vertices (from the first after an outside one to the last) and the two crossings
//! (the far edge's, then the near edge's: the edge's coordinate the line, the other and the
//! texture interpolated on the x87, +8 .. +0x14 the outside vertex's) are built in the
//! clip's work vertices and copied over the quad; the polygon's vertex count and
//! `GlowState.clip_count` are their number; returns false. Other settings: returns false,
//! nothing changed.
//!
//! Refuses null settings (a null quad cannot come: the quad is the vertices' own).

use crate::d3d::vertices_store::Vertex;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::session::{CellKind, DwordLike, FloatLike};

/// A vertex's coordinate the edge is tested on.
#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
}

impl Axis {
    fn get<K: CellKind>(self, v: &Vertex<K>) -> f32 {
        match self {
            Axis::X => v.x.get(),
            Axis::Y => v.y.get(),
        }
    }

    fn set<K: CellKind>(self, v: &Vertex<K>, value: f32) {
        match self {
            Axis::X => v.x.set(value),
            Axis::Y => v.y.set(value),
        }
    }
}

/// Per mode 1 .. 4: the line's variable, its coordinate, the other coordinate, whether
/// inside is at or below it.
const EDGES: [(usize, Axis, Axis, bool); 4] = [
    (3, Axis::Y, Axis::X, true),  // 1: the middle y less one
    (2, Axis::Y, Axis::X, false), // 2: the middle y
    (1, Axis::X, Axis::Y, true),  // 3: the middle x less one
    (0, Axis::X, Axis::Y, false), // 4: the middle x
];

/// Copies a whole vertex, field by field.
fn copy_vertex<A: CellKind, B: CellKind>(from: &Vertex<A>, to: &Vertex<B>) {
    to.x.set(from.x.get());
    to.y.set(from.y.get());
    to.z.set(from.z.get());
    to.rhw.set(from.rhw.get());
    to.colour.set(from.colour.get());
    to.specular.set(from.specular.get());
    to.u.set(from.u.get());
    to.v.set(from.v.get());
}

/// Whether all the quad is outside. The quad is the polygon vertices' first four; vertex
/// numbers wrap at 4.
pub fn clip_quad(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    effects: &crate::effects::owner::Effects,
) -> Result<bool, Refused> {
    {
        let settings: u32 = effects.setup.settings.is_set.get();
        if settings == 0 {
            return Err(BadInput("null marks settings").into());
        }
        let q = &buffer.vertices;
        let marks = effects.settings()?;
        let mode = marks.marks_layout.get();
        if !(1..=4).contains(&mode) {
            return Ok(false);
        }
        let (edge, coord, other, below) = EDGES[mode as usize - 1];
        let line = effects.clipping.clip.edges[edge].get();
        // The test of vertex k: inside (the floats compared by their bits).
        let t = |k: usize| -> bool {
            let v = coord.get(&q[k]);
            if below {
                x87::bits_at_or_below(v, line)
            } else {
                x87::bits_at_or_below(line, v)
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
                [other.get(s), other.get(o)],
                [s.u.get(), o.u.get()],
                [s.v.get(), o.v.get()],
            ];
            let v = x87::clip_lerp3_floats(line, coord.get(o), coord.get(s), pairs);
            let w = &work[built];
            coord.set(w, line);
            other.set(w, v[0]);
            w.u.set(v[1]);
            w.v.set(v[2]);
            w.z.set(o.z.get());
            w.rhw.set(o.rhw.get());
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
            .set(count);
        effects.clipping.glow.clip_count.set(count);
    }
    Ok(false)
}

/// `scene::draw_fan` of the polygon vertices.
pub fn draw_fan(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    scene: &crate::scene::owner::Scene,
) {
    crate::scene::draw_fan::draw_fan(
        crate::workers::game_scratch(),
        scene,
        crate::scene::polygon_work::front_end(),
        buffer,
    );
}
