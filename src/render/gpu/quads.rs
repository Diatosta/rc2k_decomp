//! The GPU's side of the 2D quads (`gfx::draw2d::quads`): the atlas texture, the texels sent to
//! it, and the draw of the noted rectangles, one draw call for all of a flush's. They are drawn
//! after the canvas' pixels and before the flush's 3D, over the colour target, opaque.

use super::{COLOUR_FORMAT, Gpu, ShaderCode};
use crate::gfx::draw2d::quads::{ATLAS_SIDE, Taken, vertices};
use crate::platform::ffi::sdl_gpu::{
    self, Buffer, CommandBuffer, CopyPass, Pipeline, PipelineSpec, RenderPass, Texture, Transfer,
};
use sdl3_sys::everything::*;

#[cfg(not(target_os = "macos"))]
const QUAD_VERT: ShaderCode = ShaderCode {
    spirv: include_bytes!("../shaders/quad.vert.spv"),
};
#[cfg(not(target_os = "macos"))]
const QUAD_FRAG: ShaderCode = ShaderCode {
    spirv: include_bytes!("../shaders/quad.frag.spv"),
};
#[cfg(target_os = "macos")]
const QUAD_VERT: ShaderCode = ShaderCode {
    spirv: include_bytes!("../shaders/quad.vert.spv"),
    msl: include_bytes!("../shaders/quad.vert.msl"),
};
#[cfg(target_os = "macos")]
const QUAD_FRAG: ShaderCode = ShaderCode {
    spirv: include_bytes!("../shaders/quad.frag.spv"),
    msl: include_bytes!("../shaders/quad.frag.msl"),
};

/// A vertex: its place on the screen and its atlas texel (four floats), and its palette's first
/// texel (two floats).
const VERTEX_BYTES: u32 = 24;

/// What the 2D quads keep on the GPU.
pub struct Quads {
    atlas: Texture,
    pipeline: Pipeline,
    vertices: Buffer,
    vertices_room: u32,
    transfer: Transfer,
    transfer_room: u32,
}

/// A flush's 2D, sent to the staging buffer and waiting for the copy pass and the render pass.
pub struct Prepared {
    /// The vertices to draw, and their bytes at the start of the staging buffer.
    vertices: u32,
    vertex_bytes: u32,
    /// Each block of atlas texels: where it is in the staging buffer, and its place in the atlas.
    uploads: Vec<(u32, [u32; 4])>,
}

impl Prepared {
    /// Nothing to draw or send.
    fn none() -> Self {
        Self {
            vertices: 0,
            vertex_bytes: 0,
            uploads: Vec::new(),
        }
    }

    /// Whether quads are to be drawn.
    pub fn draws(&self) -> bool {
        self.vertices > 0
    }
}

impl Quads {
    /// Everything of the 2D quads' released on `device`.
    pub(super) fn release(&self, device: sdl_gpu::Device) {
        device.release_texture(self.atlas);
        device.release_pipeline(self.pipeline);
        if !self.vertices.is_none() {
            device.release_buffer(self.vertices);
        }
        if !self.transfer.is_none() {
            device.release_transfer(self.transfer);
        }
    }
}

