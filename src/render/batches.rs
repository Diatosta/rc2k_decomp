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
        out
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
        self.push(batch);
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
        self.push(Batch {
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
        });
    }

    /// `batch`, joined to the last one when they are drawn alike.
    fn push(&mut self, batch: Batch) {
        if let Some(last) = self.batches.last_mut()
            && last.first_vertex + last.vertices == batch.first_vertex
            && (Batch {
                first_vertex: last.first_vertex,
                vertices: last.vertices,
                ..batch
            }) == *last
        {
            last.vertices += batch.vertices;
            return;
        }
        self.batches.push(batch);
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
}
