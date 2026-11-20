use super::*;

/// A fan that can take a second pass.
pub(super) struct LastFan {
    /// Its triangles' first vertex and count: the last batch's tail.
    first_vertex: u32,
    vertices: u32,
    /// Its corners' places (x, y, z, 1/w).
    places: smallvec::SmallVec<[[u32; 4]; 6]>,
    batch: Batch,
    area: Area,
}

/// Where a batch can touch the target, in the game's coordinates: a rectangle around its
/// triangles, both ends in. It is kept as the continuous extent, not rounded to whole game
/// pixels: a target finer than the game's grid (a window larger than the game's screen) has
/// pixel centres between the game's whole places, so two triangles with no whole place in
/// common can still share target pixels.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct Area {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Area {
    const EVERYWHERE: Area = Area {
        left: f32::NEG_INFINITY,
        top: f32::NEG_INFINITY,
        right: f32::INFINITY,
        bottom: f32::INFINITY,
    };

    /// Around `vertices`: a triangle touches the target's pixels whose centres it holds, so
    /// nothing outside its least and most x and y (the margin on each side covers the GPU's
    /// rounding of the corners to its sub-pixel grid, a sixteenth of a pixel at the coarsest
    /// Vulkan allows, in pixels of a target smaller than the game's screen too).
    fn around(vertices: &[Vertex]) -> Area {
        const MARGIN: f32 = 0.25;
        let (mut left, mut top) = (f32::INFINITY, f32::INFINITY);
        let (mut right, mut bottom) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        for v in vertices {
            let (x, y) = (f32::from_bits(v[0]), f32::from_bits(v[1]));
            if !x.is_finite() || !y.is_finite() {
                return Area::EVERYWHERE;
            }
            (left, right) = (left.min(x), right.max(x));
            (top, bottom) = (top.min(y), bottom.max(y));
        }
        Area {
            left: left - MARGIN,
            top: top - MARGIN,
            right: right + MARGIN,
            bottom: bottom + MARGIN,
        }
    }

    fn with(self, other: Area) -> Area {
        Area {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    fn meets(self, other: Area) -> bool {
        self.left <= other.right
            && other.left <= self.right
            && self.top <= other.bottom
            && other.top <= self.bottom
    }
}

/// A triangle's corners on the target as the GPU places them (`scene.vert`: x and y as given,
/// times the clip space's w, which the GPU divides out again; w is 1 over the fourth dword when
/// that is above 0, else 1), when that is a plain place: x, y and w finite, w small enough that
/// the clip space's place is finite too.
pub(super) fn corners(t: &[Vertex; 3]) -> Option<[(f64, f64); 3]> {
    let mut out = [(0.0, 0.0); 3];
    for (corner, v) in out.iter_mut().zip(t) {
        let (x, y, w) = (
            f32::from_bits(v[0]),
            f32::from_bits(v[1]),
            f32::from_bits(v[3]),
        );
        if !x.is_finite() || !y.is_finite() || !w.is_finite() || (w > 0.0 && w < 1e-30) {
            return None;
        }
        *corner = (f64::from(x), f64::from(y));
    }
    Some(out)
}

/// Whether triangles `t` and `u` are drawn on no pixel in common, `size` the target's.
///
/// Either a line through one's edge has the other wholly on its far side, by more than the GPU's
/// rounding of the corners can bring it back: `Area::around`'s margin, widened for a corner far
/// past the edge's ends, which a tilt of the line moves the most. Or the two have an edge with
/// the same corners, bit for bit in all the GPU's place reads (x, y, w), inside the target (no
/// clipping makes new corners on it), and the other's third corner is on its far side by more
/// than the margin: the GPU places those corners alike, so the edge stays one line between them,
/// and the pixels on it go to one triangle only (the rasterising rule for a shared edge). A
/// triangle with no area, or a place that is no plain place (`corners`), may share pixels.
pub(super) fn apart(t: &[Vertex; 3], u: &[Vertex; 3], size: (f64, f64)) -> bool {
    const MARGIN: f64 = 0.125;
    let (Some(tp), Some(up)) = (corners(t), corners(u)) else {
        return false;
    };
    let same = |a: &Vertex, b: &Vertex| a[0] == b[0] && a[1] == b[1] && a[3] == b[3];
    // Inside the clip space (the shaders put x at x + 0.5 of the target's width): no clipping.
    let inside =
        |p: (f64, f64)| (0.0..=size.0 - 1.0).contains(&p.0) && (0.0..=size.1 - 1.0).contains(&p.1);
    // An edge of `own` (its corners `n` and `n + 1`), with its third corner, against `other`.
    let cuts = |own: &[(f64, f64); 3],
                own_v: &[Vertex; 3],
                other: &[(f64, f64); 3],
                other_v: &[Vertex; 3]| {
        (0..3).any(|n| {
            let (p, q, r) = (own[n], own[(n + 1) % 3], own[(n + 2) % 3]);
            let (dx, dy) = (q.0 - p.0, q.1 - p.1);
            let length = (dx * dx + dy * dy).sqrt();
            if length == 0.0 {
                return false;
            }
            // The distance from the edge's line, positive on the third corner's side.
            let side = |s: (f64, f64)| (dx * (s.1 - p.1) - dy * (s.0 - p.0)) / length;
            let own_side = side(r);
            if own_side.abs() <= MARGIN {
                return false;
            }
            let beyond = |s: (f64, f64)| {
                let distance = |a: (f64, f64)| ((s.0 - a.0).powi(2) + (s.1 - a.1).powi(2)).sqrt();
                let reach = distance(p).max(distance(q));
                side(s) * own_side.signum() < -MARGIN * (1.0 + 2.0 * reach / length)
            };
            let (pv, qv) = (&own_v[n], &own_v[(n + 1) % 3]);
            let shared = (0..3).find(|&m| {
                let (a, b) = (&other_v[m], &other_v[(m + 1) % 3]);
                (same(a, pv) && same(b, qv)) || (same(a, qv) && same(b, pv))
            });
            match shared {
                Some(m) if inside(p) && inside(q) => beyond(other[(m + 2) % 3]),
                _ => other.iter().all(|&s| beyond(s)),
            }
        })
    };
    cuts(&tp, t, &up, u) || cuts(&up, u, &tp, t)
}

/// Whether two batches are drawn alike, wherever their triangles lie.
pub(super) fn alike(a: &Batch, b: &Batch) -> bool {
    Batch {
        first_vertex: 0,
        vertices: 0,
        ..*a
    } == Batch {
        first_vertex: 0,
        vertices: 0,
        ..*b
    }
}

pub(super) const ALWAYS: u32 = 8;

// As the rasteriser reads them.
pub(super) fn linear_of(states: &[u32; STATES]) -> bool {
    states[0x11] == 2 || matches!(states[0x12], 2 | 5 | 6)
}

/// The minifying filter is the mipmapping one (the software rasteriser's `mip_chains` asks alike).
pub(super) fn mips_of(states: &[u32; STATES]) -> bool {
    states[0x12] == 6
}

pub(super) fn clamp_of(states: &[u32; STATES]) -> bool {
    states[0x03] == 3
}

/// Whether the pixel shader's alpha test (`passes`) keeps a pixel of alpha `value`.
pub(super) fn alpha_passes(values: &PixelValues, value: f32) -> bool {
    let reference = values.alpha_reference;
    match values.alpha_function {
        0 => true,
        1 => false,
        2 => value < reference,
        3 => value == reference,
        4 => value <= reference,
        5 => value > reference,
        6 => value != reference,
        7 => value >= reference,
        _ => true,
    }
}

/// What a fan's draw can leave out without changing a pixel (`fan_fast_path`).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct FastPath {
    /// The alpha test cannot discard a pixel of the fan.
    pub discard_free: bool,
    /// The blend leaves each pixel as the shader made it, so it can be switched off.
    pub blend_off: bool,
}

