//! The GPU backend, on SDL's GPU API (Vulkan on the PC, Metal on macOS). The calls into SDL are
//! `ffi::sdl_gpu`'s: each handle here was made by the same device and lives as long as `Gpu`,
//! or until the line that replaces it.
//!
//! A race's frame is 3D in the GPU's colour target and the game's 2D on the screen's memory (the
//! canvas), which starts each layer holding `UNWRITTEN` in every pixel. When the two meet, the
//! pixels the game has written go over the target and the canvas is marked again. Something that
//! wants the true picture in the canvas asks `bring_back`, which is slow on a handheld.

use super::batches::{Batch, Batches, PipelineKey};
use super::{CommandList, Target, TextureNote};
use crate::ffi::sdl_gpu::{
    self, Buffer, Device, Pipeline, PipelineSpec, Sampler, Shader, ShaderCode, Texture, Transfer,
};
use crate::framework::log;
#[cfg(test)]
use crate::system::owner::system;
use sdl3_sys::everything::*;
use std::collections::HashMap;
use std::sync::Mutex;

/// A canvas pixel the game has not written: magenta, with an alpha byte the screen's pixels are
/// not written with (`overlay.frag`).
pub const UNWRITTEN: u32 = 0xA5FF_00FF;

/// A shader's code from `shaders/`: its SPIR-V, and on macOS its MSL.
macro_rules! shader_code {
    ($name:literal) => {
        ShaderCode {
            spirv: include_bytes!(concat!("shaders/", $name, ".spv")),
            #[cfg(target_os = "macos")]
            msl: include_bytes!(concat!("shaders/", $name, ".msl")),
        }
    };
}

const SCENE_VERT: ShaderCode = shader_code!("scene.vert");
const SCENE_FRAG: ShaderCode = shader_code!("scene.frag");
const OVERLAY_VERT: ShaderCode = shader_code!("overlay.vert");
const OVERLAY_FRAG: ShaderCode = shader_code!("overlay.frag");

const COLOUR_FORMAT: SDL_GPUTextureFormat = SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM;
const DEPTH_FORMAT: SDL_GPUTextureFormat = SDL_GPU_TEXTUREFORMAT_D32_FLOAT;
const VERTEX_BYTES: u32 = 32;

struct CachedTexture {
    texture: Texture,
    width: usize,
    height: usize,
    stale: bool,
}

/// The targets a frame is drawn into, at the screen's size.
struct Targets {
    colour: Texture,
    depth: Texture,
    canvas: Texture,
    canvas_transfer: Transfer,
}

impl Default for Targets {
    fn default() -> Targets {
        Targets {
            colour: Texture::NONE,
            depth: Texture::NONE,
            canvas: Texture::NONE,
            canvas_transfer: Transfer::NONE,
        }
    }
}

pub struct Gpu {
    device: Device,
    colour: Texture,
    depth: Texture,
    canvas: Texture,
    canvas_transfer: Transfer,
    width: u32,
    height: u32,
    /// How many times the targets have been made: names the colour target's set.
    targets_made: u64,
    depth_is_new: bool,
    scene_vert: Shader,
    scene_frag: Shader,
    overlay: Pipeline,
    pipelines: HashMap<PipelineKey, Pipeline>,
    /// By `linear as usize + 2 * clamp as usize`.
    samplers: [Sampler; 4],
    nearest: Sampler,
    white: Texture,
    textures: super::TextureMap<CachedTexture>,
    vertices: Buffer,
    vertices_room: u32,
    transfer: Transfer,
    transfer_room: u32,
}

/// The GPU, once asked for and made.
pub static GPU: Mutex<Option<Gpu>> = Mutex::new(None);

/// Whether a device is wanted: `RC2K_RENDERER=gpu`.
pub fn wanted() -> bool {
    std::env::var("RC2K_RENDERER").is_ok_and(|r| r == "gpu")
}

fn sdl_error() -> String {
    sdl_gpu::error()
}

/// A device that takes our shaders; `None` (logged) when the system has none.
pub fn make_device() -> Option<Device> {
    let device = Device::create();
    if device.is_none() {
        log(&format!("render: no GPU device: {}", sdl_error()));
    }
    device
}

