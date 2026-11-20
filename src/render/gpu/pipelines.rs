//! The shaders, samplers and pipelines, and the numbers that choose among them.

use super::*;

pub(super) fn sampler_number(batch: &Batch, cuts: Cuts) -> usize {
    let linear = (batch.linear || cuts.linear) && !cuts.nearest;
    linear as usize
        + 2 * batch.clamp as usize
        + 4 * (batch.mips && batch.texture.is_some()) as usize
}

/// The pipeline `key` asks for, with the state cuts applied: no blending, and no depth test
/// or writes.
pub(super) fn cut_key(key: PipelineKey, cuts: Cuts) -> PipelineKey {
    PipelineKey {
        blend: if cuts.blend { None } else { key.blend },
        // Direct3D's "always" (any number past the comparisons' seven).
        depth_function: if cuts.depth { 8 } else { key.depth_function },
        depth_write: key.depth_write && !cuts.depth,
        ..key
    }
}

pub(super) fn blend_factor(factor: u32) -> SDL_GPUBlendFactor {
    // As the rasteriser's `factor`.
    match factor {
        1 => SDL_GPU_BLENDFACTOR_ZERO,
        3 => SDL_GPU_BLENDFACTOR_SRC_COLOR,
        4 => SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_COLOR,
        5 => SDL_GPU_BLENDFACTOR_SRC_ALPHA,
        6 => SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
        7 => SDL_GPU_BLENDFACTOR_DST_ALPHA,
        8 => SDL_GPU_BLENDFACTOR_ONE_MINUS_DST_ALPHA,
        9 => SDL_GPU_BLENDFACTOR_DST_COLOR,
        10 => SDL_GPU_BLENDFACTOR_ONE_MINUS_DST_COLOR,
        _ => SDL_GPU_BLENDFACTOR_ONE,
    }
}

pub(super) fn compare_op(function: u32) -> SDL_GPUCompareOp {
    // As the rasteriser's `compare`.
    match function {
        1 => SDL_GPU_COMPAREOP_NEVER,
        2 => SDL_GPU_COMPAREOP_LESS,
        3 => SDL_GPU_COMPAREOP_EQUAL,
        4 => SDL_GPU_COMPAREOP_LESS_OR_EQUAL,
        5 => SDL_GPU_COMPAREOP_GREATER,
        6 => SDL_GPU_COMPAREOP_NOT_EQUAL,
        7 => SDL_GPU_COMPAREOP_GREATER_OR_EQUAL,
        _ => SDL_GPU_COMPAREOP_ALWAYS,
    }
}

impl Gpu {
    pub(super) fn shader(
        &self,
        code: ShaderCode,
        stage: SDL_GPUShaderStage,
        samplers: u32,
        uniforms: u32,
    ) -> Result<Shader, String> {
        self.device.create_shader(code, stage, samplers, uniforms)
    }

    pub(super) fn make_sampler(
        &self,
        linear: bool,
        clamp: bool,
        mips: bool,
    ) -> Result<Sampler, String> {
        let filter = if linear {
            SDL_GPU_FILTER_LINEAR
        } else {
            SDL_GPU_FILTER_NEAREST
        };
        let address = if clamp {
            SDL_GPU_SAMPLERADDRESSMODE_CLAMP_TO_EDGE
        } else {
            SDL_GPU_SAMPLERADDRESSMODE_REPEAT
        };
        let info = SDL_GPUSamplerCreateInfo {
            min_filter: filter,
            mag_filter: filter,
            // Linear inside the nearest smaller level; the levels past the first are read only by
            // the mipmapping samplers (`max_lod` 0 keeps the others to the full picture).
            mipmap_mode: SDL_GPU_SAMPLERMIPMAPMODE_NEAREST,
            max_lod: if mips { 1000.0 } else { 0.0 },
            address_mode_u: address,
            address_mode_v: address,
            address_mode_w: address,
            ..Default::default()
        };
        self.device.create_sampler(&info)
    }

    pub(in crate::render) fn make_texture(
        &self,
        width: u32,
        height: u32,
        usage: SDL_GPUTextureUsageFlags,
    ) -> Result<Texture, String> {
        self.make_texture_of(COLOUR_FORMAT, width, height, usage)
    }

    /// A texture for the game's picture of `width` x `height`, with its smaller levels when they
    /// are being made.
    pub(super) fn make_picture_texture(&self, width: u32, height: u32) -> Result<Texture, String> {
        let levels = if self.mip_levels {
            mip_levels::level_count(width as usize, height as usize) as u32
        } else {
            1
        };
        self.device.create_texture_levels(
            COLOUR_FORMAT,
            width,
            height,
            SDL_GPU_TEXTUREUSAGE_SAMPLER,
            levels,
        )
    }

