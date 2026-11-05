//! The GPU backend, on SDL's GPU API (Vulkan on the PC). All of this file's `unsafe` is SDL's calls:
//! each handle here was made by the same device and lives as long as `Gpu`, or until the line
//! that replaces it.
//!
//! A race's frame is 3D in the GPU's colour target and the game's 2D on the screen's memory (the
//! canvas), which starts each layer holding `UNWRITTEN` in every pixel. When the two meet, the
//! pixels the game has written go over the target and the canvas is marked again. Something that
//! wants the true picture in the canvas asks `bring_back`, which is slow on a handheld.

#![allow(unsafe_code)]

use super::batches::{Batch, Batches, PipelineKey};
use super::{Command, Target, TextureNote};
use crate::framework::log;
use sdl3_sys::everything::*;
use std::collections::HashMap;
use std::ptr::{null, null_mut};
use std::sync::Mutex;

/// A canvas pixel the game has not written: magenta, with an alpha byte the screen's pixels are
/// not written with (`overlay.frag`).
pub const UNWRITTEN: u32 = 0xA5FF_00FF;

const SCENE_VERT: &[u8] = include_bytes!("shaders/scene.vert.spv");
const SCENE_FRAG: &[u8] = include_bytes!("shaders/scene.frag.spv");
const OVERLAY_VERT: &[u8] = include_bytes!("shaders/overlay.vert.spv");
const OVERLAY_FRAG: &[u8] = include_bytes!("shaders/overlay.frag.spv");

const COLOUR_FORMAT: SDL_GPUTextureFormat = SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM;
const DEPTH_FORMAT: SDL_GPUTextureFormat = SDL_GPU_TEXTUREFORMAT_D32_FLOAT;
const VERTEX_BYTES: u32 = 32;

struct CachedTexture {
    texture: *mut SDL_GPUTexture,
    width: usize,
    height: usize,
    stale: bool,
}

/// The targets a frame is drawn into, at the screen's size.
struct Targets {
    colour: *mut SDL_GPUTexture,
    depth: *mut SDL_GPUTexture,
    canvas: *mut SDL_GPUTexture,
    canvas_transfer: *mut SDL_GPUTransferBuffer,
}

impl Default for Targets {
    fn default() -> Targets {
        Targets {
            colour: null_mut(),
            depth: null_mut(),
            canvas: null_mut(),
            canvas_transfer: null_mut(),
        }
    }
}

pub struct Gpu {
    device: *mut SDL_GPUDevice,
    colour: *mut SDL_GPUTexture,
    depth: *mut SDL_GPUTexture,
    canvas: *mut SDL_GPUTexture,
    canvas_transfer: *mut SDL_GPUTransferBuffer,
    width: u32,
    height: u32,
    /// How many times the targets have been made: names the colour target's set.
    targets_made: u64,
    depth_is_new: bool,
    scene_vert: *mut SDL_GPUShader,
    scene_frag: *mut SDL_GPUShader,
    overlay: *mut SDL_GPUGraphicsPipeline,
    pipelines: HashMap<PipelineKey, *mut SDL_GPUGraphicsPipeline>,
    /// By `linear as usize + 2 * clamp as usize`.
    samplers: [*mut SDL_GPUSampler; 4],
    nearest: *mut SDL_GPUSampler,
    white: *mut SDL_GPUTexture,
    textures: HashMap<crate::pointers::GamePtr, CachedTexture>,
    vertices: *mut SDL_GPUBuffer,
    vertices_room: u32,
    transfer: *mut SDL_GPUTransferBuffer,
    transfer_room: u32,
}

// Safety: used only under `GPU`'s lock, and SDL's device may be used from any one thread at a time.
unsafe impl Send for Gpu {}

/// The GPU, once asked for and made.
pub static GPU: Mutex<Option<Gpu>> = Mutex::new(None);

/// Whether a device is wanted: `RC2K_RENDERER=gpu`.
pub fn wanted() -> bool {
    std::env::var("RC2K_RENDERER").is_ok_and(|r| r == "gpu")
}

fn sdl_error() -> String {
    // SAFETY: SDL's own text, valid until its next call, copied at once.
    unsafe { std::ffi::CStr::from_ptr(SDL_GetError()) }
        .to_string_lossy()
        .into_owned()
}

/// A device that takes our shaders; `None` (logged) when the system has none.
pub fn make_device() -> Option<*mut SDL_GPUDevice> {
    // SAFETY: no pointers but SDL's error text.
    let device = unsafe { SDL_CreateGPUDevice(SDL_GPU_SHADERFORMAT_SPIRV, false, null()) };
    if device.is_null() {
        log(&format!("render: no GPU device: {}", sdl_error()));
        return None;
    }
    Some(device)
}

