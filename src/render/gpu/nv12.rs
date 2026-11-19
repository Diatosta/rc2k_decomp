//! The video's frames converted on the card: the colour target's picture becomes the two planes
//! of NV12 (a luma plane, then a plane of blue and red differences at half the size each way)
//! in two small passes, and comes back at 1.5 bytes a pixel instead of 4. The encoder then
//! takes the planes as they are (`ffmpeg -pix_fmt nv12`), whatever it is: a hardware encoder
//! of any make, or x264.

use super::{Gpu, OVERLAY_VERT, Pipeline, PipelineSpec, Texture, UV_FRAG, Y_FRAG};
use crate::platform::ffi::sdl_gpu::{self, ShaderCode};
use rc2k_core::gfx::nv12::{even_size, frame_bytes};
use sdl3_sys::everything::*;

/// The two planes' targets and pipelines for frames of one size.
pub struct Planes {
    /// The size of the luma plane: the frame's, each side cut to an even number.
    pub size: (u32, u32),
    luma: Texture,
    chroma: Texture,
    luma_pipeline: Pipeline,
    chroma_pipeline: Pipeline,
}

impl Planes {
    pub fn release(self, device: sdl_gpu::Device) {
        device.release_texture(self.luma);
        device.release_texture(self.chroma);
        device.release_pipeline(self.luma_pipeline);
        device.release_pipeline(self.chroma_pipeline);
    }
}

impl Gpu {
    /// A pipeline that draws `fragment` over the whole of a target of `format`.
    fn make_plane_pipeline(
        &self,
        fragment: ShaderCode,
        format: SDL_GPUTextureFormat,
    ) -> Result<Pipeline, String> {
        let vertex = self.shader(OVERLAY_VERT, SDL_GPU_SHADERSTAGE_VERTEX, 0, 0)?;
        let fragment = match self.shader(fragment, SDL_GPU_SHADERSTAGE_FRAGMENT, 1, 0) {
            Ok(shader) => shader,
            Err(why) => {
                self.device.release_shader(vertex);
                return Err(why);
            }
        };
        let spec = PipelineSpec {
            vertices: None,
            colour: SDL_GPUColorTargetDescription {
                format,
                blend_state: SDL_GPUColorTargetBlendState::default(),
            },
            rasterizer: SDL_GPURasterizerState::default(),
            depth: None,
        };
        let pipeline = self.device.create_pipeline(vertex, fragment, &spec);
        self.device.release_shader(vertex);
        self.device.release_shader(fragment);
        pipeline
    }

    /// The planes' targets for frames of the screen's size, made when there are none or the
    /// screen's size changed.
    fn planes_for_size(&mut self) -> Result<(), String> {
        let size = even_size(self.width, self.height);
        if self.planes.as_ref().is_some_and(|p| p.size == size) {
            return Ok(());
        }
        if let Some(old) = self.planes.take() {
            old.release(self.device);
        }
        let usage = SDL_GPU_TEXTUREUSAGE_COLOR_TARGET;
        let luma = self.make_texture_of(SDL_GPU_TEXTUREFORMAT_R8_UNORM, size.0, size.1, usage)?;
        let chroma = self.make_texture_of(
            SDL_GPU_TEXTUREFORMAT_R8G8_UNORM,
            size.0 / 2,
            size.1 / 2,
            usage,
        )?;
        let luma_pipeline = self.make_plane_pipeline(Y_FRAG, SDL_GPU_TEXTUREFORMAT_R8_UNORM)?;
        let chroma_pipeline =
            self.make_plane_pipeline(UV_FRAG, SDL_GPU_TEXTUREFORMAT_R8G8_UNORM)?;
        self.planes = Some(Planes {
            size,
            luma,
            chroma,
            luma_pipeline,
            chroma_pipeline,
        });
        Ok(())
    }

    /// The colour target's picture converted to NV12 and queued to come back (the copy's end is
    /// the returned fence; the bytes are in `video_transfer`: the luma plane, then the chroma).
    pub(super) fn queue_nv12_read(&mut self) -> Result<sdl_gpu::Fence, String> {
        self.planes_for_size()?;
        let Some(planes) = self.planes.as_ref() else {
            return Err("no planes".into());
        };
        let (size, luma, chroma) = (planes.size, planes.luma, planes.chroma);
        let (luma_pipeline, chroma_pipeline) = (planes.luma_pipeline, planes.chroma_pipeline);
        let bytes = frame_bytes(size) as u32;
        self.video_transfer_of(bytes)?;
        let buffer = self.device.acquire_command_buffer()?;
        for (target, pipeline, plane) in [
            (luma, luma_pipeline, size),
            (chroma, chroma_pipeline, (size.0 / 2, size.1 / 2)),
        ] {
            let pass = buffer.begin_plain_pass(target);
            pass.set_viewport(plane.0 as f32, plane.1 as f32);
            pass.bind_pipeline(pipeline);
            pass.bind_fragment_sampler(self.colour, self.nearest);
            pass.draw(3, 0);
            pass.end();
        }
        let copy = buffer.begin_copy_pass();
        copy.download_texture_at(luma, size, 1, self.video_transfer, 0);
        copy.download_texture_at(
            chroma,
            (size.0 / 2, size.1 / 2),
            2,
            self.video_transfer,
            size.0 * size.1,
        );
        copy.end();
        buffer.submit_with_fence()
    }
}
