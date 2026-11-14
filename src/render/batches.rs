//! The commands as a GPU wants them: triangles in one list, and runs of them drawn alike. What a
//! state means is decided here, the same for every GPU backend; nothing here touches one.

use super::{Command, CommandList, STATES, TextureNote};

/// A vertex as the game makes it: x, y, z, 1/w, colour, specular, u, v.
pub type Vertex = crate::rasteriser::Vertex;

/// What makes up a pipeline. The comparisons and factors are Direct3D's numbers.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PipelineKey {
    /// Direct3D's culling: 1 none, 2 clockwise, 3 counter-clockwise on the screen.
    pub cull: u32,
    /// The depth test's comparison (always, when the test is off: writing goes on regardless,
    /// which a GPU's switched-off test would stop).
    pub depth_function: u32,
    pub depth_write: bool,
    /// The blend's source and target factors.
    pub blend: Option<(u32, u32)>,
    /// False for a depth clear: only the depth is written.
    pub colour: bool,
}

/// The pixel shader's values (`scene.frag`'s `Pixel`).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PixelValues {
    pub fog_colour: [f32; 4],
    pub alpha_reference: f32,
    pub alpha_function: i32,
    pub mixing: i32,
    pub flags: i32,
}

impl PixelValues {
    /// The values' dwords as the shader's uniform block holds them.
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    pub fn to_words(self) -> [u32; 8] {
        let [red, green, blue, alpha] = self.fog_colour;
        [
            red.to_bits(),
            green.to_bits(),
            blue.to_bits(),
            alpha.to_bits(),
            self.alpha_reference.to_bits(),
            self.alpha_function as u32,
            self.mixing as u32,
            self.flags as u32,
        ]
    }
}

pub const TEXTURED: i32 = 1;
pub const TEXTURE_HAS_ALPHA: i32 = 2;
pub const FOG: i32 = 4;
/// The batch draws a second pass over its first in the same draw (`Batch::second`).
pub const SECOND: i32 = 8;

/// A second pass drawn over a batch's own in the same draw: the face drawn again at once
/// without depth writes, blended by its alpha (the ground's second texture). Its vertices' colour,
/// specular and texture place are in `Batches::seconds`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Second {
    pub values: PixelValues,
    pub texture: Option<u32>,
    pub linear: bool,
    pub clamp: bool,
}

/// A run of triangles drawn alike.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Batch {
    pub first_vertex: u32,
    pub vertices: u32,
    pub pipeline: PipelineKey,
    pub values: PixelValues,
    /// Which of `Batches::textures`.
    pub texture: Option<u32>,
    pub linear: bool,
    pub clamp: bool,
    /// Scissor rectangle for a depth clear: left, top, right, bottom.
    pub scissor: Option<[u32; 4]>,
    pub second: Option<Second>,
}

#[derive(Default)]
pub struct Batches {
    pub vertices: Vec<Vertex>,
    pub batches: Vec<Batch>,
    pub textures: Vec<TextureNote>,
    /// With second passes joined (`of`), each vertex's second pass's colour, specular and
    /// texture place (vertices of batches without one: zeros).
    pub seconds: Vec<[u32; 4]>,
    /// Layered (`of_layered`): each vertex's first and second pass's texture, by number in
    /// `textures` (`NO_TEXTURE` for none).
    pub vertex_textures: Vec<[u16; 2]>,
    /// Layered: whether two textures can be read in one draw.
    layered: Option<fn(&TextureNote, &TextureNote) -> bool>,
    /// The pixels each batch can touch, while they are being gathered.
    areas: Vec<Area>,
    /// The last fan, while it can still take a second pass.
    last_fan: Option<LastFan>,
}

/// A pass without a texture, in `Batches::vertex_textures`.
pub const NO_TEXTURE: u16 = u16::MAX;