impl Gpu {
    /// The atlas, the pipeline and the buffers, made the first time something is to be drawn.
    fn ensure_quads(&mut self) -> Result<(), String> {
        if self.quads.is_some() {
            return Ok(());
        }
        let vertex_shader = self.shader(QUAD_VERT, SDL_GPU_SHADERSTAGE_VERTEX, 0, 1)?;
        let fragment_shader = self.shader(QUAD_FRAG, SDL_GPU_SHADERSTAGE_FRAGMENT, 1, 0)?;
        let attributes = [
            (0, SDL_GPU_VERTEXELEMENTFORMAT_FLOAT4, 0),
            (1, SDL_GPU_VERTEXELEMENTFORMAT_FLOAT2, 16),
        ]
        .map(|(location, format, offset)| SDL_GPUVertexAttribute {
            location,
            buffer_slot: 0,
            format,
            offset,
        });
        let spec = PipelineSpec {
            vertices: Some((VERTEX_BYTES, &attributes)),
            colour: SDL_GPUColorTargetDescription {
                format: COLOUR_FORMAT,
                ..Default::default()
            },
            rasterizer: SDL_GPURasterizerState::default(),
            depth: None,
        };
        let pipeline = self
            .device
            .create_pipeline(vertex_shader, fragment_shader, &spec);
        // The pipeline keeps what it needs of them.
        self.device.release_shader(vertex_shader);
        self.device.release_shader(fragment_shader);
        let pipeline = pipeline?;
        let atlas = self.make_texture(ATLAS_SIDE, ATLAS_SIDE, SDL_GPU_TEXTUREUSAGE_SAMPLER)?;
        self.quads = Some(Quads {
            atlas,
            pipeline,
            vertices: Buffer::NONE,
            vertices_room: 0,
            transfer: Transfer::NONE,
            transfer_room: 0,
        });
        Ok(())
    }

    /// The flush's noted rectangles and the atlas texels waiting, in the staging buffer.
    pub(super) fn prepare_2d(&mut self, taken: Taken) -> Result<Prepared, String> {
        let Taken {
            rects,
            uploads,
            size: _,
        } = taken.fitted_to((self.width, self.height));
        if rects.is_empty() && uploads.is_empty() {
            return Ok(Prepared::none());
        }
        self.ensure_quads()?;
        let mut staging: Vec<u8> = Vec::new();
        staging.extend_from_slice(sdl_gpu::words_as_bytes(&vertices(&rects)));
        let vertex_bytes = staging.len() as u32;
        let mut placed = Vec::with_capacity(uploads.len());
        for upload in &uploads {
            placed.push((
                staging.len() as u32,
                [upload.x, upload.y, upload.width, upload.height],
            ));
            for texel in &upload.texels {
                staging.extend_from_slice(texel);
            }
        }
        let device = self.device;
        let Some(quads) = self.quads.as_mut() else {
            return Ok(Prepared::none());
        };
        if vertex_bytes > quads.vertices_room {
            let room = vertex_bytes.next_power_of_two().max(1 << 16);
            if !quads.vertices.is_none() {
                device.release_buffer(quads.vertices);
            }
            quads.vertices = Buffer::NONE;
            quads.vertices = device.create_vertex_buffer(room)?;
            quads.vertices_room = room;
        }
        let staged = staging.len() as u32;
        if staged > quads.transfer_room {
            let room = staged.next_power_of_two().max(1 << 18);
            if !quads.transfer.is_none() {
                device.release_transfer(quads.transfer);
            }
            quads.transfer = device.create_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD, room)?;
            quads.transfer_room = room;
        }
        device.fill_transfer(quads.transfer, 0, &staging, true)?;
        Ok(Prepared {
            vertices: rects.len() as u32 * 6,
            vertex_bytes,
            uploads: placed,
        })
    }

    /// The vertices and the atlas texels sent, in the copy pass.
    pub(super) fn copy_2d(&self, copy: &CopyPass, prepared: &Prepared) {
        let Some(quads) = &self.quads else {
            return;
        };
        if prepared.vertex_bytes > 0 {
            copy.upload_buffer(quads.transfer, quads.vertices, prepared.vertex_bytes);
        }
        for &(offset, region) in &prepared.uploads {
            copy.upload_texture_region(quads.transfer, offset, quads.atlas, region);
        }
    }

    /// The rectangles drawn, in an open pass.
    pub(super) fn draw_2d(&self, buffer: &CommandBuffer, pass: &RenderPass, prepared: &Prepared) {
        let Some(quads) = &self.quads else {
            return;
        };
        if !prepared.draws() {
            return;
        }
        buffer.push_vertex_uniform(0, &[self.width as f32, self.height as f32]);
        pass.bind_pipeline(quads.pipeline);
        pass.bind_vertex_buffer(quads.vertices);
        pass.bind_fragment_sampler(quads.atlas, self.nearest);
        pass.draw(prepared.vertices, 0);
    }
}