/// Whether the alpha test keeps every pixel of a fan: always when the test is off or its
/// comparison is not one the shader knows, and, when `alpha_full` (every pixel's alpha is 1,
/// up to the few millionths the interpolation may be off), for the comparisons that pass
/// there whatever the reference is up to 254/255.
pub(super) fn alpha_test_keeps_all(values: &PixelValues, alpha_full: bool) -> bool {
    match values.alpha_function {
        0 | 8.. => true,
        5..=7 => alpha_full && values.alpha_reference * 255.0 <= 254.5,
        _ => false,
    }
}

/// Which work a fan's draw can leave out, the picture the same. `texture_clear`: the fan has
/// no texture, or a texture with no pixel under a full alpha. With that and every corner's
/// colour at a full alpha, each pixel's alpha is 1 (the fog and the alpha test change none).
/// - Blend factors source alpha / inverse source alpha keep the source's colour where its alpha
///   is 1 (`1 * s + 0 * d`), and one / zero do for any alpha: the blend can be off.
/// - An alpha test that passes (or is off) cannot discard.
///
/// A fan that takes a second pass later (`join_second`) keeps what it has here: that pass is
/// blended in the shader and discards nothing.
pub fn fan_fast_path(states: &[u32; STATES], vertices: &[Vertex], texture_clear: bool) -> FastPath {
    let alpha_full = texture_clear && vertices.iter().all(|v| v[4] >> 24 == 0xFF);
    let values = values_of(states, None);
    FastPath {
        discard_free: alpha_test_keeps_all(&values, alpha_full),
        blend_off: match pipeline_of(states).blend {
            Some((2, 1)) => true,
            Some((5, 6)) => alpha_full,
            _ => false,
        },
    }
}