/// Starts the backend on `device`.
pub fn start(device: Device) {
    match Gpu::new(device) {
        Ok(gpu) => {
            log(&format!("render: the GPU backend on {}", device.driver()));
            *GPU.lock().unwrap() = Some(gpu);
            super::gpu_started();
        }
        Err(why) => log(&format!("render: the GPU backend does not start: {why}")),
    }
}

impl Gpu {
    pub fn new(device: Device) -> Result<Gpu, String> {
        let mut gpu = Gpu {
            device,
            colour: Texture::NONE,
            depth: Texture::NONE,
            canvas: Texture::NONE,
            canvas_transfer: Transfer::NONE,
            width: 0,
            height: 0,
            targets_made: 0,
            depth_is_new: true,
            scene_vert: Shader::NONE,
            scene_frag: Shader::NONE,
            overlay: Pipeline::NONE,
            pipelines: HashMap::new(),
            samplers: [Sampler::NONE; 4],
            nearest: Sampler::NONE,
            white: Texture::NONE,
            textures: Default::default(),
            vertices: Buffer::NONE,
            vertices_room: 0,
            transfer: Transfer::NONE,
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
        code: ShaderCode,
        stage: SDL_GPUShaderStage,
        samplers: u32,
        uniforms: u32,
    ) -> Result<Shader, String> {
        self.device.create_shader(code, stage, samplers, uniforms)
    }

    fn make_sampler(&self, linear: bool, clamp: bool) -> Result<Sampler, String> {
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
        self.device.create_sampler(&info)
    }

    fn make_texture(
        &self,
        width: u32,
        height: u32,
        usage: SDL_GPUTextureUsageFlags,
    ) -> Result<Texture, String> {
        self.make_texture_of(COLOUR_FORMAT, width, height, usage)
    }

    fn make_texture_of(
        &self,
        format: SDL_GPUTextureFormat,
        width: u32,
        height: u32,
        usage: SDL_GPUTextureUsageFlags,
    ) -> Result<Texture, String> {
        self.device.create_texture(format, width, height, usage)
    }

    fn make_overlay_pipeline(
        &self,
        vertex_shader: Shader,
        fragment_shader: Shader,
    ) -> Result<Pipeline, String> {
        let spec = PipelineSpec {
            vertices: None,
            colour: SDL_GPUColorTargetDescription {
                format: COLOUR_FORMAT,
                ..Default::default()
            },
            rasterizer: SDL_GPURasterizerState::default(),
            depth: None,
        };
        self.device
            .create_pipeline(vertex_shader, fragment_shader, &spec)
    }

    fn pipeline(&mut self, key: PipelineKey) -> Result<Pipeline, String> {
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
                enable_depth_clip: false,
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
        for texture in [targets.colour, targets.depth, targets.canvas] {
            if !texture.is_none() {
                self.device.release_texture(texture);
            }
        }
        if !targets.canvas_transfer.is_none() {
            self.device.release_transfer(targets.canvas_transfer);
        }
    }

    fn make_transfer(
        &self,
        usage: SDL_GPUTransferBufferUsage,
        size: u32,
    ) -> Result<Transfer, String> {
        self.device.create_transfer(usage, size)
    }

    /// The colour target, for showing it, and the number of its set of targets (a new
    /// target may be made at an old one's address).
    pub fn colour_texture(&self) -> (*mut SDL_GPUTexture, u64) {
        (self.colour.as_raw(), self.targets_made)
    }

    /// `bytes` into the transfer buffer `transfer` from `at` (cycled when `at` is 0: the last
    /// frame's may be in use).
    fn fill_transfer(&self, transfer: Transfer, at: usize, bytes: &[u8]) -> Result<(), String> {
        self.device.fill_transfer(transfer, at, bytes, at == 0)
    }

    fn upload_white(&self) -> Result<(), String> {
        let transfer = self.make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD, 4)?;
        self.fill_transfer(transfer, 0, &[0xFF; 4])?;
        // The command buffer is submitted at once.
        let commands = self.device.acquire_command_buffer()?;
        let copy = commands.begin_copy_pass();
        copy.upload_texture(transfer, 0, self.white, 1, 1);
        copy.end();
        let _ = commands.submit();
        self.device.release_transfer(transfer);
        Ok(())
    }