/// Starts the backend on `device`.
pub fn start(device: *mut SDL_GPUDevice) {
    match Gpu::new(device) {
        Ok(gpu) => {
            // SAFETY: the device's own constant text.
            let driver = unsafe { std::ffi::CStr::from_ptr(SDL_GetGPUDeviceDriver(device)) };
            log(&format!(
                "render: the GPU backend on {}",
                driver.to_string_lossy()
            ));
            *GPU.lock().unwrap() = Some(gpu);
            super::gpu_started();
        }
        Err(why) => log(&format!("render: the GPU backend does not start: {why}")),
    }
}

impl Gpu {
    pub fn new(device: *mut SDL_GPUDevice) -> Result<Gpu, String> {
        let mut gpu = Gpu {
            device,
            colour: null_mut(),
            depth: null_mut(),
            canvas: null_mut(),
            canvas_transfer: null_mut(),
            width: 0,
            height: 0,
            targets_made: 0,
            depth_is_new: true,
            scene_vert: null_mut(),
            scene_frag: null_mut(),
            overlay: null_mut(),
            pipelines: HashMap::new(),
            samplers: [null_mut(); 4],
            nearest: null_mut(),
            white: null_mut(),
            textures: HashMap::new(),
            vertices: null_mut(),
            vertices_room: 0,
            transfer: null_mut(),
            transfer_room: 0,
        };
        gpu.scene_vert = gpu.shader(SCENE_VERT, SDL_GPU_SHADERSTAGE_VERTEX, 0, 1)?;
        gpu.scene_frag = gpu.shader(SCENE_FRAG, SDL_GPU_SHADERSTAGE_FRAGMENT, 1, 1)?;
        let overlay_vert = gpu.shader(OVERLAY_VERT, SDL_GPU_SHADERSTAGE_VERTEX, 0, 0)?;
        let overlay_frag = gpu.shader(OVERLAY_FRAG, SDL_GPU_SHADERSTAGE_FRAGMENT, 1, 0)?;
        gpu.overlay = gpu.make_overlay_pipeline(overlay_vert, overlay_frag)?;
        for linear in [false, true] {
            for clamp in [false, true] {
                gpu.samplers[linear as usize + 2 * clamp as usize] =
                    gpu.make_sampler(linear, clamp)?;
            }
        }
        gpu.nearest = gpu.samplers[0];
        gpu.white = gpu.make_texture(1, 1, SDL_GPU_TEXTUREUSAGE_SAMPLER)?;
        gpu.upload_white()?;
        Ok(gpu)
    }

    fn shader(
        &self,
        code: &[u8],
        stage: SDL_GPUShaderStage,
        samplers: u32,
        uniforms: u32,
    ) -> Result<*mut SDL_GPUShader, String> {
        let info = SDL_GPUShaderCreateInfo {
            code_size: code.len(),
            code: code.as_ptr(),
            entrypoint: c"main".as_ptr(),
            format: SDL_GPU_SHADERFORMAT_SPIRV,
            stage,
            num_samplers: samplers,
            num_uniform_buffers: uniforms,
            ..Default::default()
        };
        // SAFETY: `info` and the code it points to live through the call.
        let shader = unsafe { SDL_CreateGPUShader(self.device, &raw const info) };
        (!shader.is_null()).then_some(shader).ok_or_else(sdl_error)
    }