pub(super) fn pipeline_of(states: &[u32; STATES]) -> PipelineKey {
    PipelineKey {
        cull: match states[0x16] {
            0 => 3, // Direct3D's default
            cull => cull,
        },
        depth_function: if states[0x07] != 0 {
            states[0x17]
        } else {
            ALWAYS
        },
        depth_write: states[0x0E] != 0,
        blend: (states[0x1B] != 0).then_some((states[0x13], states[0x14])),
        colour: true,
    }
}

pub(super) fn values_of(states: &[u32; STATES], texture: Option<&TextureNote>) -> PixelValues {
    let fog = states[0x22];
    let unit = |shift: u32| (fog >> shift & 0xFF) as f32 / 255.0;
    let mut flags = 0;
    if let Some(t) = texture {
        flags |= TEXTURED;
        if t.masks[3] != 0 {
            flags |= TEXTURE_HAS_ALPHA;
        }
    }
    if states[0x1C] != 0 {
        flags |= FOG;
    }
    PixelValues {
        fog_colour: [unit(16), unit(8), unit(0), unit(24)],
        alpha_reference: (states[0x18] & 0xFF) as f32 / 255.0,
        alpha_function: if states[0x0F] != 0 {
            states[0x19] as i32
        } else {
            0
        },
        mixing: states[0x15] as i32,
        flags,
    }
}

impl Batches {
    /// `commands` in the order they came. With `opaque` (whether a texture's every pixel has a
    /// full alpha), a fan drawn again at once over the same places without depth writes and
    /// blended joins the fan before as its second pass (`Batch::second`): one draw for both,
    /// the same picture but for the blend's rounding done in the shader (a step at most).
    pub fn of(
        commands: &CommandList,
        width: u32,
        height: u32,
        opaque: Option<&dyn Fn(&TextureNote) -> bool>,
    ) -> Batches {
        Batches::of_in(Batches::default(), commands, width, height, opaque)
    }

