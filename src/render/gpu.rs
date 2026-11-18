//! The GPU backend, on SDL's GPU API (Vulkan on the PC, Metal on macOS). The calls into SDL are
//! `ffi::sdl_gpu`'s: each handle here was made by the same device and lives as long as `Gpu`,
//! or until the line that replaces it.
//!
//! A race's frame is 3D in the GPU's colour target and the game's 2D on the screen's memory (the
//! canvas), which starts each layer holding `UNWRITTEN` in every pixel. When the two meet, the
//! rows the game has written (`canvas_rows`) are uploaded and their pixels go over the target,
//! and the canvas is marked again. A menu's background picture is not on the canvas: it is kept
//! as a texture of its own (`background`) and drawn under the canvas's pixels. Something that
//! wants the true picture in the canvas asks `bring_back`, which is slow on a handheld.

mod check;
mod quads;

use super::batches::{Batch, Batches, PipelineKey};
use super::gles::mip_levels;
use super::gpu_cut::Cuts;
use super::{CommandList, Target, TextureNote};
use crate::logfile::program_log::log;
use crate::platform::ffi::sdl_gpu::{
    self, Buffer, Device, Pipeline, PipelineSpec, Sampler, Shader, ShaderCode, Texture, Transfer,
};
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
            #[cfg(target_vendor = "apple")]
            msl: include_bytes!(concat!("shaders/", $name, ".msl")),
        }
    };
}

const SCENE_VERT: ShaderCode = shader_code!("scene.vert");
const SCENE_FRAG: ShaderCode = shader_code!("scene.frag");
const OVERLAY_VERT: ShaderCode = shader_code!("overlay.vert");
const OVERLAY_FRAG: ShaderCode = shader_code!("overlay.frag");
const MERGE_FRAG: ShaderCode = shader_code!("merge.frag");

const COLOUR_FORMAT: SDL_GPUTextureFormat = SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM;
const DEPTH_FORMAT: SDL_GPUTextureFormat = SDL_GPU_TEXTUREFORMAT_D32_FLOAT;
const VERTEX_BYTES: u32 = 32;

struct CachedTexture {
    texture: Texture,
    width: usize,
    height: usize,
    stale: bool,
    /// The levels it was made with (1 until a batch asks for mipmapping).
    levels: u32,
}

/// The targets a frame is drawn into, at the screen's size.
struct Targets {
    colour: Texture,
    depth: Texture,
    canvas: Texture,
    canvas_transfer: Transfer,
    background: Texture,
    background_transfer: Transfer,
}

impl Default for Targets {
    fn default() -> Targets {
        Targets {
            colour: Texture::NONE,
            depth: Texture::NONE,
            canvas: Texture::NONE,
            canvas_transfer: Transfer::NONE,
            background: Texture::NONE,
            background_transfer: Transfer::NONE,
        }
    }
}

/// The targets of a 3D drawn at another size than the screen's.
struct Small {
    colour: Texture,
    depth: Texture,
    /// The size they were made at.
    size: (u32, u32),
    depth_is_new: bool,
}

/// The targets of a menu's 3D drawn larger than the menu's frame, so that it is sharp on a window
/// larger than the frame (`render::menu_high_percent`).
struct MenuHigh {
    /// The picture of the frame with the 3D drawn so far, one of them being drawn into while the
    /// other is read (`Gpu::frame_high`).
    colour: [Texture; 2],
    /// The finished merge of the frame's end, which is what the window shows.
    shown: Texture,
    depth: Texture,
    /// The frame's colour as it was right after the 3D was drawn into it (`merge.frag`).
    kept: Texture,
    /// The size they were made at.
    size: (u32, u32),
    depth_is_new: bool,
}

