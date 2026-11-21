//! `effects::clip_quad`: a quad (four vertices: x, y, the depth, `rhw`, colour and specular kept,
//! then the texture u and v) clipped against one screen edge chosen by the settings'
//! `marks_layout`: 4 x at least the screen's middle x, 3 x at most its middle x less one, 2 y at
//! least the middle y, 1 y at most the middle y less one (each test a signed compare of the floats'
//! bits). All in: returns false, nothing changed. All out: returns true. Else the run of inside
//! vertices (from the first after an outside one to the last) and the two crossings (the far
//! edge's, then the near edge's: the edge's coordinate the line, the other and the texture
//! interpolated on the x87, the depth, `rhw`, colour and specular the outside vertex's) are built
//! in the clip's work vertices and copied over the quad; the polygon's vertex count and
//! `GlowState.clip_count` are their number; returns false. Other settings: returns false, nothing
//! changed.
//!
//! Refuses null settings (a null quad cannot come: the quad is the vertices' own).

use crate::d3d::vertices_store::Vertex;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::memory::session::{CellKind, DwordLike, FloatLike};

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

    fn set_corner(self, v: &mut Corner, value: f32) {
        match self {
            Axis::X => v.x = value,
            Axis::Y => v.y = value,
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

/// Sets whether the effects drawn from now on are cut to the front end window's rectangle.
pub fn cut_to_part(effects: &crate::effects::owner::Effects, on: bool) {
    effects
        .clipping
        .cut_to_part
        .store(on, std::sync::atomic::Ordering::Relaxed);
}

/// One vertex's drawn fields, held while the polygon is cut.
#[derive(Clone, Copy)]
struct Corner {
    x: f32,
    y: f32,
    z: f32,
    rhw: f32,
    u: f32,
    v: f32,
    colour: u32,
    specular: u32,
}

impl Corner {
    /// The vertex's drawn fields.
    fn of<K: CellKind>(v: &Vertex<K>) -> Corner {
        Corner {
            x: v.x.get(),
            y: v.y.get(),
            z: v.z.get(),
            rhw: v.rhw.get(),
            u: v.u.get(),
            v: v.v.get(),
            colour: v.colour.get(),
            specular: v.specular.get(),
        }
    }

    /// Writes the drawn fields into the vertex.
    fn put<K: CellKind>(self, v: &Vertex<K>) {
        v.x.set(self.x);
        v.y.set(self.y);
        v.z.set(self.z);
        v.rhw.set(self.rhw);
        v.u.set(self.u);
        v.v.set(self.v);
        v.colour.set(self.colour);
        v.specular.set(self.specular);
    }

    fn lerp(self, to: Corner, t: f32) -> Corner {
        let mix = |a: f32, b: f32| a + (b - a) * t;
        Corner {
            x: mix(self.x, to.x),
            y: mix(self.y, to.y),
            z: mix(self.z, to.z),
            rhw: mix(self.rhw, to.rhw),
            u: mix(self.u, to.u),
            v: mix(self.v, to.v),
            colour: self.colour,
            specular: self.specular,
        }
    }
}

/// The polygon cut to the rectangle `[left, top, right, bottom]`, one side after the other.
fn cut_to_rectangle(polygon: Vec<Corner>, rect: [f32; 4]) -> Vec<Corner> {
    // (the coordinate cut on, the line, whether the inside is at or above the line)
    type Side = (fn(&Corner) -> f32, f32, bool);
    let sides: [Side; 4] = [
        (|c| c.x, rect[0], true),
        (|c| c.x, rect[2], false),
        (|c| c.y, rect[1], true),
        (|c| c.y, rect[3], false),
    ];
    let mut polygon = polygon;
    for (coord, line, above) in sides {
        let inside = |c: &Corner| {
            if above {
                coord(c) >= line
            } else {
                coord(c) <= line
            }
        };
        let mut next = Vec::with_capacity(polygon.len() + 2);
        for (k, &from) in polygon.iter().enumerate() {
            let to = polygon[(k + 1) % polygon.len()];
            let (a, b) = (inside(&from), inside(&to));
            if a {
                next.push(from);
            }
            if a != b {
                let t = (line - coord(&from)) / (coord(&to) - coord(&from));
                next.push(from.lerp(to, t));
            }
        }
        polygon = next;
        if polygon.is_empty() {
            break;
        }
    }
    polygon
}

/// A part of a larger split screen: the quad cut to the part's rectangle (the front end's window
/// holds it). Whether all of the quad is outside.
fn clip_to_part(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> bool {
    let window = &scene.scratch.front_end_polygon().window;
    let (x, y) = (
        window.view_offset[0].get() as f32,
        window.view_offset[1].get() as f32,
    );
    let rect = [
        x,
        y,
        x + window.view_size[0].get() as f32,
        y + window.view_size[1].get() as f32,
    ];
    let polygon: Vec<Corner> = buffer.vertices[..4]
        .iter()
        .map(|v| Corner {
            x: v.x.get(),
            y: v.y.get(),
            z: v.z.get(),
            rhw: v.rhw.get(),
            u: v.u.get(),
            v: v.v.get(),
            colour: v.colour.get(),
            specular: v.specular.get(),
        })
        .collect();
    let cut = cut_to_rectangle(polygon, rect);
    if cut.len() < 3 {
        return true;
    }
    for (c, v) in cut.iter().zip(buffer.vertices.iter()) {
        v.x.set(c.x);
        v.y.set(c.y);
        v.z.set(c.z);
        v.rhw.set(c.rhw);
        v.u.set(c.u);
        v.v.set(c.v);
        v.colour.set(c.colour);
        v.specular.set(c.specular);
    }
    scene
        .scratch
        .front_end_polygon()
        .polygon_vertex_count
        .set(cut.len() as u32);
    effects.clipping.glow.borrow_mut().clip_count = cut.len() as u32;
    false
}

/// Whether all the quad is outside. The quad is the polygon vertices' first four; vertex
/// numbers wrap at 4.
pub fn clip_quad(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    scene: &crate::scene::owner::Scene,
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
        if effects
            .clipping
            .cut_to_part
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Ok(clip_to_part(buffer, scene, effects));
        }
        if !(1..=4).contains(&mode) {
            return Ok(false);
        }
        let (edge, coord, other, below) = EDGES[mode as usize - 1];
        let line = effects.clipping.clip.borrow().edges[edge];
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
        let mut work = [Corner::of(&q[0]); 6];
        let mut built = 0;
        let mut k = edi;
        loop {
            work[built] = Corner::of(&q[k]);
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
            let mut w = Corner::of(o);
            coord.set_corner(&mut w, line);
            other.set_corner(&mut w, v[0]);
            w.u = v[1];
            w.v = v[2];
            work[built] = w;
            built += 1;
        }
        for (from, to) in work.iter().zip(q.iter()).take(built) {
            from.put(to);
        }
        let count = built as u32;
        scene
            .scratch
            .front_end_polygon()
            .polygon_vertex_count
            .set(count);
        effects.clipping.glow.borrow_mut().clip_count = count;
    }
    Ok(false)
}

/// `scene::draw_fan` of the polygon vertices.
pub fn draw_fan(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    scene: &crate::scene::owner::Scene,
) {
    crate::scene::draw_fan::draw_fan(
        scene.scratch.game_scratch(),
        scene,
        scene.scratch.front_end_polygon(),
        buffer,
    );
}