    pub(in crate::render) fn make_texture_of(
        &self,
        format: SDL_GPUTextureFormat,
        width: u32,
        height: u32,
        usage: SDL_GPUTextureUsageFlags,
    ) -> Result<Texture, String> {
        self.device.create_texture(format, width, height, usage)
    }

    pub(super) fn make_overlay_pipeline(
        &self,
        vertex_shader: Shader,
        fragment_shader: Shader,
    ) -> Result<Pipeline, String> {
        let spec = PipelineSpec {
            vertices: None,
            // Translucent pixels (a button picture's edge, `overlay.frag`) blend with the target;
            // opaque ones have alpha 1 and replace it.
            colour: SDL_GPUColorTargetDescription {
                format: COLOUR_FORMAT,
                blend_state: SDL_GPUColorTargetBlendState {
                    enable_blend: true,
                    src_color_blendfactor: SDL_GPU_BLENDFACTOR_SRC_ALPHA,
                    dst_color_blendfactor: SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
                    color_blend_op: SDL_GPU_BLENDOP_ADD,
                    src_alpha_blendfactor: SDL_GPU_BLENDFACTOR_ONE,
                    dst_alpha_blendfactor: SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
                    alpha_blend_op: SDL_GPU_BLENDOP_ADD,
                    ..Default::default()
                },
            },
            rasterizer: SDL_GPURasterizerState::default(),
            depth: None,
        };
        self.device
            .create_pipeline(vertex_shader, fragment_shader, &spec)
    }

    /// The merge of a menu's larger 3D and its frame (`merge.frag`): every pixel replaced.
    pub(super) fn make_merge_pipeline(
        &self,
        vertex_shader: Shader,
        fragment_shader: Shader,
    ) -> Result<Pipeline, String> {
        let spec = PipelineSpec {
            vertices: None,
            colour: SDL_GPUColorTargetDescription {
                format: COLOUR_FORMAT,
                blend_state: SDL_GPUColorTargetBlendState::default(),
            },
            rasterizer: SDL_GPURasterizerState::default(),
            depth: None,
        };
        self.device
            .create_pipeline(vertex_shader, fragment_shader, &spec)
    }

    pub(super) fn pipeline(&mut self, key: PipelineKey) -> Result<Pipeline, String> {
        if let Some(&pipeline) = self.pipelines.get(&key) {
            return Ok(pipeline);
        }
        let attributes = [
            (0, SDL_GPU_VERTEXELEMENTFORMAT_FLOAT4, 0),
            (1, SDL_GPU_VERTEXELEMENTFORMAT_UBYTE4_NORM, 16),
            (2, SDL_GPU_VERTEXELEMENTFORMAT_UBYTE4_NORM, 20),
            (3, SDL_GPU_VERTEXELEMENTFORMAT_FLOAT2, 24),
        ]
        .map(|(location, format, offset)| SDL_GPUVertexAttribute {
            location,
            buffer_slot: 0,
            format,
            offset,
        });
        let mut blend = SDL_GPUColorTargetBlendState {
            enable_color_write_mask: !key.colour,
            ..Default::default()
        };
        if let Some((source, target)) = key.blend {
            blend.enable_blend = true;
            blend.src_color_blendfactor = blend_factor(source);
            blend.dst_color_blendfactor = blend_factor(target);
            blend.color_blend_op = SDL_GPU_BLENDOP_ADD;
            blend.src_alpha_blendfactor = blend_factor(source);
            blend.dst_alpha_blendfactor = blend_factor(target);
            blend.alpha_blend_op = SDL_GPU_BLENDOP_ADD;
        }
        let spec = PipelineSpec {
            vertices: Some((VERTEX_BYTES, &attributes)),
            colour: SDL_GPUColorTargetDescription {
                format: COLOUR_FORMAT,
                blend_state: blend,
            },
            rasterizer: SDL_GPURasterizerState {
                fill_mode: SDL_GPU_FILLMODE_FILL,
                cull_mode: match key.cull {
                    3 => SDL_GPU_CULLMODE_BACK,
                    2 => SDL_GPU_CULLMODE_FRONT,
                    _ => SDL_GPU_CULLMODE_NONE,
                },
                // The triangles the game shows are clockwise on the screen.
                front_face: SDL_GPU_FRONTFACE_CLOCKWISE,
                // Clamped where the device can (see `Device::create`), clipped where it cannot.
                enable_depth_clip: !sdl_gpu::depth_clamp(),
                ..Default::default()
            },
            depth: Some((
                SDL_GPUDepthStencilState {
                    compare_op: compare_op(key.depth_function),
                    enable_depth_test: true,
                    enable_depth_write: key.depth_write,
                    ..Default::default()
                },
                DEPTH_FORMAT,
            )),
        };
        let pipeline = self
            .device
            .create_pipeline(self.scene_vert, self.scene_frag, &spec)?;
        self.pipelines.insert(key, pipeline);
        Ok(pipeline)
    }
}