    fn make_sampler(&self, linear: bool, clamp: bool) -> Result<*mut SDL_GPUSampler, String> {
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
            mipmap_mode: SDL_GPU_SAMPLERMIPMAPMODE_NEAREST,
            address_mode_u: address,
            address_mode_v: address,
            address_mode_w: address,
            ..Default::default()
        };
        // SAFETY: `info` lives through the call.
        let sampler = unsafe { SDL_CreateGPUSampler(self.device, &raw const info) };
        (!sampler.is_null())
            .then_some(sampler)
            .ok_or_else(sdl_error)
    }

    fn make_texture(
        &self,
        width: u32,
        height: u32,
        usage: SDL_GPUTextureUsageFlags,
    ) -> Result<*mut SDL_GPUTexture, String> {
        self.make_texture_of(COLOUR_FORMAT, width, height, usage)
    }

    fn make_texture_of(
        &self,
        format: SDL_GPUTextureFormat,
        width: u32,
        height: u32,
        usage: SDL_GPUTextureUsageFlags,
    ) -> Result<*mut SDL_GPUTexture, String> {
        let info = SDL_GPUTextureCreateInfo {
            r#type: SDL_GPU_TEXTURETYPE_2D,
            format,
            usage,
            width,
            height,
            layer_count_or_depth: 1,
            num_levels: 1,
            sample_count: SDL_GPU_SAMPLECOUNT_1,
            props: SDL_PropertiesID(0),
        };
        // SAFETY: `info` lives through the call.
        let texture = unsafe { SDL_CreateGPUTexture(self.device, &raw const info) };
        (!texture.is_null())
            .then_some(texture)
            .ok_or_else(sdl_error)
    }

    fn make_overlay_pipeline(
        &self,
        vertex_shader: *mut SDL_GPUShader,
        fragment_shader: *mut SDL_GPUShader,
    ) -> Result<*mut SDL_GPUGraphicsPipeline, String> {
        let colour = [SDL_GPUColorTargetDescription {
            format: COLOUR_FORMAT,
            ..Default::default()
        }];
        let info = SDL_GPUGraphicsPipelineCreateInfo {
            vertex_shader,
            fragment_shader,
            primitive_type: SDL_GPU_PRIMITIVETYPE_TRIANGLELIST,
            target_info: SDL_GPUGraphicsPipelineTargetInfo {
                color_target_descriptions: colour.as_ptr(),
                num_color_targets: 1,
                ..Default::default()
            },
            ..Default::default()
        };
        // SAFETY: `info` and `colour` live through the call.
        let pipeline = unsafe { SDL_CreateGPUGraphicsPipeline(self.device, &raw const info) };
        (!pipeline.is_null())
            .then_some(pipeline)
            .ok_or_else(sdl_error)
    }

    fn pipeline(&mut self, key: PipelineKey) -> Result<*mut SDL_GPUGraphicsPipeline, String> {
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
        let buffer = [SDL_GPUVertexBufferDescription {
            slot: 0,
            pitch: VERTEX_BYTES,
            input_rate: SDL_GPU_VERTEXINPUTRATE_VERTEX,
            instance_step_rate: 0,
        }];
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
        let colour = [SDL_GPUColorTargetDescription {
            format: COLOUR_FORMAT,
            blend_state: blend,
        }];
        let info = SDL_GPUGraphicsPipelineCreateInfo {
            vertex_shader: self.scene_vert,
            fragment_shader: self.scene_frag,
            vertex_input_state: SDL_GPUVertexInputState {
                vertex_buffer_descriptions: buffer.as_ptr(),
                num_vertex_buffers: 1,
                vertex_attributes: attributes.as_ptr(),
                num_vertex_attributes: attributes.len() as u32,
            },
            primitive_type: SDL_GPU_PRIMITIVETYPE_TRIANGLELIST,
            rasterizer_state: SDL_GPURasterizerState {
                fill_mode: SDL_GPU_FILLMODE_FILL,
                cull_mode: match key.cull {
                    3 => SDL_GPU_CULLMODE_BACK,
                    2 => SDL_GPU_CULLMODE_FRONT,
                    _ => SDL_GPU_CULLMODE_NONE,
                },
                // The triangles the game shows are clockwise on the screen.
                front_face: SDL_GPU_FRONTFACE_CLOCKWISE,
                enable_depth_clip: false,
                ..Default::default()
            },
            depth_stencil_state: SDL_GPUDepthStencilState {
                compare_op: compare_op(key.depth_function),
                enable_depth_test: true,
                enable_depth_write: key.depth_write,
                ..Default::default()
            },
            target_info: SDL_GPUGraphicsPipelineTargetInfo {
                color_target_descriptions: colour.as_ptr(),
                num_color_targets: 1,
                depth_stencil_format: DEPTH_FORMAT,
                has_depth_stencil_target: true,
                ..Default::default()
            },
            ..Default::default()
        };
        // SAFETY: `info` and the arrays it points to live through the call.
        let pipeline = unsafe { SDL_CreateGPUGraphicsPipeline(self.device, &raw const info) };
        if pipeline.is_null() {
            return Err(sdl_error());
        }
        self.pipelines.insert(key, pipeline);
        Ok(pipeline)
    }

    /// The targets at the screen's size. On a failure the old ones are kept, whole.
    pub fn set_size(&mut self, width: u32, height: u32) -> Result<(), String> {
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }
        let mut made = Targets::default();
        let outcome = self.make_targets(&mut made, width, height);
        if let Err(why) = outcome {
            self.release_targets(&made);
            return Err(why);
        }
        let old = Targets {
            colour: std::mem::replace(&mut self.colour, made.colour),
            depth: std::mem::replace(&mut self.depth, made.depth),
            canvas: std::mem::replace(&mut self.canvas, made.canvas),
            canvas_transfer: std::mem::replace(&mut self.canvas_transfer, made.canvas_transfer),
        };
        self.release_targets(&old);
        (self.width, self.height) = (width, height);
        self.targets_made += 1;
        self.depth_is_new = true;
        Ok(())
    }

    /// Fills `made` with targets of `width` x `height`, up to the first that fails.
    fn make_targets(&self, made: &mut Targets, width: u32, height: u32) -> Result<(), String> {
        made.colour = self.make_texture(
            width,
            height,
            SDL_GPU_TEXTUREUSAGE_COLOR_TARGET | SDL_GPU_TEXTUREUSAGE_SAMPLER,
        )?;
        made.depth = self.make_texture_of(
            DEPTH_FORMAT,
            width,
            height,
            SDL_GPU_TEXTUREUSAGE_DEPTH_STENCIL_TARGET,
        )?;
        made.canvas = self.make_texture(width, height, SDL_GPU_TEXTUREUSAGE_SAMPLER)?;
        made.canvas_transfer =
            self.make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD, width * height * 4)?;
        Ok(())
    }

    /// The targets released, those of them that were made.
    fn release_targets(&self, targets: &Targets) {
        // SAFETY: ours, no longer held by `self`; SDL waits for work that uses them.
        unsafe {
            for texture in [targets.colour, targets.depth, targets.canvas] {
                if !texture.is_null() {
                    SDL_ReleaseGPUTexture(self.device, texture);
                }
            }
            if !targets.canvas_transfer.is_null() {
                SDL_ReleaseGPUTransferBuffer(self.device, targets.canvas_transfer);
            }
        }
    }

    fn make_transfer(
        &self,
        usage: SDL_GPUTransferBufferUsage,
        size: u32,
    ) -> Result<*mut SDL_GPUTransferBuffer, String> {
        let info = SDL_GPUTransferBufferCreateInfo {
            usage,
            size,
            props: SDL_PropertiesID(0),
        };
        // SAFETY: `info` lives through the call.
        let transfer = unsafe { SDL_CreateGPUTransferBuffer(self.device, &raw const info) };
        (!transfer.is_null())
            .then_some(transfer)
            .ok_or_else(sdl_error)
    }

    /// The colour target, for showing it, and the number of its set of targets (a new
    /// target may be made at an old one's address).
    pub fn colour_texture(&self) -> (*mut SDL_GPUTexture, u64) {
        (self.colour, self.targets_made)
    }

    /// `bytes` into the transfer buffer `transfer` from `at` (cycled when `at` is 0: the last
    /// frame's may be in use).
    fn fill_transfer(
        &self,
        transfer: *mut SDL_GPUTransferBuffer,
        at: usize,
        bytes: &[u8],
    ) -> Result<(), String> {
        // SAFETY: the buffer holds at least `at + bytes.len()` bytes (the callers' sums), mapped
        // only here.
        unsafe {
            let map = SDL_MapGPUTransferBuffer(self.device, transfer, at == 0);
            if map.is_null() {
                return Err(sdl_error());
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), map.cast::<u8>().add(at), bytes.len());
            SDL_UnmapGPUTransferBuffer(self.device, transfer);
        }
        Ok(())
    }

    fn upload_white(&self) -> Result<(), String> {
        let transfer = self.make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD, 4)?;
        self.fill_transfer(transfer, 0, &[0xFF; 4])?;
        // SAFETY: the handles are ours; the command buffer is submitted at once.
        unsafe {
            let commands = SDL_AcquireGPUCommandBuffer(self.device);
            let copy = SDL_BeginGPUCopyPass(commands);
            upload(copy, transfer, 0, self.white, 1, 1);
            SDL_EndGPUCopyPass(copy);
            SDL_SubmitGPUCommandBuffer(commands);
            SDL_ReleaseGPUTransferBuffer(self.device, transfer);
        }
        Ok(())
    }

    /// Room for the frame's vertices and textures.
    fn ensure_room(&mut self, vertex_bytes: u32, transfer_bytes: u32) -> Result<(), String> {
        if vertex_bytes > self.vertices_room {
            let room = vertex_bytes.next_power_of_two().max(1 << 20);
            let info = SDL_GPUBufferCreateInfo {
                usage: SDL_GPU_BUFFERUSAGE_VERTEX,
                size: room,
                props: SDL_PropertiesID(0),
            };
            // SAFETY: `info` lives through the call; the old buffer is ours.
            unsafe {
                if !self.vertices.is_null() {
                    SDL_ReleaseGPUBuffer(self.device, self.vertices);
                }
                self.vertices = SDL_CreateGPUBuffer(self.device, &raw const info);
            }
            if self.vertices.is_null() {
                return Err(sdl_error());
            }
            self.vertices_room = room;
        }
        if transfer_bytes > self.transfer_room {
            let room = transfer_bytes.next_power_of_two().max(1 << 22);
            if !self.transfer.is_null() {
                // SAFETY: ours.
                unsafe { SDL_ReleaseGPUTransferBuffer(self.device, self.transfer) };
            }
            self.transfer = self.make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD, room)?;
            self.transfer_room = room;
        }
        Ok(())
    }

    /// Forgets what is known of textures whose memory has changed, and lets go of those whose
    /// memory is gone.
    fn take_changes(&mut self) {
        let changes = super::take_changed_textures();
        for pixels in changes.written {
            if let Some(texture) = self.textures.get_mut(&pixels) {
                texture.stale = true;
            }
        }
        for pixels in changes.gone {
            if let Some(gone) = self.textures.remove(&pixels) {
                // SAFETY: ours; SDL waits for work that uses it.
                unsafe { SDL_ReleaseGPUTexture(self.device, gone.texture) };
            }
        }
    }

    /// Draws the commands over the target, the canvas's written pixels first.
    pub fn draw(&mut self, canvas: &mut [u8], commands: &[Command]) -> Result<(), String> {
        self.take_changes();
        let batches = Batches::of(commands, self.width, self.height);
        let vertex_bytes = batches.vertices.len() as u32 * VERTEX_BYTES;
        let mut uploads: Vec<(TextureNote, Vec<u8>)> = Vec::new();
        for note in &batches.textures {
            let up_to_date = self
                .textures
                .get(&note.pixels)
                .is_some_and(|t| !t.stale && (t.width, t.height) == (note.width, note.height));
            if !up_to_date {
                uploads.push((*note, super::texture_as_bgra(note)));
            }
        }
        let upload_bytes: usize = uploads.iter().map(|(_, b)| b.len()).sum();
        self.ensure_room(
            vertex_bytes.max(1),
            (vertex_bytes as usize + upload_bytes).max(1) as u32,
        )?;
        for (note, _) in &uploads {
            let fits = self
                .textures
                .get(&note.pixels)
                .is_some_and(|t| (t.width, t.height) == (note.width, note.height));
            if !fits {
                let texture = self.make_texture(
                    note.width as u32,
                    note.height as u32,
                    SDL_GPU_TEXTUREUSAGE_SAMPLER,
                )?;
                if let Some(old) = self.textures.insert(
                    note.pixels,
                    CachedTexture {
                        texture,
                        width: note.width,
                        height: note.height,
                        stale: false,
                    },
                ) {
                    // SAFETY: ours; SDL waits for work that uses it.
                    unsafe { SDL_ReleaseGPUTexture(self.device, old.texture) };
                }
            }
        }
        // The pipelines first: making one in the render pass would keep `self` borrowed.
        let mut pipelines = Vec::with_capacity(batches.batches.len());
        for batch in &batches.batches {
            pipelines.push(self.pipeline(batch.pipeline)?);
        }
        let vertex_data: &[u8] = bytes_of_vertices(&batches.vertices);
        self.fill_transfer(self.transfer, 0, vertex_data)?;
        let mut at = vertex_data.len();
        let mut placed = Vec::with_capacity(uploads.len());
        for (note, bytes) in &uploads {
            self.fill_transfer(self.transfer, at, bytes)?;
            placed.push((note, at as u32));
            at += bytes.len();
        }
        self.fill_transfer(self.canvas_transfer, 0, canvas)?;
        // SAFETY: every handle is ours and alive; the offsets are within the buffers (the sums
        // above); the bound arrays live through their calls; the command buffer is submitted
        // before the end of this block.
        unsafe {
            let buffer = SDL_AcquireGPUCommandBuffer(self.device);
            if buffer.is_null() {
                return Err(sdl_error());
            }
            let copy = SDL_BeginGPUCopyPass(buffer);
            let from = SDL_GPUTransferBufferLocation {
                transfer_buffer: self.transfer,
                offset: 0,
            };
            let to = SDL_GPUBufferRegion {
                buffer: self.vertices,
                offset: 0,
                size: vertex_bytes,
            };
            if vertex_bytes > 0 {
                SDL_UploadToGPUBuffer(copy, &raw const from, &raw const to, true);
            }
            for (note, offset) in &placed {
                let texture = self.textures.get_mut(&note.pixels).expect("made above");
                upload(
                    copy,
                    self.transfer,
                    *offset,
                    texture.texture,
                    note.width as u32,
                    note.height as u32,
                );
                texture.stale = false;
            }
            upload(
                copy,
                self.canvas_transfer,
                0,
                self.canvas,
                self.width,
                self.height,
            );
            SDL_EndGPUCopyPass(copy);
            let colour = SDL_GPUColorTargetInfo {
                texture: self.colour,
                load_op: SDL_GPU_LOADOP_LOAD,
                store_op: SDL_GPU_STOREOP_STORE,
                ..Default::default()
            };
            let depth = SDL_GPUDepthStencilTargetInfo {
                texture: self.depth,
                clear_depth: 1.0,
                load_op: if self.depth_is_new {
                    SDL_GPU_LOADOP_CLEAR
                } else {
                    SDL_GPU_LOADOP_LOAD
                },
                store_op: SDL_GPU_STOREOP_STORE,
                stencil_load_op: SDL_GPU_LOADOP_DONT_CARE,
                stencil_store_op: SDL_GPU_STOREOP_DONT_CARE,
                ..Default::default()
            };
            self.depth_is_new = false;
            let pass = SDL_BeginGPURenderPass(buffer, &raw const colour, 1, &raw const depth);
            self.draw_overlay(pass);
            let size = [self.width as f32, self.height as f32];
            SDL_PushGPUVertexUniformData(buffer, 0, size.as_ptr().cast(), 8);
            let binding = SDL_GPUBufferBinding {
                buffer: self.vertices,
                offset: 0,
            };
            SDL_BindGPUVertexBuffers(pass, 0, &raw const binding, 1);
            let mut last_pipeline = null_mut();
            let mut last_values = None;
            let mut last_sampling = None;
            let mut scissored = false;
            for (batch, &pipeline) in batches.batches.iter().zip(&pipelines) {
                if pipeline != last_pipeline {
                    SDL_BindGPUGraphicsPipeline(pass, pipeline);
                    last_pipeline = pipeline;
                }
                if last_values != Some(batch.values) {
                    SDL_PushGPUFragmentUniformData(
                        buffer,
                        0,
                        (&raw const batch.values).cast(),
                        size_of_val(&batch.values) as u32,
                    );
                    last_values = Some(batch.values);
                }
                let sampling = (batch.texture, batch.linear, batch.clamp);
                if last_sampling != Some(sampling) {
                    let texture = batch.texture.map_or(self.white, |n| {
                        self.textures[&batches.textures[n as usize].pixels].texture
                    });
                    let bound = SDL_GPUTextureSamplerBinding {
                        texture,
                        sampler: self.samplers[sampler_number(batch)],
                    };
                    SDL_BindGPUFragmentSamplers(pass, 0, &raw const bound, 1);
                    last_sampling = Some(sampling);
                }
                if let Some([left, top, right, bottom]) = batch.scissor {
                    let rect = SDL_Rect {
                        x: left as i32,
                        y: top as i32,
                        w: (right - left) as i32,
                        h: (bottom - top) as i32,
                    };
                    SDL_SetGPUScissor(pass, &raw const rect);
                    scissored = true;
                } else if scissored {
                    let rect = SDL_Rect {
                        x: 0,
                        y: 0,
                        w: self.width as i32,
                        h: self.height as i32,
                    };
                    SDL_SetGPUScissor(pass, &raw const rect);
                    scissored = false;
                }
                SDL_DrawGPUPrimitives(pass, batch.vertices, 1, batch.first_vertex, 0);
            }
            SDL_EndGPURenderPass(pass);
            if !SDL_SubmitGPUCommandBuffer(buffer) {
                return Err(sdl_error());
            }
        }
        mark_unwritten(canvas);
        Ok(())
    }

    /// The canvas's written pixels over the target (bound in an open pass).
    ///
    /// # Safety
    /// `pass` is an open render pass on the target.
    unsafe fn draw_overlay(&self, pass: *mut SDL_GPURenderPass) {
        let bound = SDL_GPUTextureSamplerBinding {
            texture: self.canvas,
            sampler: self.nearest,
        };
        // SAFETY: the caller's pass; handles ours; `bound` lives through the call.
        unsafe {
            SDL_BindGPUGraphicsPipeline(pass, self.overlay);
            SDL_BindGPUFragmentSamplers(pass, 0, &raw const bound, 1);
            SDL_DrawGPUPrimitives(pass, 3, 1, 0, 0);
        }
    }

    /// The frame's end: what the canvas has written goes over the target.
    pub fn finish_frame(&mut self, canvas: &mut [u8]) -> Result<(), String> {
        self.draw(canvas, &[])
    }

    /// The target's picture, with the canvas's written pixels, into the canvas.
    pub fn bring_back(&mut self, canvas: &mut [u8]) -> Result<(), String> {
        self.finish_frame(canvas)?;
        let bytes = self.width * self.height * 4;
        let transfer = self.make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_DOWNLOAD, bytes)?;
        // SAFETY: handles ours; the wait ends the GPU's use of the buffer before it is read, and
        // the copy is of the `bytes` the buffer holds, which `canvas` holds too (`Target::bytes`).
        unsafe {
            let buffer = SDL_AcquireGPUCommandBuffer(self.device);
            let copy = SDL_BeginGPUCopyPass(buffer);
            let region = SDL_GPUTextureRegion {
                texture: self.colour,
                w: self.width,
                h: self.height,
                d: 1,
                ..Default::default()
            };
            let to = SDL_GPUTextureTransferInfo {
                transfer_buffer: transfer,
                ..Default::default()
            };
            SDL_DownloadFromGPUTexture(copy, &raw const region, &raw const to);
            SDL_EndGPUCopyPass(copy);
            let fence = SDL_SubmitGPUCommandBufferAndAcquireFence(buffer);
            if fence.is_null() {
                return Err(sdl_error());
            }
            SDL_WaitForGPUFences(self.device, true, &raw const fence, 1);
            SDL_ReleaseGPUFence(self.device, fence);
            let map = SDL_MapGPUTransferBuffer(self.device, transfer, false);
            if map.is_null() {
                return Err(sdl_error());
            }
            std::ptr::copy_nonoverlapping(map.cast::<u8>(), canvas.as_mut_ptr(), bytes as usize);
            SDL_UnmapGPUTransferBuffer(self.device, transfer);
            SDL_ReleaseGPUTransferBuffer(self.device, transfer);
        }
        Ok(())
    }
}