/// A fan that can take a second pass.
struct LastFan {
    /// Its triangles' first vertex and count: the last batch's tail.
    first_vertex: u32,
    vertices: u32,
    /// Its corners' places (x, y, z, 1/w).
    places: smallvec::SmallVec<[[u32; 4]; 6]>,
    batch: Batch,
    area: Area,
}

/// The pixels a batch can touch, by column and row, both ends in: a rectangle around them.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Area {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Area {
    const EVERYWHERE: Area = Area {
        left: i32::MIN,
        top: i32::MIN,
        right: i32::MAX,
        bottom: i32::MAX,
    };

    /// Around `vertices`. The shaders put a game place `x` at `x + 0.5` on the target, where
    /// pixel `x`'s centre is: a triangle touches the pixels whose centres it holds, so the
    /// columns from its least x up to its most (rows the same). The margin covers the GPU's
    /// rounding to its sub-pixel grid (a sixteenth at the coarsest Vulkan allows).
    fn around(vertices: &[Vertex]) -> Area {
        const MARGIN: f32 = 0.125;
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
        // `as` saturates: a place far off the target stays far off.
        Area {
            left: (left - MARGIN).ceil() as i32,
            top: (top - MARGIN).ceil() as i32,
            right: (right + MARGIN).floor() as i32,
            bottom: (bottom + MARGIN).floor() as i32,
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

/// Whether two batches are drawn alike, wherever their triangles lie.
fn alike(a: &Batch, b: &Batch) -> bool {
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

const ALWAYS: u32 = 8;

// As the rasteriser reads them.
fn linear_of(states: &[u32; STATES]) -> bool {
    states[0x11] == 2 || matches!(states[0x12], 2 | 5 | 6)
}

fn clamp_of(states: &[u32; STATES]) -> bool {
    states[0x03] == 3
}

/// Whether the pixel shader's alpha test (`passes`) keeps a pixel of alpha `value`.
fn alpha_passes(values: &PixelValues, value: f32) -> bool {
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

fn pipeline_of(states: &[u32; STATES]) -> PipelineKey {
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

fn values_of(states: &[u32; STATES], texture: Option<&TextureNote>) -> PixelValues {
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
    #[cfg(any(test, all(target_os = "linux", target_arch = "aarch64")))]
    pub fn of_layered(
        commands: &CommandList,
        width: u32,
        height: u32,
        opaque: &dyn Fn(&TextureNote) -> bool,
        same_array: fn(&TextureNote, &TextureNote) -> bool,
    ) -> Batches {
        let start = Batches {
            layered: Some(same_array),
            ..Batches::default()
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
                        out.push_fan(vertices, states, texture.as_ref());
                        continue;
                    };
                    if out.join_second(vertices, states, texture.as_ref()) {
                        continue;
                    }
                    out.push_fan(vertices, states, texture.as_ref());
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
        let mut runs: Vec<Run> = Vec::with_capacity(self.batches.len());
        for (n, (batch, &area)) in self.batches.iter().zip(&self.areas).enumerate() {
            let mut joined = None;
            for (r, run) in runs.iter().enumerate().rev().take(LOOK_BACK) {
                if self.joinable(&run.batch, batch) {
                    joined = Some(r);
                    break;
                }
                if run.area.meets(area) && run.members.iter().any(|&m| self.areas[m].meets(area)) {
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

    fn texture_number(&mut self, texture: &TextureNote) -> u32 {
        let found = self
            .textures
            .iter()
            .position(|t| t.pixels == texture.pixels);
        found.unwrap_or_else(|| {
            self.textures.push(*texture);
            self.textures.len() - 1
        }) as u32
    }

    fn push_fan(
        &mut self,
        vertices: &[Vertex],
        states: &[u32; STATES],
        texture: Option<&TextureNote>,
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
        let batch = Batch {
            first_vertex,
            vertices: (vertices.len() as u32 - 2) * 3,
            pipeline: pipeline_of(states),
            values: values_of(states, texture),
            texture: number,
            linear: linear_of(states),
            clamp: clamp_of(states),
            scissor: None,
            second: None,
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
                scissor: Some([left, top, right, bottom]),
                second: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::render::{TestCommand, list};

    fn fan(corners: usize, states: [u32; STATES]) -> TestCommand {
        TestCommand::Fan {
            vertices: vec![[0; 8]; corners],
            states: std::sync::Arc::new(states),
            texture: None,
            bounds: (0.0, 0.0, 0.0, 0.0),
        }
    }

    #[test]
    fn fans_become_triangles_and_alike_runs_join() {
        let plain = [0; STATES];
        let mut blended = plain;
        blended[0x1B] = 1;
        let b = Batches::of(
            &list([fan(4, plain), fan(3, plain), fan(3, blended)]),
            640,
            480,
            None,
        );
        assert_eq!(b.vertices.len(), 12);
        assert_eq!(b.batches.len(), 2);
        assert_eq!(b.batches[0].vertices, 9);
        assert_eq!(b.batches[1].first_vertex, 9);
    }

    #[test]
    fn a_switched_off_depth_test_still_writes() {
        let mut states = [0; STATES];
        states[0x0E] = 1;
        states[0x17] = 2;
        let p = pipeline_of(&states);
        assert_eq!((p.depth_function, p.depth_write), (ALWAYS, true));
        states[0x07] = 1;
        assert_eq!(pipeline_of(&states).depth_function, 2);
    }

    #[test]
    fn a_clear_is_cut_to_the_target_and_only_touches_depth() {
        let b = Batches::of(
            &list([TestCommand::Clear([10, 20, 9999, 30])]),
            640,
            480,
            None,
        );
        assert_eq!(b.batches[0].scissor, Some([10, 20, 640, 30]));
        assert!(!b.batches[0].pipeline.colour);
        assert!(
            Batches::of(&list([TestCommand::Clear([5, 5, 5, 9])]), 640, 480, None)
                .batches
                .is_empty()
        );
    }

    /// A triangle over x `left ..= right`, y 0 ..= 10, marked by its first vertex's colour.
    fn triangle(left: f32, right: f32, mark: u32, states: [u32; STATES]) -> TestCommand {
        let corner = |x: f32, y: f32| [x.to_bits(), y.to_bits(), 0, 0, mark, 0, 0, 0];
        TestCommand::Fan {
            vertices: vec![corner(left, 0.0), corner(right, 0.0), corner(left, 10.0)],
            states: std::sync::Arc::new(states),
            texture: None,
            bounds: (0.0, 0.0, 0.0, 0.0),
        }
    }

    fn marks(b: &Batches) -> Vec<Vec<u32>> {
        let batch = |x: &Batch| {
            let first = x.first_vertex as usize;
            b.vertices[first..first + x.vertices as usize]
                .iter()
                .step_by(3)
                .map(|v| v[4])
                .collect()
        };
        b.batches.iter().map(batch).collect()
    }

    #[test]
    fn alike_batches_gather_past_others_that_touch_none_of_their_pixels() {
        let plain = [0; STATES];
        let mut blended = plain;
        blended[0x1B] = 1;
        let b = Batches::of(
            &list([
                triangle(0.0, 10.0, 1, plain),
                triangle(20.0, 30.0, 2, blended),
                triangle(40.0, 50.0, 3, plain),
            ]),
            640,
            480,
            None,
        );
        assert_eq!(marks(&b), [vec![1, 3], vec![2]]);
        assert_eq!(b.vertices.len(), 9);
    }

    #[test]
    fn a_batch_between_that_touches_the_same_pixels_keeps_the_order() {
        let plain = [0; STATES];
        let mut blended = plain;
        blended[0x1B] = 1;
        // Pixel column 20 is in all three: the order there is the picture.
        let b = Batches::of(
            &list([
                triangle(0.0, 20.0, 1, plain),
                triangle(20.0, 30.0, 2, blended),
                triangle(20.0, 50.0, 3, plain),
            ]),
            640,
            480,
            None,
        );
        assert_eq!(marks(&b), [vec![1], vec![2], vec![3]]);
        // A column apart, with a margin under a pixel, is apart.
        let b = Batches::of(
            &list([
                triangle(0.0, 19.0, 1, plain),
                triangle(20.0, 30.0, 2, blended),
                triangle(31.0, 50.0, 3, plain),
            ]),
            640,
            480,
            None,
        );
        assert_eq!(marks(&b), [vec![1, 3], vec![2]]);
    }

    #[test]
    fn nothing_gathers_past_a_clear() {
        let plain = [0; STATES];
        let b = Batches::of(
            &list([
                triangle(0.0, 10.0, 1, plain),
                TestCommand::Clear([100, 100, 200, 200]),
                triangle(40.0, 50.0, 3, plain),
            ]),
            640,
            480,
            None,
        );
        assert_eq!(b.batches.len(), 3);
        assert_eq!(b.batches[1].scissor, Some([100, 100, 200, 200]));
    }

    #[test]
    fn a_place_that_is_no_number_touches_everything() {
        let plain = [0; STATES];
        let mut blended = plain;
        blended[0x1B] = 1;
        let b = Batches::of(
            &list([
                triangle(0.0, 10.0, 1, plain),
                triangle(f32::NAN, 30.0, 2, blended),
                triangle(40.0, 50.0, 3, plain),
            ]),
            640,
            480,
            None,
        );
        assert_eq!(b.batches.len(), 3);
    }

    /// A textured fan of `corners` places (the same each time), its colour `colour`, its
    /// texture at `texture` (an identity only).
    fn textured(corners: usize, colour: u32, texture: u32, states: [u32; STATES]) -> TestCommand {
        let corner = |n: usize| {
            let (x, y) = (n as f32 * 10.0, (n % 2) as f32 * 10.0);
            [x.to_bits(), y.to_bits(), 0, 0, colour, 0x7F00_0000, 3, 4]
        };
        TestCommand::Fan {
            vertices: (0..corners).map(corner).collect(),
            states: std::sync::Arc::new(states),
            texture: Some(TextureNote {
                pixels: crate::render::TextureId {
                    surface: texture,
                    serial: 0,
                },
                len: 4,
                width: 1,
                height: 1,
                pitch: 4,
                bytes: 4,
                masks: [0xFF_0000, 0xFF00, 0xFF, 0xFF00_0000],
            }),
            bounds: (0.0, 0.0, 0.0, 0.0),
        }
    }

    /// The ground's passes: the first writes depth, the second does not and blends.
    fn passes() -> ([u32; STATES], [u32; STATES]) {
        let mut first = [0; STATES];
        first[0x07] = 1;
        first[0x17] = 4;
        first[0x0E] = 1;
        first[0x1B] = 1;
        first[0x13] = 5;
        first[0x14] = 6;
        let mut second = first;
        second[0x0E] = 0;
        (first, second)
    }

    const OPAQUE: u32 = 0x1000;
    const SEE_THROUGH: u32 = 0x2000;

    fn joined(commands: impl IntoIterator<Item = TestCommand>) -> Batches {
        Batches::of(
            &list(commands),
            640,
            480,
            Some(&|t: &TextureNote| t.pixels.surface == OPAQUE),
        )
    }

    #[test]
    fn a_second_pass_over_the_same_places_joins_the_first() {
        let (first, second) = passes();
        let b = joined([
            textured(4, 0xFF00_0000, OPAQUE, first),
            textured(4, 0x8000_0000, SEE_THROUGH, second),
        ]);
        assert_eq!(b.batches.len(), 1);
        let batch = &b.batches[0];
        assert_eq!(batch.vertices, 6);
        assert_eq!(batch.pipeline.blend, None);
        assert!(batch.pipeline.depth_write);
        assert_ne!(batch.values.flags & SECOND, 0);
        let s = batch.second.expect("a second pass");
        assert_eq!(s.texture, Some(1));
        assert_eq!(b.seconds.len(), 6);
        assert!(
            b.seconds
                .iter()
                .all(|v| *v == [0x8000_0000, 0x7F00_0000, 3, 4])
        );
    }

    #[test]
    fn a_second_pass_joins_only_over_an_opaque_full_alpha_first() {
        let (first, second) = passes();
        // A texture with see-through pixels, corners not fully opaque, other places: apart.
        for commands in [
            [
                textured(4, 0xFF00_0000, SEE_THROUGH, first),
                textured(4, 0x8000_0000, OPAQUE, second),
            ],
            [
                textured(4, 0xFE00_0000, OPAQUE, first),
                textured(4, 0x8000_0000, OPAQUE, second),
            ],
            [
                textured(4, 0xFF00_0000, OPAQUE, first),
                textured(5, 0x8000_0000, OPAQUE, second),
            ],
            [
                textured(4, 0xFF00_0000, OPAQUE, first),
                textured(4, 0x8000_0000, OPAQUE, first),
            ],
        ] {
            let b = joined(commands);
            assert!(b.batches.iter().all(|b| b.second.is_none()));
            assert_eq!(b.seconds.len(), b.vertices.len());
        }
    }

    /// `textured`'s, with a texture of `size` x `size`.
    fn sized(texture: u32, size: usize, x: f32, states: [u32; STATES]) -> TestCommand {
        let mut command = textured(3, 0xFF00_0000, texture, states);
        if let TestCommand::Fan {
            vertices,
            texture: Some(note),
            ..
        } = &mut command
        {
            (note.width, note.height) = (size, size);
            for v in vertices.iter_mut() {
                v[0] = (f32::from_bits(v[0]) + x).to_bits();
            }
        }
        command
    }

    #[test]
    fn layered_batches_join_across_textures_of_the_same_arrays() {
        let plain = [0; STATES];
        let same = |a: &TextureNote, b: &TextureNote| {
            let main = |t: &TextureNote| matches!(t.width, 128 | 256);
            a.width == b.width || (main(a) && main(b))
        };
        let b = Batches::of_layered(
            &list([
                sized(0x10, 128, 0.0, plain),
                sized(0x20, 256, 0.0, plain),
                sized(0x30, 64, 0.0, plain),
                sized(0x40, 128, 100.0, plain),
            ]),
            640,
            480,
            &|_| false,
            same,
        );
        // The 64 x 64 one apart (it overlaps the first two: the last one, apart from it,
        // gathers back into theirs).
        assert_eq!(b.batches.len(), 2);
        assert_eq!(b.batches[0].vertices, 9);
        let numbers: Vec<u16> = b.vertex_textures.iter().map(|v| v[0]).collect();
        assert_eq!(numbers, [0, 0, 0, 1, 1, 1, 3, 3, 3, 2, 2, 2]);
        assert!(b.vertex_textures.iter().all(|v| v[1] == NO_TEXTURE));
        // Unlayered, one texture a draw.
        let commands = list([sized(0x10, 128, 0.0, plain), sized(0x20, 256, 0.0, plain)]);
        assert_eq!(Batches::of(&commands, 640, 480, None).batches.len(), 2);
        assert!(
            Batches::of(&commands, 640, 480, None)
                .vertex_textures
                .is_empty()
        );
    }

    #[test]
    fn the_first_pass_leaves_the_batch_it_had_joined() {
        let (first, second) = passes();
        let b = joined([
            textured(3, 0xFF00_0000, OPAQUE, first),
            textured(4, 0xFF00_0000, OPAQUE, first),
            textured(4, 0x8000_0000, SEE_THROUGH, second),
        ]);
        // The first fan alone, then the second fan with its pass: their places meet, so in
        // that order.
        assert_eq!(b.batches.len(), 2);
        assert_eq!((b.batches[0].vertices, b.batches[0].second), (3, None));
        assert_eq!(b.batches[1].vertices, 6);
        assert!(b.batches[1].second.is_some());
        assert_eq!(b.seconds[..3], [[0; 4]; 3]);
    }
}