    /// Room for the frame's vertices and textures.
    fn ensure_room(&mut self, vertex_bytes: u32, transfer_bytes: u32) -> Result<(), String> {
        if vertex_bytes > self.vertices_room {
            let room = vertex_bytes.next_power_of_two().max(1 << 20);
            if !self.vertices.is_none() {
                self.device.release_buffer(self.vertices);
            }
            self.vertices = Buffer::NONE;
            self.vertices = self.device.create_vertex_buffer(room)?;
            self.vertices_room = room;
        }
        if transfer_bytes > self.transfer_room {
            let room = transfer_bytes.next_power_of_two().max(1 << 22);
            if !self.transfer.is_none() {
                self.device.release_transfer(self.transfer);
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
                self.device.release_texture(gone.texture);
            }
        }
    }

    /// Draws the commands over the target, the canvas's written pixels first.
    pub fn draw(&mut self, canvas: &mut [u8], commands: &CommandList) -> Result<(), String> {
        self.take_changes();
        let batches = Batches::of(commands, self.width, self.height, None);
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
                    self.device.release_texture(old.texture);
                }
            }
        }
        // The pipelines first: making one in the render pass would keep `self` borrowed.
        let mut pipelines = Vec::with_capacity(batches.batches.len());
        for batch in &batches.batches {
            pipelines.push(self.pipeline(batch.pipeline)?);
        }
        let vertex_data: &[u8] = sdl_gpu::words_as_bytes(&batches.vertices);
        self.fill_transfer(self.transfer, 0, vertex_data)?;
        let mut at = vertex_data.len();
        let mut placed = Vec::with_capacity(uploads.len());
        for (note, bytes) in &uploads {
            self.fill_transfer(self.transfer, at, bytes)?;
            placed.push((note, at as u32));
            at += bytes.len();
        }
        self.fill_transfer(self.canvas_transfer, 0, canvas)?;
        // Every handle is ours and alive; the offsets are within the buffers (the sums above);
        // the command buffer is submitted before the end of this block.
        let buffer = self.device.acquire_command_buffer()?;
        let copy = buffer.begin_copy_pass();
        if vertex_bytes > 0 {
            copy.upload_buffer(self.transfer, self.vertices, vertex_bytes);
        }
        for (note, offset) in &placed {
            let texture = self.textures.get_mut(&note.pixels).expect("made above");
            copy.upload_texture(
                self.transfer,
                *offset,
                texture.texture,
                note.width as u32,
                note.height as u32,
            );
            texture.stale = false;
        }
        copy.upload_texture(
            self.canvas_transfer,
            0,
            self.canvas,
            self.width,
            self.height,
        );
        copy.end();
        let pass = buffer.begin_render_pass(self.colour, self.depth, self.depth_is_new);
        self.depth_is_new = false;
        self.draw_overlay(&pass);
        buffer.push_vertex_uniform(0, &[self.width as f32, self.height as f32]);
        pass.bind_vertex_buffer(self.vertices);
        let mut last_pipeline = None;
        let mut last_values = None;
        let mut last_sampling = None;
        let mut scissored = false;
        for (batch, &pipeline) in batches.batches.iter().zip(&pipelines) {
            if last_pipeline != Some(pipeline) {
                pass.bind_pipeline(pipeline);
                last_pipeline = Some(pipeline);
            }
            if last_values != Some(batch.values) {
                buffer.push_fragment_uniform(0, &batch.values.to_words());
                last_values = Some(batch.values);
            }
            let sampling = (batch.texture, batch.linear, batch.clamp);
            if last_sampling != Some(sampling) {
                let texture = batch.texture.map_or(self.white, |n| {
                    self.textures[&batches.textures[n as usize].pixels].texture
                });
                pass.bind_fragment_sampler(texture, self.samplers[sampler_number(batch)]);
                last_sampling = Some(sampling);
            }
            if let Some([left, top, right, bottom]) = batch.scissor {
                pass.set_scissor(
                    left as i32,
                    top as i32,
                    (right - left) as i32,
                    (bottom - top) as i32,
                );
                scissored = true;
            } else if scissored {
                pass.set_scissor(0, 0, self.width as i32, self.height as i32);
                scissored = false;
            }
            pass.draw(batch.vertices, batch.first_vertex);
        }
        pass.end();
        buffer.submit()?;
        mark_unwritten(canvas);
        Ok(())
    }

    /// The canvas's written pixels over the target (bound in an open pass).
    fn draw_overlay(&self, pass: &sdl_gpu::RenderPass) {
        pass.bind_pipeline(self.overlay);
        pass.bind_fragment_sampler(self.canvas, self.nearest);
        pass.draw(3, 0);
    }

    /// The frame's end: what the canvas has written goes over the target.
    pub fn finish_frame(&mut self, canvas: &mut [u8]) -> Result<(), String> {
        self.draw(canvas, &CommandList::new())
    }

    /// The target's picture, with the canvas's written pixels, into the canvas.
    pub fn bring_back(&mut self, canvas: &mut [u8]) -> Result<(), String> {
        self.finish_frame(canvas)?;
        let bytes = self.width * self.height * 4;
        let transfer = self.make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_DOWNLOAD, bytes)?;
        // The wait ends the GPU's use of the buffer before it is read; the copy is of the
        // `bytes` the buffer holds, which `canvas` holds too (`Target::bytes`).
        let buffer = self.device.acquire_command_buffer()?;
        let copy = buffer.begin_copy_pass();
        copy.download_texture(self.colour, self.width, self.height, transfer);
        copy.end();
        buffer.submit_and_wait(self.device)?;
        self.device
            .read_transfer(transfer, &mut canvas[..bytes as usize])?;
        self.device.release_transfer(transfer);
        Ok(())
    }
}