/// A copy pass's upload of a whole picture.
///
/// # Safety
/// `copy` is an open copy pass, and the transfer buffer holds the picture at `offset`.
unsafe fn upload(
    copy: *mut SDL_GPUCopyPass,
    transfer: *mut SDL_GPUTransferBuffer,
    offset: u32,
    texture: *mut SDL_GPUTexture,
    width: u32,
    height: u32,
) {
    let from = SDL_GPUTextureTransferInfo {
        transfer_buffer: transfer,
        offset,
        pixels_per_row: width,
        rows_per_layer: height,
    };
    let to = SDL_GPUTextureRegion {
        texture,
        w: width,
        h: height,
        d: 1,
        ..Default::default()
    };
    // SAFETY: the caller's pass and buffer; the two records live through the call.
    unsafe { SDL_UploadToGPUTexture(copy, &raw const from, &raw const to, true) };
}

fn sampler_number(batch: &Batch) -> usize {
    batch.linear as usize + 2 * batch.clamp as usize
}

fn bytes_of_vertices(vertices: &[super::batches::Vertex]) -> &[u8] {
    // SAFETY: dwords as bytes: same memory, a smaller alignment, every value valid.
    unsafe { std::slice::from_raw_parts(vertices.as_ptr().cast(), size_of_val(vertices)) }
}