    /// `of`, for a GPU that holds its textures as layers of a few texture arrays and reads
    /// which one each vertex's passes take from `vertex_textures`: batches that differ only in
    /// their textures are drawn as one when `same_array` says each pass's textures can be
    /// read in one draw. The order and the gathering's rule are `of`'s, so the picture is.
    /// With `fast_path`, each fan's batch is told what it can leave out (`fan_fast_path`).
    pub fn of_layered(
        commands: &CommandList,
        width: u32,
        height: u32,
        opaque: &dyn Fn(&TextureNote) -> bool,
        same_array: fn(&TextureNote, &TextureNote) -> bool,
        fast_path: bool,
    ) -> Batches {
        let start = Batches {
            layered: Some(same_array),
            fast_path,
            ..spare_batches()
        };
        Batches::of_in(start, commands, width, height, Some(opaque))
    }

    fn of_in(
        mut out: Batches,
        noted: &CommandList,
        width: u32,
        height: u32,
        opaque: Option<&dyn Fn(&TextureNote) -> bool>,
    ) -> Batches {
        out.size = (f64::from(width), f64::from(height));
        for command in &noted.commands {
            match command {
                Command::Fan {
                    vertices,
                    states,
                    texture,
                    ..
                } => {
                    let vertices = noted.fan_vertices(*vertices);
                    if vertices.len() < 3 {
                        continue;
                    }
                    let Some(opaque) = opaque else {
                        out.push_fan(vertices, states, texture.as_ref(), None);
                        continue;
                    };
                    if out.join_second(vertices, states, texture.as_ref()) {
                        continue;
                    }
                    out.push_fan(vertices, states, texture.as_ref(), Some(opaque));
                    out.seconds.resize(out.vertices.len(), [0; 4]);
                    out.note_first(vertices, texture.as_ref(), opaque);
                }
                Command::Clear(rect) => {
                    out.last_fan = None;
                    out.push_clear(*rect, width, height);
                    if opaque.is_some() {
                        out.seconds.resize(out.vertices.len(), [0; 4]);
                    }
                }
            }
        }
        out.gather();
        out
    }

    /// Whether two batches can be drawn as one: alike, or (layered) alike but for textures
    /// read from the same arrays.
    fn joinable(&self, a: &Batch, b: &Batch) -> bool {
        let Some(same_array) = self.layered else {
            return alike(a, b);
        };
        let fit = |x: Option<u32>, y: Option<u32>| match (x, y) {
            (None, None) => true,
            (Some(x), Some(y)) => {
                x == y || same_array(&self.textures[x as usize], &self.textures[y as usize])
            }
            _ => false,
        };
        let seconds_fit = match (&a.second, &b.second) {
            (Some(x), Some(y)) => fit(x.texture, y.texture),
            (None, None) => true,
            _ => false,
        };
        // Only whether each pass is textured stays in the comparison.
        let plain = |x: &Batch| Batch {
            first_vertex: 0,
            vertices: 0,
            texture: x.texture.map(|_| 0),
            second: x.second.map(|s| Second {
                texture: s.texture.map(|_| 0),
                ..s
            }),
            ..*x
        };
        fit(a.texture, b.texture) && seconds_fit && plain(a) == plain(b)
    }