fn sampler_number(batch: &Batch) -> usize {
    batch.linear as usize + 2 * batch.clamp as usize
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
    fn draw(&mut self, target: &Target, commands: CommandList) {
        let done = target.with_canvas(|canvas| {
            self.set_size(target.width as u32, target.height as u32)
                .and_then(|()| Gpu::draw(self, canvas, &commands))
        });
        super::give_back(commands);
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
        let system = system();
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
        let levels: Vec<_> = bytes
            .iter()
            .map(|b| crate::platform_3d::test_level(b.clone()))
            .collect();
        let note = |n: usize| {
            let (width, height, pitch, bytes, masks, len) = notes[n];
            TextureNote {
                pixels: levels[n],
                len,
                width,
                height,
                pitch,
                bytes,
                masks,
            }
        };
        let read_commands = |r: &mut Reader| {
            let mut commands = CommandList::new();
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
                    commands.push_fan(
                        &vertices,
                        states.into(),
                        (number != u32::MAX).then(|| note(number as usize)),
                        (f32::MIN, f32::MIN, f32::MAX, f32::MAX),
                    );
                } else {
                    commands.push_clear([0; 4].map(|_: u8| r.dword() as usize));
                }
            }
            commands
        };
        let commands = read_commands(&mut Reader(r.0));

        system.session.screen.get().width.set(width as u32);
        system.session.screen.get().height.set(height as u32);
        system.session.screen.get().bits.set(bits);
        let soft_held =
            std::sync::Arc::new(crate::pool::PoolBytes::from_vec(vec![
                0u8;
                width * height * 4
            ]));
        let soft_handle = 0x7000_0000;
        crate::gfx::locked_pixels::register(soft_handle, &soft_held);
        let target = Target {
            pixels: soft_handle,
            pitch: width * 4,
            width,
            height,
            bits,
        };
        crate::render::Backend::draw(
            &mut crate::render::software::Software {
                screen: (width, height),
            },
            &target,
            read_commands(&mut Reader(r.0)),
        );

        let soft = soft_held.to_vec();
        sdl_gpu::init_video();
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