pub struct Gpu {
    device: Device,
    colour: Texture,
    depth: Texture,
    canvas: Texture,
    canvas_transfer: Transfer,
    /// The retained background (`background`) and its upload buffer.
    background: Texture,
    background_transfer: Transfer,
    /// What `background` holds.
    background_holds: Option<super::background::Held>,
    /// With `RC2K_BACKGROUND_CHECK`: the frame drawn a second time as the canvas carried the
    /// background (`check`).
    check: Option<check::Check>,
    /// The 2D quads' atlas and buffers (`quads`), made when first needed.
    quads: Option<quads::Quads>,
    width: u32,
    height: u32,
    /// How many times the targets have been made: names the colour target's set.
    targets_made: u64,
    depth_is_new: bool,
    /// The colour and depth the 3D is drawn into when it is drawn smaller than the screen
    /// (`render_scale`), the screen's size with the picture in their top-left corner; made when
    /// first needed.
    small: Option<Small>,
    /// The targets of a menu's larger 3D, made when first needed.
    menu_high: Option<MenuHigh>,
    /// Which of its `colour` holds the frame in progress, once a flush of the frame has drawn
    /// 3D there.
    frame_high: Option<usize>,
    /// Whether the frame last finished is the larger picture (`MenuHigh::shown`).
    high_shown: bool,
    /// How many times the larger targets have been made.
    high_made: u64,
    merge: Pipeline,
    scene_vert: Shader,
    scene_frag: Shader,
    overlay: Pipeline,
    pipelines: HashMap<PipelineKey, Pipeline>,
    /// By `linear as usize + 2 * clamp as usize + 4 * mips as usize`.
    samplers: [Sampler; 8],
    /// Whether textures are made with their smaller levels: set the first time a batch asks for
    /// mipmapping, so a game that never does pays no memory for them.
    mip_levels: bool,
    nearest: Sampler,
    white: Texture,
    textures: super::TextureMap<CachedTexture>,
    vertices: Buffer,
    vertices_room: u32,
    transfer: Transfer,
    transfer_room: u32,
    /// The diagnostic cuts of the flush last drawn (`gpu_cut`), to log their changes.
    cuts: Cuts,
}

/// The GPU, once asked for and made.
pub static GPU: Mutex<Option<Gpu>> = Mutex::new(None);