    /// Each batch moved back into the last run drawn like it, when no batch between them can
    /// touch its pixels. Batches that touch no pixel in common give the same picture in either
    /// order, whatever their depth tests and blending, so the picture is the one the order the
    /// commands came in gives; a stage's frame then takes about half the draws (the ground's two
    /// passes per face), the handheld's driver's cost.
    fn gather(&mut self) {
        /// How many runs back a batch looks for its own (32 found nearly all of a frame's).
        const LOOK_BACK: usize = 32;
        struct Run {
            batch: Batch,
            members: smallvec::SmallVec<[usize; 4]>,
            /// All the members' areas.
            area: Area,
        }
        // Layered, each triangle's area (`may_share_pixels`).
        let triangle_areas: Vec<Area> = if self.layered.is_some() {
            let (triangles, _) = self.vertices.as_chunks::<3>();
            triangles.iter().map(|t| Area::around(t)).collect()
        } else {
            Vec::new()
        };
        let mut runs: Vec<Run> = Vec::with_capacity(self.batches.len());
        for (n, (batch, &area)) in self.batches.iter().zip(&self.areas).enumerate() {
            let mut joined = None;
            for (r, run) in runs.iter().enumerate().rev().take(LOOK_BACK) {
                if self.joinable(&run.batch, batch) {
                    joined = Some(r);
                    break;
                }
                if run.area.meets(area)
                    && run.members.iter().any(|&m| {
                        self.areas[m].meets(area) && self.may_share_pixels(m, n, &triangle_areas)
                    })
                {
                    break;
                }
            }
            match joined {
                Some(r) => {
                    runs[r].members.push(n);
                    runs[r].area = runs[r].area.with(area);
                }
                None => runs.push(Run {
                    batch: *batch,
                    members: smallvec::smallvec![n],
                    area,
                }),
            }
        }
        self.areas.clear();
        if runs.len() == self.batches.len() {
            return;
        }
        let mut vertices = Vec::with_capacity(self.vertices.len());
        let mut seconds = Vec::with_capacity(self.seconds.len());
        let mut vertex_textures = Vec::with_capacity(self.vertex_textures.len());
        let mut batches = Vec::with_capacity(runs.len());
        for run in runs {
            let first_vertex = vertices.len() as u32;
            for m in run.members {
                let b = &self.batches[m];
                let range = b.first_vertex as usize..(b.first_vertex + b.vertices) as usize;
                vertices.extend_from_slice(&self.vertices[range.clone()]);
                if !self.seconds.is_empty() {
                    seconds.extend_from_slice(&self.seconds[range.clone()]);
                }
                if !self.vertex_textures.is_empty() {
                    vertex_textures.extend_from_slice(&self.vertex_textures[range]);
                }
            }
            batches.push(Batch {
                first_vertex,
                vertices: vertices.len() as u32 - first_vertex,
                ..run.batch
            });
        }
        self.vertices = vertices;
        self.seconds = seconds;
        self.vertex_textures = vertex_textures;
        self.batches = batches;
    }

    /// Whether batches `a` and `b` (numbers in `batches`, before gathering), whose areas meet,
    /// may touch a pixel in common. Layered, triangles are looked at pair by pair (`apart`), each
    /// with its area (`triangle_areas`, by its first vertex's number over 3): a car's faces, each
    /// drawn twice (its texture, then its shine added), have areas that meet their neighbours'
    /// along every edge they share, though no pixel is drawn by both. A depth clear (a scissor
    /// over a triangle bigger than the target) touches everything its area meets.
    fn may_share_pixels(&self, a: usize, b: usize, triangle_areas: &[Area]) -> bool {
        if self.layered.is_none() {
            return true;
        }
        let (area_a, area_b) = (self.areas[a], self.areas[b]);
        let (a, b) = (&self.batches[a], &self.batches[b]);
        if a.scissor.is_some() || b.scissor.is_some() {
            return true;
        }
        // Each batch's triangles that reach into the other's area, by number.
        let near = |x: &Batch, other: Area| {
            let first = x.first_vertex as usize / 3;
            (first..first + x.vertices as usize / 3)
                .filter(move |&n| triangle_areas[n].meets(other))
        };
        let theirs: smallvec::SmallVec<[usize; 16]> = near(b, area_a).collect();
        let triangle = |n: usize| {
            let (chunks, _) = self.vertices[3 * n..3 * n + 3].as_chunks::<3>();
            &chunks[0]
        };
        near(a, area_b).any(|t| {
            theirs.iter().any(|&u| {
                triangle_areas[t].meets(triangle_areas[u])
                    && !apart(triangle(t), triangle(u), self.size)
            })
        })
    }

