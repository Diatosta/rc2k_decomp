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

mod device;
mod draw;
mod nv12;
mod pipelines;
mod quads;
mod readback;
mod stretch;
mod targets;
mod uploads;

#[cfg(target_os = "android")]
pub use self::device::probe_early;
pub use self::device::{make_device, size_targets, software_phone, start, stop, wanted};
use self::pipelines::*;

use super::batches::{Batch, Batches, PipelineKey};
use super::dev::check;
use super::dev::gpu_cut::Cuts;
use super::gles::mip_levels;
use super::{CommandList, Target, TextureNote};
use crate::platform::ffi::sdl_gpu::{
    self, Buffer, Device, Fence, Pipeline, PipelineSpec, Sampler, Shader, ShaderCode, Texture,
    Transfer,
};
use crate::program_log::log;
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
            #[cfg(apple)]
            msl: include_bytes!(concat!("shaders/", $name, ".msl")),
        }
    };
}

const SCENE_VERT: ShaderCode = shader_code!("scene.vert");
const SCENE_FRAG: ShaderCode = shader_code!("scene.frag");
const OVERLAY_VERT: ShaderCode = shader_code!("overlay.vert");
const OVERLAY_FRAG: ShaderCode = shader_code!("overlay.frag");
const MERGE_FRAG: ShaderCode = shader_code!("merge.frag");
/// The video's conversion to NV12 (`nv12`).
const Y_FRAG: ShaderCode = shader_code!("nv12_y.frag");
const UV_FRAG: ShaderCode = shader_code!("nv12_uv.frag");
/// The frame stretched to a smaller picture (`stretch`).
const STRETCH_FRAG: ShaderCode = shader_code!("stretch.frag");

const COLOUR_FORMAT: SDL_GPUTextureFormat = SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM;
pub(in crate::render) const DEPTH_FORMAT: SDL_GPUTextureFormat = SDL_GPU_TEXTUREFORMAT_D32_FLOAT;
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
    pub(in crate::render) device: Device,
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
    pub(in crate::render) check: Option<check::Check>,
    /// The 2D quads' atlas and buffers (`quads`), made when first needed.
    quads: Option<quads::Quads>,
    pub(in crate::render) width: u32,
    pub(in crate::render) height: u32,
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
    /// The buffer the picture is read back through, kept from one read-back to the next (a new
    /// one of the screen's size every frame costs a render more than the copy does).
    download: Transfer,
    /// The video's frame being read back: the buffer it comes through and the fence of its copy
    /// (`start_video_read`, `collect_video_read`).
    video_read: Option<(Transfer, Fence)>,
    /// The buffer the video's frames come through, one at a time.
    video_transfer: Transfer,
    /// The targets of the video's conversion to NV12 (`nv12`), once a frame asked for it.
    planes: Option<nv12::Planes>,
    /// The target and buffer of the frame's stretched copy (`stretch`), once a frame asked for it.
    stretched: Option<stretch::Stretched>,
    /// The diagnostic cuts of the flush last drawn (`gpu_cut`), to log their changes.
    cuts: Cuts,
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
            download: Transfer::NONE,
            video_read: None,
            video_transfer: Transfer::NONE,
            planes: None,
            stretched: None,
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
        if !self.download.is_none() {
            device.release_transfer(self.download);
        }
        if let Some((_, fence)) = self.video_read.take() {
            device.wait_fence(fence);
        }
        if !self.video_transfer.is_none() {
            device.release_transfer(self.video_transfer);
        }
        if let Some(planes) = self.planes.take() {
            planes.release(device);
        }
        if let Some(stretched) = self.stretched.take() {
            stretched.release(device);
        }
    }
}

/// Every pixel of the canvas marked as not written.
pub fn mark_unwritten(canvas: &mut [u8]) {
    for pixel in canvas.as_chunks_mut::<4>().0 {
        *pixel = UNWRITTEN.to_le_bytes();
    }
}

impl super::Backend for Gpu {
    fn draw(&mut self, target: &Target, commands: CommandList) {
        let began = std::time::Instant::now();
        let done = target.with_canvas(|canvas| {
            self.set_size(target.width as u32, target.height as u32)
                .and_then(|()| {
                    let rows = super::canvas_rows::written_rows(target, canvas);
                    Gpu::draw(self, canvas, Some(rows), &commands)
                })
        });
        crate::platform::timing::FLUSH_RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        crate::platform::timing::note_span(
            &crate::platform::timing::FLUSH_MICROS,
            &crate::platform::timing::FLUSH_MAX,
            began.elapsed().as_micros() as u64,
        );
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
mod comparison;