/// Whether a device is to be tried: unless `--renderer software` or `RC2K_RENDERER=software`.
pub fn wanted() -> bool {
    super::choice::gpu_wanted()
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

/// Starts the backend on `device`; whether it runs. A backend that does not start has let go
/// of what it made (the device is the caller's).
pub fn start(device: Device) -> bool {
    let made = if super::choice::forced_failure("backend") {
        Err("made to fail".to_string())
    } else {
        Gpu::new(device)
    };
    match made {
        Ok(gpu) => {
            log(&format!("render: the GPU backend on {}", device.driver()));
            *GPU.lock().unwrap() = Some(gpu);
            super::gpu_started();
            true
        }
        Err(why) => {
            log(&format!(
                "render: the GPU backend does not start ({why}): the software rasteriser draws the 3D"
            ));
            false
        }
    }
}

/// Stops the backend if it runs: everything it made is let go, and the device is handed back
/// (the caller destroys it once the window's renderer on it is gone).
pub fn stop() -> Option<Device> {
    let gpu = GPU.lock().unwrap().take()?;
    super::gpu_stopped();
    let device = gpu.device;
    gpu.release();
    Some(device)
}

impl Gpu {
    pub fn new(device: Device) -> Result<Gpu, String> {
        let mut gpu = Gpu {
            device,
            colour: Texture::NONE,
            depth: Texture::NONE,
            canvas: Texture::NONE,
            canvas_transfer: Transfer::NONE,
            background: Texture::NONE,
            background_transfer: Transfer::NONE,
            background_holds: None,
            check: None,
            quads: None,
            width: 0,
            height: 0,
            targets_made: 0,
            depth_is_new: true,
            small: None,
            menu_high: None,
            frame_high: None,
            high_shown: false,
            high_made: 0,
            merge: Pipeline::NONE,
            scene_vert: Shader::NONE,
            scene_frag: Shader::NONE,
            overlay: Pipeline::NONE,
            pipelines: HashMap::new(),
            samplers: [Sampler::NONE; 8],
            mip_levels: false,
            nearest: Sampler::NONE,
            white: Texture::NONE,
            textures: Default::default(),
            vertices: Buffer::NONE,
            vertices_room: 0,
            transfer: Transfer::NONE,
            transfer_room: 0,
            cuts: Cuts::default(),
        };
        if let Some(line) = Cuts::start_from_env() {
            log(&line);
        }
        if let Err(why) = gpu.make_fixed_parts() {
            gpu.release();
            return Err(why);
        }
        Ok(gpu)
    }

    /// The shaders, the overlay's pipeline, the samplers and the white texture every frame
    /// needs.
    fn make_fixed_parts(&mut self) -> Result<(), String> {
        let gpu = self;
        gpu.scene_vert = gpu.shader(SCENE_VERT, SDL_GPU_SHADERSTAGE_VERTEX, 0, 1)?;
        gpu.scene_frag = gpu.shader(SCENE_FRAG, SDL_GPU_SHADERSTAGE_FRAGMENT, 1, 1)?;
        let overlay_vert = gpu.shader(OVERLAY_VERT, SDL_GPU_SHADERSTAGE_VERTEX, 0, 0)?;
        let overlay_frag = gpu.shader(OVERLAY_FRAG, SDL_GPU_SHADERSTAGE_FRAGMENT, 1, 0);
        let overlay_frag = match overlay_frag {
            Ok(shader) => shader,
            Err(why) => {
                gpu.device.release_shader(overlay_vert);
                return Err(why);
            }
        };
        let overlay = gpu.make_overlay_pipeline(overlay_vert, overlay_frag);
        // The pipeline keeps what it needs of them.
        gpu.device.release_shader(overlay_vert);
        gpu.device.release_shader(overlay_frag);
        gpu.overlay = overlay?;
        let merge_vert = gpu.shader(OVERLAY_VERT, SDL_GPU_SHADERSTAGE_VERTEX, 0, 0)?;
        let merge_frag = gpu.shader(MERGE_FRAG, SDL_GPU_SHADERSTAGE_FRAGMENT, 3, 0);
        let merge_frag = match merge_frag {
            Ok(shader) => shader,
            Err(why) => {
                gpu.device.release_shader(merge_vert);
                return Err(why);
            }
        };
        let merge = gpu.make_merge_pipeline(merge_vert, merge_frag);
        gpu.device.release_shader(merge_vert);
        gpu.device.release_shader(merge_frag);
        gpu.merge = merge?;
        for mips in [false, true] {
            for clamp in [false, true] {
                for linear in [false, true] {
                    gpu.samplers[linear as usize + 2 * clamp as usize + 4 * mips as usize] =
                        gpu.make_sampler(linear, clamp, mips)?;
                }
            }
        }
        gpu.nearest = gpu.samplers[0];
        gpu.white = gpu.make_texture(1, 1, SDL_GPU_TEXTUREUSAGE_SAMPLER)?;
        gpu.upload_white()
    }

    /// Everything the backend made let go (its device is the caller's): the device must be
    /// idle of the frame's work, which SDL waits for in each release.
    fn release(mut self) {
        let device = self.device;
        if !self.white.is_none() {
            device.release_texture(self.white);
        }
        if let Some(small) = self.small.take() {
            device.release_texture(small.colour);
            device.release_texture(small.depth);
        }
        if let Some(high) = self.menu_high.take() {
            for texture in [
                high.colour[0],
                high.colour[1],
                high.shown,
                high.depth,
                high.kept,
            ] {
                device.release_texture(texture);
            }
        }
        if !self.merge.is_none() {
            device.release_pipeline(self.merge);
        }
        self.release_targets(&Targets {
            colour: self.colour,
            depth: self.depth,
            canvas: self.canvas,
            canvas_transfer: self.canvas_transfer,
            background: self.background,
            background_transfer: self.background_transfer,
        });
        if let Some(check) = self.check.take() {
            for texture in [check.colour, check.depth, check.canvas] {
                device.release_texture(texture);
            }
            device.release_transfer(check.canvas_transfer);
        }
        if let Some(quads) = self.quads.take() {
            quads.release(device);
        }
        for (_, pipeline) in self.pipelines.drain() {
            device.release_pipeline(pipeline);
        }
        for (_, cached) in self.textures.drain() {
            device.release_texture(cached.texture);
        }
        for object in [self.scene_vert, self.scene_frag] {
            if !object.is_none() {
                device.release_shader(object);
            }
        }
        if !self.overlay.is_none() {
            device.release_pipeline(self.overlay);
        }
        for sampler in self.samplers {
            if !sampler.is_none() {
                device.release_sampler(sampler);
            }
        }
        if !self.vertices.is_none() {
            device.release_buffer(self.vertices);
        }
        if !self.transfer.is_none() {
            device.release_transfer(self.transfer);
        }
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

    fn make_sampler(&self, linear: bool, clamp: bool, mips: bool) -> Result<Sampler, String> {
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

    fn make_texture(
        &self,
        width: u32,
        height: u32,
        usage: SDL_GPUTextureUsageFlags,
    ) -> Result<Texture, String> {
        self.make_texture_of(COLOUR_FORMAT, width, height, usage)
    }

    /// A texture for the game's picture of `width` x `height`, with its smaller levels when they
    /// are being made.
    fn make_picture_texture(&self, width: u32, height: u32) -> Result<Texture, String> {
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
    fn make_merge_pipeline(
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
            background: std::mem::replace(&mut self.background, made.background),
            background_transfer: std::mem::replace(
                &mut self.background_transfer,
                made.background_transfer,
            ),
        };
        self.background_holds = None;
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
        made.background = self.make_texture(width, height, SDL_GPU_TEXTUREUSAGE_SAMPLER)?;
        made.background_transfer =
            self.make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD, width * height * 4)?;
        Ok(())
    }

    /// The targets released, those of them that were made.
    fn release_targets(&self, targets: &Targets) {
        for texture in [
            targets.colour,
            targets.depth,
            targets.canvas,
            targets.background,
        ] {
            if !texture.is_none() {
                self.device.release_texture(texture);
            }
        }
        for transfer in [targets.canvas_transfer, targets.background_transfer] {
            if !transfer.is_none() {
                self.device.release_transfer(transfer);
            }
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
    pub fn colour_texture(&self) -> (*mut SDL_GPUTexture, u64, (u32, u32)) {
        if self.high_shown
            && let Some(high) = &self.menu_high
        {
            // A made-again set of the larger targets is another number, past any of the
            // screen's sets.
            let number = (self.targets_made << 20) + self.high_made;
            return (high.shown.as_raw(), number, high.size);
        }
        (
            self.colour.as_raw(),
            self.targets_made,
            (self.width, self.height),
        )
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

    /// Logs a change of the live cuts.
    fn note_cuts(&mut self, cuts: Cuts) {
        if cuts != self.cuts {
            log(&format!("render: GPU cuts now on: {}", cuts.names()));
            self.cuts = cuts;
        }
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

    /// The noted background brought up to the GPU if it is not there already: whether the
    /// background texture is to be drawn under the canvas, and whether its upload is to be sent.
    fn bring_up_background(
        &mut self,
        picture_copy: &mut Option<Vec<u8>>,
    ) -> Result<(bool, bool), String> {
        use super::background::Pending;
        let Some(pending) = super::background::take() else {
            return Ok((false, false));
        };
        let bytes = self.width as usize * self.height as usize * 4;
        let buffers = crate::platform::direct3d::handles().pictures;
        if check::wanted() {
            *picture_copy = match &pending {
                Pending::Buffer { handle, .. } => {
                    crate::gfx::picture::with_buffer_unchanged(buffers, *handle, |b| {
                        b.get(..bytes).map(<[u8]>::to_vec)
                    })
                    .ok()
                    .flatten()
                }
                Pending::Copy(picture) => picture.get(..bytes).map(<[u8]>::to_vec),
                Pending::Rescaled { pixels, .. } => pixels.get(..bytes).map(<[u8]>::to_vec),
            };
        }
        let upload = super::background::bring_up(
            pending,
            &mut self.background_holds,
            bytes,
            buffers,
            |picture| {
                self.device
                    .fill_transfer(self.background_transfer, 0, picture, true)
            },
        )?;
        if upload {
            super::BACKGROUND_UPLOAD_BYTES
                .fetch_add(bytes as u64, std::sync::atomic::Ordering::Relaxed);
        }
        Ok((true, upload))
    }

    /// Draws the commands over the target: the retained background first if one is noted, then
    /// the canvas's written rows (`rows`, one flag a row; `None` to look at every row).
    /// The small targets (at least the screen's size, or the share's when larger), made or made
    /// again.
    fn ensure_small(&mut self, percent: u32) -> Result<(), String> {
        // Large enough for the screen's picture too (a share under 100 draws in the corner).
        let size = (
            (rc2k_core::gfx::render_scale::scaled(self.width as usize, percent) as u32)
                .max(self.width),
            (rc2k_core::gfx::render_scale::scaled(self.height as usize, percent) as u32)
                .max(self.height),
        );
        if self.small.as_ref().is_some_and(|s| s.size == size) {
            return Ok(());
        }
        let colour = self.make_texture(
            size.0,
            size.1,
            SDL_GPU_TEXTUREUSAGE_COLOR_TARGET | SDL_GPU_TEXTUREUSAGE_SAMPLER,
        )?;
        let depth = match self.make_texture_of(
            DEPTH_FORMAT,
            size.0,
            size.1,
            SDL_GPU_TEXTUREUSAGE_DEPTH_STENCIL_TARGET,
        ) {
            Ok(depth) => depth,
            Err(why) => {
                self.device.release_texture(colour);
                return Err(why);
            }
        };
        if let Some(old) = self.small.replace(Small {
            colour,
            depth,
            size,
            depth_is_new: true,
        }) {
            self.device.release_texture(old.colour);
            self.device.release_texture(old.depth);
        }
        Ok(())
    }

    pub fn draw(
        &mut self,
        canvas: &mut [u8],
        rows: Option<Vec<bool>>,
        commands: &CommandList,
    ) -> Result<(), String> {
        self.take_changes();
        let cuts = super::gpu_cut::live();
        self.note_cuts(cuts);
        let two_d = self.prepare_2d(crate::gfx::draw2d::quads::take())?;
        let mut background_bytes = None;
        let (background, upload_background) = self.bring_up_background(&mut background_bytes)?;
        self.ensure_check()?;
        let stride = self.width as usize * 4;
        let rows = rows.unwrap_or_else(|| super::canvas_rows::rows_holding_written(canvas, stride));
        let runs = super::canvas_rows::written_runs(&rows);
        // The check's canvas holds every written pixel, noted rows or not, as it did before.
        let old_canvas = self.check.is_some().then(|| {
            if crate::gfx::draw2d::quads::check_wanted() {
                // The canvas the old way would hold: everything the funnels wrote.
                return crate::gfx::draw2d::take_interval()
                    .unwrap_or_else(|| check::canvas_as_before(canvas, &[], stride, None));
            }
            let scan = super::canvas_rows::rows_holding_written(canvas, stride);
            check::note_missed_rows(&scan, &rows);
            let scanned = super::canvas_rows::written_runs(&scan);
            check::canvas_as_before(canvas, &scanned, stride, background_bytes.as_deref())
        });
        if let (Some(check), Some(old)) = (&self.check, &old_canvas) {
            self.device
                .fill_transfer(check.canvas_transfer, 0, old, true)?;
        }
        super::seethru::note_list(commands, self.width, self.height);
        let batches = Batches::of(commands, self.width, self.height, None);
        let vertex_bytes = batches.vertices.len() as u32 * VERTEX_BYTES;
        // The 3D of a race at a share of the screen's size: drawn into the small targets and
        // enlarged over the screen (the debug cuts and the check draw it whole).
        let percent = super::flush_scale();
        let smaller = percent != 100
            && !batches.batches.is_empty()
            && !cuts.scene
            && !cuts.half
            && self.check.is_none();
        if smaller {
            self.ensure_small(percent)?;
        }
        // A menu's 3D at the window's size, not the frame's.
        let high = super::menu_high_percent().filter(|_| {
            percent == 100
                && !batches.batches.is_empty()
                && !cuts.scene
                && !cuts.half
                && self.check.is_none()
                && !crate::gfx::draw2d::quads::check_2d_only()
        });
        if let Some(high) = high {
            self.ensure_menu_high(high)?;
        }
        if !self.mip_levels && batches.batches.iter().any(|b| b.mips) {
            // The textures made so far have a level; they are made again, with the rest, as
            // they are drawn.
            self.mip_levels = true;
            for (_, old) in self.textures.drain() {
                self.device.release_texture(old.texture);
            }
        }
        let mut uploads: Vec<(TextureNote, Vec<u8>)> = Vec::new();
        for note in &batches.textures {
            let up_to_date = self
                .textures
                .get(&note.pixels)
                .is_some_and(|t| !t.stale && (t.width, t.height) == (note.width, note.height));
            if !up_to_date {
                let mut bytes = super::texture_as_bgra(note);
                if self.mip_levels {
                    // The smaller levels follow the full picture, largest first.
                    let whole = bytes.len();
                    for (_, _, level) in
                        mip_levels::lower_levels(&bytes[..whole], note.width, note.height)
                    {
                        bytes.extend_from_slice(&level);
                    }
                }
                uploads.push((*note, bytes));
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
                let texture = self.make_picture_texture(note.width as u32, note.height as u32)?;
                if let Some(old) = self.textures.insert(
                    note.pixels,
                    CachedTexture {
                        texture,
                        width: note.width,
                        height: note.height,
                        stale: false,
                        levels: if self.mip_levels {
                            mip_levels::level_count(note.width, note.height) as u32
                        } else {
                            1
                        },
                    },
                ) {
                    self.device.release_texture(old.texture);
                }
            }
        }
        // The pipelines first: making one in the render pass would keep `self` borrowed.
        let mut pipelines = Vec::with_capacity(batches.batches.len());
        for batch in &batches.batches {
            pipelines.push(self.pipeline(cut_key(batch.pipeline, cuts))?);
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
        let parts: Vec<(usize, &[u8])> = runs
            .iter()
            .map(|&(first, last)| (first * stride, &canvas[first * stride..(last + 1) * stride]))
            .collect();
        super::GPU_FLUSHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if !parts.is_empty() {
            super::CANVAS_UPLOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.device
                .fill_transfer_parts(self.canvas_transfer, &parts)?;
            let sent: usize = parts.iter().map(|(_, bytes)| bytes.len()).sum();
            super::CANVAS_UPLOAD_BYTES.fetch_add(sent as u64, std::sync::atomic::Ordering::Relaxed);
        }
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
            if self.mip_levels {
                let mut at = *offset + (note.width * note.height * 4) as u32;
                for level in 1..texture.levels as usize {
                    let (w, h) = (
                        mip_levels::side_at(note.width, level) as u32,
                        mip_levels::side_at(note.height, level) as u32,
                    );
                    copy.upload_texture_level(
                        self.transfer,
                        at,
                        texture.texture,
                        level as u32,
                        w,
                        h,
                    );
                    at += w * h * 4;
                }
            }
            texture.stale = false;
        }
        if upload_background {
            copy.upload_texture(
                self.background_transfer,
                0,
                self.background,
                self.width,
                self.height,
            );
        }
        for &(first, last) in &runs {
            copy.upload_texture_rows(
                self.canvas_transfer,
                (first * stride) as u32,
                self.canvas,
                first as u32,
                self.width,
                (last + 1 - first) as u32,
            );
        }
        self.copy_2d(&copy, &two_d);
        copy.end();
        let clear_depth = std::mem::replace(&mut self.depth_is_new, false);
        let check_targets = self.check.as_mut().map(|c| {
            let clear = std::mem::replace(&mut c.depth_is_new, false);
            (c.colour, c.depth, c.canvas, c.canvas_transfer, clear)
        });
        // The rectangles are scaled with the viewport (the `half` cut's and the small 3D's).
        let draw_batches = |pass: &sdl_gpu::RenderPass, percent: u32| {
            let down = |n: i32| (n as i64 * i64::from(percent) / 100) as i32;
            let up = |n: i32| ((n as i64 * i64::from(percent) + 99) / 100) as i32;
            let (full_width, full_height) = (up(self.width as i32), up(self.height as i32));
            buffer.push_vertex_uniform(0, &[self.width as f32, self.height as f32]);
            pass.bind_vertex_buffer(self.vertices);
            let mut last_pipeline = None;
            let mut last_values = None;
            let mut last_sampling = None;
            let mut scissored = false;
            for (batch, &pipeline) in batches.batches.iter().zip(&pipelines) {
                if cuts.drops(batch.pipeline.blend.is_some()) {
                    continue;
                }
                if last_pipeline != Some(pipeline) {
                    pass.bind_pipeline(pipeline);
                    last_pipeline = Some(pipeline);
                }
                if last_values != Some(batch.values) {
                    let mut words = batch.values.to_words();
                    words[7] |= cuts.flag_bits();
                    buffer.push_fragment_uniform(0, &words);
                    last_values = Some(batch.values);
                }
                let sampling = (batch.texture, batch.linear, batch.clamp, batch.mips);
                if last_sampling != Some(sampling) {
                    let texture = batch.texture.map_or(self.white, |n| {
                        self.textures[&batches.textures[n as usize].pixels].texture
                    });
                    pass.bind_fragment_sampler(texture, self.samplers[sampler_number(batch, cuts)]);
                    last_sampling = Some(sampling);
                }
                if let Some([left, top, right, bottom]) = batch.scissor {
                    let (x0, y0) = (down(left as i32), down(top as i32));
                    pass.set_scissor(x0, y0, up(right as i32) - x0, up(bottom as i32) - y0);
                    scissored = true;
                } else if scissored {
                    pass.set_scissor(0, 0, full_width, full_height);
                    scissored = false;
                }
                pass.draw(batch.vertices, batch.first_vertex);
            }
        };
        let pass = buffer.begin_render_pass(self.colour, self.depth, clear_depth);
        let shrink = |n: i32| if cuts.half { n / 2 } else { n };
        if cuts.half {
            pass.set_viewport((self.width / 2) as f32, (self.height / 2) as f32);
        }
        if background && !cuts.two_d {
            self.draw_overlay(&pass, self.background);
        }
        if !cuts.two_d {
            for &(first, last) in &runs {
                pass.set_scissor(
                    0,
                    shrink(first as i32),
                    shrink(self.width as i32),
                    shrink((last + 1 - first) as i32),
                );
                self.draw_overlay(&pass, self.canvas);
            }
            if !runs.is_empty() {
                pass.set_scissor(0, 0, self.width as i32, self.height as i32);
            }
            self.draw_2d(&buffer, &pass, &two_d);
        }
        if !cuts.scene && !smaller && high.is_none() && !crate::gfx::draw2d::quads::check_2d_only()
        {
            draw_batches(&pass, if cuts.half { 50 } else { 100 });
        }
        pass.end();
        if let (Some(percent), Some(high)) = (high, self.menu_high.as_mut()) {
            let full = (self.width, self.height);
            let size = high.size;
            // The picture so far enlarged (or, when an earlier flush of the frame drew 3D, what
            // that left, with the 2D drawn since in its place) is what the 3D is drawn over.
            let into = match self.frame_high {
                None => {
                    buffer.blit(self.colour, full, high.colour[0], size, true);
                    0
                }
                Some(held) => {
                    let into = 1 - held;
                    let inputs = (self.merge, self.colour, self.samplers[3], self.nearest);
                    Self::merge_into(&buffer, inputs, high, high.colour[held], high.colour[into]);
                    into
                }
            };
            let clear = std::mem::replace(&mut high.depth_is_new, false);
            let pass = buffer.begin_render_pass(high.colour[into], high.depth, clear);
            pass.set_viewport(size.0 as f32, size.1 as f32);
            draw_batches(&pass, percent);
            pass.end();
            // The same 3D in the frame's own picture, which the 2D after it is drawn over, and
            // what it then holds kept to tell where that 2D is.
            let pass = buffer.begin_render_pass(self.colour, self.depth, false);
            draw_batches(&pass, 100);
            pass.end();
            buffer.blit(self.colour, full, high.kept, full, false);
            self.frame_high = Some(into);
        }
        if smaller && let Some(small) = self.small.as_mut() {
            let full = (self.width, self.height);
            let part = (
                rc2k_core::gfx::render_scale::scaled(full.0 as usize, percent) as u32,
                rc2k_core::gfx::render_scale::scaled(full.1 as usize, percent) as u32,
            );
            // The picture so far made smaller (or larger), the 3D drawn over it, the result
            // brought back to the screen's size: nearest when enlarging, the bilinear filter when
            // reducing (exactly the mean of four pixels at 200 percent). The 3D may blend with
            // what the screen held.
            buffer.blit(self.colour, full, small.colour, part, false);
            let clear = std::mem::replace(&mut small.depth_is_new, false);
            let pass = buffer.begin_render_pass(small.colour, small.depth, clear);
            pass.set_viewport(part.0 as f32, part.1 as f32);
            draw_batches(&pass, percent);
            pass.end();
            buffer.blit(small.colour, part, self.colour, full, percent > 100);
        }
        if let Some((colour, depth, canvas, transfer, clear)) = check_targets {
            // The same frame once more, the way the canvas carried the background.
            let copy = buffer.begin_copy_pass();
            copy.upload_texture(transfer, 0, canvas, self.width, self.height);
            copy.end();
            let pass = buffer.begin_render_pass(colour, depth, clear);
            if check::same() {
                // The noise floor: the same overlay as the real pass.
                if background {
                    self.draw_overlay(&pass, self.background);
                }
                for &(first, last) in &runs {
                    let rows = (last + 1 - first) as i32;
                    pass.set_scissor(0, first as i32, self.width as i32, rows);
                    self.draw_overlay(&pass, self.canvas);
                }
                pass.set_scissor(0, 0, self.width as i32, self.height as i32);
                self.draw_2d(&buffer, &pass, &two_d);
            } else {
                self.draw_overlay(&pass, canvas);
            }
            if !crate::gfx::draw2d::quads::check_2d_only() {
                draw_batches(&pass, 100);
            }
            pass.end();
        }
        buffer.submit()?;
        for &(first, last) in &runs {
            mark_unwritten(&mut canvas[first * stride..(last + 1) * stride]);
        }
        Ok(())
    }

    /// The written pixels of `layer` (the canvas, or the background) over the target, in an open
    /// pass.
    /// The larger targets of a menu's 3D at `percent` of the frame's size, made or made again.
    fn ensure_menu_high(&mut self, percent: u32) -> Result<(), String> {
        let size = (
            rc2k_core::gfx::render_scale::scaled(self.width as usize, percent) as u32,
            rc2k_core::gfx::render_scale::scaled(self.height as usize, percent) as u32,
        );
        if self.menu_high.as_ref().is_some_and(|h| h.size == size) {
            return Ok(());
        }
        let both = SDL_GPU_TEXTUREUSAGE_COLOR_TARGET | SDL_GPU_TEXTUREUSAGE_SAMPLER;
        let mut colour = [Texture::NONE; 2];
        let mut shown = Texture::NONE;
        let mut kept = Texture::NONE;
        let mut depth = Texture::NONE;
        let made = (|| -> Result<(), String> {
            colour[0] = self.make_texture(size.0, size.1, both)?;
            colour[1] = self.make_texture(size.0, size.1, both)?;
            shown = self.make_texture(size.0, size.1, both)?;
            kept = self.make_texture(self.width, self.height, both)?;
            depth = self.make_texture_of(
                DEPTH_FORMAT,
                size.0,
                size.1,
                SDL_GPU_TEXTUREUSAGE_DEPTH_STENCIL_TARGET,
            )?;
            Ok(())
        })();
        if let Err(why) = made {
            for texture in [colour[0], colour[1], shown, kept, depth] {
                if !texture.is_none() {
                    self.device.release_texture(texture);
                }
            }
            return Err(why);
        }
        if let Some(old) = self.menu_high.replace(MenuHigh {
            colour,
            shown,
            depth,
            kept,
            size,
            depth_is_new: true,
        }) {
            for texture in [old.colour[0], old.colour[1], old.shown, old.depth, old.kept] {
                self.device.release_texture(texture);
            }
        }
        self.high_made += 1;
        self.frame_high = None;
        self.high_shown = false;
        Ok(())
    }

    /// The frame's picture merged with the larger 3D picture `from` (`merge.frag`) into `to`.
    fn merge_into(
        buffer: &sdl_gpu::CommandBuffer,
        (pipeline, colour, linear, nearest): (Pipeline, Texture, Sampler, Sampler),
        high: &MenuHigh,
        from: Texture,
        to: Texture,
    ) {
        let pass = buffer.begin_render_pass(to, high.depth, false);
        pass.set_viewport(high.size.0 as f32, high.size.1 as f32);
        pass.bind_pipeline(pipeline);
        pass.bind_fragment_samplers3([(colour, linear), (high.kept, nearest), (from, nearest)]);
        pass.draw(3, 0);
        pass.end();
    }

    /// The end of a menu frame whose 3D was drawn larger: the merge of the frame's own picture
    /// (the text and buttons drawn after the 3D in it) and the larger 3D picture, which is what
    /// the window shows from here (`colour_texture`). The frame as it stands otherwise.
    pub fn finish_high(&mut self) -> Result<(), String> {
        let Some(held) = self.frame_high.take() else {
            self.high_shown = false;
            return Ok(());
        };
        let Some(high) = self.menu_high.as_ref() else {
            self.high_shown = false;
            return Ok(());
        };
        let buffer = self.device.acquire_command_buffer()?;
        let inputs = (self.merge, self.colour, self.samplers[3], self.nearest);
        Self::merge_into(&buffer, inputs, high, high.colour[held], high.shown);
        buffer.submit()?;
        self.high_shown = true;
        Ok(())
    }

    fn draw_overlay(&self, pass: &sdl_gpu::RenderPass, layer: Texture) {
        pass.bind_pipeline(self.overlay);
        pass.bind_fragment_sampler(layer, self.nearest);
        pass.draw(3, 0);
    }

    /// The frame's end: what the canvas has written goes over the target.
    pub fn finish_frame(
        &mut self,
        rows: Option<Vec<bool>>,
        canvas: &mut [u8],
    ) -> Result<(), String> {
        self.draw(canvas, rows, &CommandList::new())?;
        super::seethru::frame_end();
        self.compare_check()
    }

    /// The colour target's picture (the GPU's end of the frame is waited for).
    fn read_colour(&mut self) -> Result<Vec<u8>, String> {
        self.read_texture(self.colour)
    }

    /// The picture of a texture of the screen's size (the GPU's end of the frame is waited for).
    fn read_texture(&mut self, texture: Texture) -> Result<Vec<u8>, String> {
        let bytes = (self.width * self.height * 4) as usize;
        let transfer = self.make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_DOWNLOAD, bytes as u32)?;
        let buffer = self.device.acquire_command_buffer()?;
        let copy = buffer.begin_copy_pass();
        copy.download_texture(texture, self.width, self.height, transfer);
        copy.end();
        buffer.submit_and_wait(self.device)?;
        let mut picture = vec![0u8; bytes];
        self.device.read_transfer(transfer, &mut picture)?;
        self.device.release_transfer(transfer);
        Ok(picture)
    }

    /// The target's picture, with the canvas's written pixels, into the canvas.
    pub fn bring_back(&mut self, rows: Option<Vec<bool>>, canvas: &mut [u8]) -> Result<(), String> {
        self.finish_frame(rows, canvas)?;
        // The frame comes back as its own picture: the larger 3D of the frame so far is dropped.
        self.frame_high = None;
        self.high_shown = false;
        let picture = self.read_colour()?;
        canvas[..picture.len()].copy_from_slice(&picture);
        Ok(())
    }
}

fn sampler_number(batch: &Batch, cuts: Cuts) -> usize {
    let linear = (batch.linear || cuts.linear) && !cuts.nearest;
    linear as usize
        + 2 * batch.clamp as usize
        + 4 * (batch.mips && batch.texture.is_some()) as usize
}

/// The pipeline `key` asks for, with the state cuts applied: no blending, and no depth test
/// or writes.
fn cut_key(key: PipelineKey, cuts: Cuts) -> PipelineKey {
    PipelineKey {
        blend: if cuts.blend { None } else { key.blend },
        // Direct3D's "always" (any number past the comparisons' seven).
        depth_function: if cuts.depth { 8 } else { key.depth_function },
        depth_write: key.depth_write && !cuts.depth,
        ..key
    }
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
                .and_then(|()| {
                    let rows = super::canvas_rows::written_rows(target, canvas);
                    Gpu::draw(self, canvas, Some(rows), &commands)
                })
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
        let system = crate::test_support::own::system();
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
            .map(|b| crate::platform::direct3d::test_level(b.clone()))
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
        let soft_held = std::sync::Arc::new(crate::memory::pool::PoolBytes::from_vec(vec![
            0u8;
            width * height
                * 4
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
        Gpu::draw(&mut gpu, &mut canvas, None, &commands).unwrap();
        gpu.bring_back(None, &mut canvas).unwrap();

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
