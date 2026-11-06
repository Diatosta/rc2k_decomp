//! The commands as a GPU wants them: triangles in one list, and runs of them drawn alike. What a
//! state means is decided here, the same for every GPU backend; nothing here touches one.

use super::{Command, STATES, TextureNote};

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

pub const TEXTURED: i32 = 1;
pub const TEXTURE_HAS_ALPHA: i32 = 2;
pub const FOG: i32 = 4;

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
}

#[derive(Default)]
pub struct Batches {
    pub vertices: Vec<Vertex>,
    pub batches: Vec<Batch>,
    pub textures: Vec<TextureNote>,
    /// The pixels each batch can touch, while they are being gathered.
    areas: Vec<Area>,
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
    /// `commands` in the order they came.
    pub fn of(commands: &[Command], width: u32, height: u32) -> Batches {
        let mut out = Batches::default();
        for command in commands {
            match command {
                Command::Fan {
                    vertices,
                    states,
                    texture,
                    ..
                } => out.push_fan(vertices, states, texture.as_ref()),
                Command::Clear(rect) => out.push_clear(*rect, width, height),
            }
        }
        out.gather();
        out
    }

    /// Each batch moved back into the last run drawn like it, when no batch between them can
    /// touch its pixels. Batches that touch no pixel in common give the same picture in either
    /// order, whatever their depth tests and blending, so the picture is the one the order the
    /// commands came in gives; a stage's frame then takes about half the draws (the ground's two
    /// passes per face, 2026-11-06), the handheld's driver's cost.
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
                if alike(&run.batch, batch) {
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
        let mut batches = Vec::with_capacity(runs.len());
        for run in runs {
            let first_vertex = vertices.len() as u32;
            for m in run.members {
                let b = &self.batches[m];
                let start = b.first_vertex as usize;
                vertices.extend_from_slice(&self.vertices[start..start + b.vertices as usize]);
            }
            batches.push(Batch {
                first_vertex,
                vertices: vertices.len() as u32 - first_vertex,
                ..run.batch
            });
        }
        self.vertices = vertices;
        self.batches = batches;
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
        let batch = Batch {
            first_vertex,
            vertices: (vertices.len() as u32 - 2) * 3,
            pipeline: pipeline_of(states),
            values: values_of(states, texture),
            texture: texture.map(|t| self.texture_number(t)),
            // As the rasteriser reads them.
            linear: states[0x11] == 2 || matches!(states[0x12], 2 | 5 | 6),
            clamp: states[0x03] == 3,
            scissor: None,
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
            },
            // The triangle is the whole target's and more; the scissor is not counted on.
            Area::EVERYWHERE,
        );
    }

    /// `batch`, touching `area`, joined to the last one when they are drawn alike.
    fn push(&mut self, batch: Batch, area: Area) {
        if let (Some(last), Some(last_area)) = (self.batches.last_mut(), self.areas.last_mut())
            && last.first_vertex + last.vertices == batch.first_vertex
            && alike(last, &batch)
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

    fn fan(corners: usize, states: [u32; STATES]) -> Command {
        Command::Fan {
            vertices: smallvec::smallvec![[0; 8]; corners],
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
        let b = Batches::of(&[fan(4, plain), fan(3, plain), fan(3, blended)], 640, 480);
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
        let b = Batches::of(&[Command::Clear([10, 20, 9999, 30])], 640, 480);
        assert_eq!(b.batches[0].scissor, Some([10, 20, 640, 30]));
        assert!(!b.batches[0].pipeline.colour);
        assert!(
            Batches::of(&[Command::Clear([5, 5, 5, 9])], 640, 480)
                .batches
                .is_empty()
        );
    }

    /// A triangle over x `left ..= right`, y 0 ..= 10, marked by its first vertex's colour.
    fn triangle(left: f32, right: f32, mark: u32, states: [u32; STATES]) -> Command {
        let corner = |x: f32, y: f32| [x.to_bits(), y.to_bits(), 0, 0, mark, 0, 0, 0];
        Command::Fan {
            vertices: smallvec::smallvec![
                corner(left, 0.0),
                corner(right, 0.0),
                corner(left, 10.0)
            ],
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
            &[
                triangle(0.0, 10.0, 1, plain),
                triangle(20.0, 30.0, 2, blended),
                triangle(40.0, 50.0, 3, plain),
            ],
            640,
            480,
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
            &[
                triangle(0.0, 20.0, 1, plain),
                triangle(20.0, 30.0, 2, blended),
                triangle(20.0, 50.0, 3, plain),
            ],
            640,
            480,
        );
        assert_eq!(marks(&b), [vec![1], vec![2], vec![3]]);
        // A column apart, with a margin under a pixel, is apart.
        let b = Batches::of(
            &[
                triangle(0.0, 19.0, 1, plain),
                triangle(20.0, 30.0, 2, blended),
                triangle(31.0, 50.0, 3, plain),
            ],
            640,
            480,
        );
        assert_eq!(marks(&b), [vec![1, 3], vec![2]]);
    }

    #[test]
    fn nothing_gathers_past_a_clear() {
        let plain = [0; STATES];
        let b = Batches::of(
            &[
                triangle(0.0, 10.0, 1, plain),
                Command::Clear([100, 100, 200, 200]),
                triangle(40.0, 50.0, 3, plain),
            ],
            640,
            480,
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
            &[
                triangle(0.0, 10.0, 1, plain),
                triangle(f32::NAN, 30.0, 2, blended),
                triangle(40.0, 50.0, 3, plain),
            ],
            640,
            480,
        );
        assert_eq!(b.batches.len(), 3);
    }
}