    /// Notes the fan just pushed as one that can take a second pass, when its own pass leaves
    /// exactly its colour: depth written, an opaque texture, every corner's alpha full, its
    /// alpha test passed at a full alpha, and a blend (if any) that keeps a full alpha's source.
    fn note_first(
        &mut self,
        vertices: &[Vertex],
        texture: Option<&TextureNote>,
        opaque: &dyn Fn(&TextureNote) -> bool,
    ) {
        self.last_fan = None;
        let Some(batch) = self.batches.last() else {
            return;
        };
        let count = (vertices.len() as u32 - 2) * 3;
        let keeps_source = matches!(batch.pipeline.blend, None | Some((5, 6)) | Some((2, 1)));
        if batch.pipeline.depth_write
            && batch.pipeline.colour
            && keeps_source
            && batch.scissor.is_none()
            && texture.is_some_and(opaque)
            && vertices.iter().all(|v| v[4] >> 24 == 0xFF)
            && alpha_passes(&batch.values, 1.0)
        {
            self.last_fan = Some(LastFan {
                first_vertex: self.vertices.len() as u32 - count,
                vertices: count,
                places: vertices.iter().map(|v| [v[0], v[1], v[2], v[3]]).collect(),
                batch: Batch {
                    first_vertex: 0,
                    vertices: 0,
                    ..*batch
                },
                area: Area::around(vertices),
            });
        }
    }

    /// Joins the fan as the last fan's second pass, when it is one: the same places, depth
    /// tested alike without writing, blended by its alpha. False when it is not.
    fn join_second(
        &mut self,
        vertices: &[Vertex],
        states: &[u32; STATES],
        texture: Option<&TextureNote>,
    ) -> bool {
        let Some(first) = self.last_fan.take() else {
            return false;
        };
        let pipeline = pipeline_of(states);
        let same_places = first.places.len() == vertices.len()
            && first
                .places
                .iter()
                .zip(vertices)
                .all(|(p, v)| p[..] == v[..4]);
        if !same_places
            || pipeline.depth_write
            || pipeline.blend != Some((5, 6))
            || pipeline.depth_function != first.batch.pipeline.depth_function
            || pipeline.cull != first.batch.pipeline.cull
        {
            return false;
        }
        let second = Second {
            values: values_of(states, texture),
            texture: texture.map(|t| self.texture_number(t)),
            linear: linear_of(states),
            clamp: clamp_of(states),
            mips: mips_of(states),
        };
        // The first fan's triangles leave the last batch, which they end, for a batch of
        // their own with the second pass.
        if let Some(last) = self.batches.last_mut() {
            last.vertices -= first.vertices;
            if last.vertices == 0 {
                self.batches.pop();
                self.areas.pop();
            }
        }
        let start = first.first_vertex as usize;
        for (n, corner) in (1..vertices.len() - 1)
            .flat_map(|n| [vertices[0], vertices[n], vertices[n + 1]])
            .enumerate()
        {
            self.seconds[start + n] = [corner[4], corner[5], corner[6], corner[7]];
        }
        if self.layered.is_some() {
            let number = second.texture.map_or(NO_TEXTURE, |n| n as u16);
            for v in &mut self.vertex_textures[start..start + first.vertices as usize] {
                v[1] = number;
            }
        }
        // Layered, `first.batch` (the batch the fan had joined) may name another texture of
        // the same arrays: the vertices' own numbers say which each reads.
        let mut values = first.batch.values;
        values.flags |= SECOND;
        self.push(
            Batch {
                first_vertex: first.first_vertex,
                vertices: first.vertices,
                pipeline: PipelineKey {
                    blend: None,
                    ..first.batch.pipeline
                },
                values,
                second: Some(second),
                ..first.batch
            },
            first.area,
        );
        true
    }

    /// `texture`'s number in `textures`, which it joins when it is new: looked up by its id,
    /// the last fan's first (most fans run on in the last one's texture).
    fn texture_number(&mut self, texture: &TextureNote) -> u32 {
        if let Some((pixels, number)) = self.last_texture
            && pixels == texture.pixels
        {
            return number;
        }
        let next = self.textures.len() as u32;
        let number = *self.texture_numbers.entry(texture.pixels).or_insert(next);
        if number == next {
            self.textures.push(*texture);
        }
        self.last_texture = Some((texture.pixels, number));
        number
    }