/// Every pixel of the canvas marked as not written.
pub fn mark_unwritten(canvas: &mut [u8]) {
    for pixel in canvas.as_chunks_mut::<4>().0 {
        *pixel = UNWRITTEN.to_le_bytes();
    }
}

fn blend_factor(factor: u32) -> SDL_GPUBlendFactor {
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

fn compare_op(function: u32) -> SDL_GPUCompareOp {
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

impl super::Backend for Gpu {
    fn draw(&mut self, target: &Target, commands: Vec<Command>) {
        // SAFETY: the game's code waits for this call; nothing else writes the canvas meanwhile.
        let canvas = unsafe { target.bytes() };
        let done = self
            .set_size(target.width as u32, target.height as u32)
            .and_then(|()| Gpu::draw(self, canvas, &commands));
        if let Err(why) = done {
            crate::fatal::raise::end(
                "render::gpu",
                crate::errors::BadInput(Box::leak(why.into_boxed_str())),
            );
        }
    }
}

#[cfg(test)]
mod comparison {
    //! One recorded frame (`RC2K_DUMP`, the file named by `RC2K_COMPARE_FRAME`) through the
    //! rasteriser and the GPU, and how far the pictures differ. Skips without the file (it holds
    //! the game's textures, so it is not in the repository) or a GPU.

    use super::*;
    use crate::render::STATES;

    struct Reader<'a>(&'a [u8]);
    impl Reader<'_> {
        fn dword(&mut self) -> u32 {
            let (head, rest) = self.0.split_first_chunk::<4>().unwrap();
            self.0 = rest;
            u32::from_le_bytes(*head)
        }
    }

    #[test]
    fn a_recorded_frame_looks_the_same() {
        let Some(path) = std::env::var_os("RC2K_COMPARE_FRAME") else {
            println!("SKIPPED: no RC2K_COMPARE_FRAME");
            return;
        };
        let file = std::fs::read(path).unwrap();
        assert_eq!(&file[..8], b"RC2KDUMP");
        let mut r = Reader(&file[8..]);
        let (width, height, bits) = (r.dword() as usize, r.dword() as usize, r.dword());
        assert_eq!(bits, 32);
        let mut bytes = Vec::new();
        let mut notes = Vec::new();
        for _ in 0..r.dword() {
            let [w, h, pitch, each] = [0; 4].map(|_: u8| r.dword() as usize);
            let masks = [0; 4].map(|_: u8| r.dword());
            let len = r.dword() as usize;
            let (data, rest) = r.0.split_at(len);
            r.0 = rest;
            bytes.push(data.to_vec());
            notes.push((w, h, pitch, each, masks, len));
        }
        let lent: Vec<_> = bytes
            .iter()
            .map(|b| crate::pointers::Lent::new(b))
            .collect();
        let note = |n: usize| {
            let (width, height, pitch, bytes, masks, len) = notes[n];
            TextureNote {
                pixels: crate::pointers::GamePtr::new(lent[n].addr()).limited(len as u32),
                len,
                width,
                height,
                pitch,
                bytes,
                masks,
            }
        };
        let read_commands = |r: &mut Reader| {
            let mut commands = Vec::new();
            for _ in 0..r.dword() {
                if r.dword() == 1 {
                    let corners = r.dword() as usize;
                    let vertices: Vec<[u32; 8]> = (0..corners)
                        .map(|_| [0; 8].map(|_: u8| r.dword()))
                        .collect();
                    let mut states = Box::new([0; STATES]);
                    for s in states.iter_mut() {
                        *s = r.dword();
                    }
                    let number = r.dword();
                    commands.push(Command::Fan {
                        vertices: vertices.into(),
                        states: states.into(),
                        texture: (number != u32::MAX).then(|| note(number as usize)),
                        bounds: (f32::MIN, f32::MIN, f32::MAX, f32::MAX),
                    });
                } else {
                    commands.push(Command::Clear([0; 4].map(|_: u8| r.dword() as usize)));
                }
            }
            commands
        };
        let commands = read_commands(&mut Reader(r.0));

        crate::session::SCREEN.width.set(width as u32);
        crate::session::SCREEN.height.set(height as u32);
        let soft = vec![0u8; width * height * 4];
        let soft_lent = crate::pointers::Lent::new(&soft);
        let target = Target {
            pixels: soft_lent.addr() as usize,
            pitch: width * 4,
            width,
            height,
            bits,
        };
        crate::render::Backend::draw(
            &mut crate::render::software::Software,
            &target,
            read_commands(&mut Reader(r.0)),
        );

        // SAFETY: no pointers; SDL's start-up, on this test's thread.
        unsafe { SDL_Init(SDL_INIT_VIDEO) };
        let Some(device) = make_device() else {
            println!("SKIPPED: no GPU device: {}", sdl_error());
            return;
        };
        let mut gpu = Gpu::new(device).unwrap();
        gpu.set_size(width as u32, height as u32).unwrap();
        let mut canvas = vec![0u8; width * height * 4];
        Gpu::draw(&mut gpu, &mut canvas, &commands).unwrap();
        gpu.bring_back(&mut canvas).unwrap();

        let (mut far, mut total) = (0usize, 0u64);
        for (a, b) in soft
            .as_chunks::<4>()
            .0
            .iter()
            .zip(canvas.as_chunks::<4>().0)
        {
            let most = (0..3).map(|n| a[n].abs_diff(b[n])).max().unwrap();
            total += u64::from(most);
            if most > 32 {
                far += 1;
            }
        }
        let pixels = width * height;
        println!(
            "{width} x {height}, {} commands: {:.2}% of pixels differ by more than 32, mean difference {:.2}",
            commands.len(),
            far as f64 * 100.0 / pixels as f64,
            total as f64 / pixels as f64
        );
        if let Some(out) = std::env::var_os("RC2K_COMPARE_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::write(out.join("software.raw"), &soft).unwrap();
            std::fs::write(out.join("gpu.raw"), &canvas).unwrap();
        }
        assert!(far * 20 < pixels, "over 5% of the pixels differ a lot");
    }
}