    fn push_fan(
        &mut self,
        vertices: &[Vertex],
        states: &[u32; STATES],
        texture: Option<&TextureNote>,
        opaque: Option<&dyn Fn(&TextureNote) -> bool>,
    ) {
        if vertices.len() < 3 {
            return;
        }
        let first_vertex = self.vertices.len() as u32;
        let area = Area::around(vertices);
        for n in 1..vertices.len() - 1 {
            self.vertices
                .extend([vertices[0], vertices[n], vertices[n + 1]]);
        }
        let number = texture.map(|t| self.texture_number(t));
        if self.layered.is_some() {
            let layer = number.map_or(NO_TEXTURE, |n| n as u16);
            self.vertex_textures
                .resize(self.vertices.len(), [layer, NO_TEXTURE]);
        }
        let mut pipeline = pipeline_of(states);
        let mut discard_free = false;
        if self.fast_path {
            // A texture whose opacity is not known counts as one with alpha.
            let clear = texture.is_none_or(|t| opaque.is_some_and(|opaque| opaque(t)));
            let fast = fan_fast_path(states, vertices, clear);
            discard_free = fast.discard_free;
            if fast.blend_off {
                pipeline.blend = None;
            }
        }
        let batch = Batch {
            first_vertex,
            vertices: (vertices.len() as u32 - 2) * 3,
            pipeline,
            values: values_of(states, texture),
            texture: number,
            linear: linear_of(states),
            clamp: clamp_of(states),
            mips: mips_of(states),
            scissor: None,
            second: None,
            discard_free,
        };
        self.push(batch, area);
    }

    fn push_clear(&mut self, [left, top, right, bottom]: [usize; 4], width: u32, height: u32) {
        let (right, bottom) = ((right as u32).min(width), (bottom as u32).min(height));
        let (left, top) = (left as u32, top as u32);
        if left >= right || top >= bottom {
            return;
        }
        let first_vertex = self.vertices.len() as u32;
        // A triangle over the whole target, at the farthest depth, cut by the scissor.
        let corner = |x: f32, y: f32| -> Vertex {
            [
                x.to_bits(),
                y.to_bits(),
                1f32.to_bits(),
                1f32.to_bits(),
                0,
                0,
                0,
                0,
            ]
        };
        let (w, h) = (width as f32, height as f32);
        // Wound clockwise on the screen, though nothing culls it.
        self.vertices.extend([
            corner(-1.0, -1.0),
            corner(2.0 * w, -1.0),
            corner(-1.0, 2.0 * h),
        ]);
        if self.layered.is_some() {
            self.vertex_textures
                .resize(self.vertices.len(), [NO_TEXTURE; 2]);
        }
        self.push(
            Batch {
                first_vertex,
                vertices: 3,
                pipeline: PipelineKey {
                    cull: 1,
                    depth_function: ALWAYS,
                    depth_write: true,
                    blend: None,
                    colour: false,
                },
                values: PixelValues {
                    fog_colour: [0.0; 4],
                    alpha_reference: 0.0,
                    alpha_function: 0,
                    mixing: 0,
                    flags: 0,
                },
                texture: None,
                linear: false,
                clamp: false,
                mips: false,
                scissor: Some([left, top, right, bottom]),
                second: None,
                // Nothing a shader decides is drawn; the program in use stays.
                discard_free: false,
            },
            // The triangle is the whole target's and more; the scissor is not counted on.
            Area::EVERYWHERE,
        );
    }

    /// `batch`, touching `area`, joined to the last one when they are drawn alike.
    fn push(&mut self, batch: Batch, area: Area) {
        let joins = self.batches.last().is_some_and(|last| {
            last.first_vertex + last.vertices == batch.first_vertex && self.joinable(last, &batch)
        });
        if let (true, Some(last), Some(last_area)) =
            (joins, self.batches.last_mut(), self.areas.last_mut())
        {
            last.vertices += batch.vertices;
            *last_area = last_area.with(area);
            return;
        }
        self.batches.push(batch);
        self.areas.push(area);
    }
}
