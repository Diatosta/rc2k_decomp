//! The frame's picture stretched to a smaller one on the card and brought back at that size: a
//! texture made from the screen (the rear-view mirror's) reads a few thousand pixels of it, not
//! the whole frame's millions.

use super::{Gpu, Pipeline, STRETCH_FRAG, Texture};
use crate::platform::ffi::sdl_gpu::{self, Transfer};
use sdl3_sys::everything::*;

/// The target and the buffer for stretched pictures of one size, with their pipeline.
pub struct Stretched {
    /// The stretched picture's width and height.
    size: (u32, u32),
    target: Texture,
    transfer: Transfer,
    pipeline: Pipeline,
}

impl Stretched {
    pub fn release(self, device: sdl_gpu::Device) {
        device.release_texture(self.target);
        device.release_transfer(self.transfer);
        device.release_pipeline(self.pipeline);
    }
}

impl Gpu {
    /// The targets for stretched pictures of `size`, made when there are none or the size
    /// changed.
    fn stretched_for(&mut self, size: (u32, u32)) -> Result<(), String> {
        if self.stretched.as_ref().is_some_and(|s| s.size == size) {
            return Ok(());
        }
        if let Some(old) = self.stretched.take() {
            old.release(self.device);
        }
        let target = self.make_texture(size.0, size.1, SDL_GPU_TEXTUREUSAGE_COLOR_TARGET)?;
        let transfer =
            match self.make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_DOWNLOAD, size.0 * size.1 * 4) {
                Ok(transfer) => transfer,
                Err(why) => {
                    self.device.release_texture(target);
                    return Err(why);
                }
            };
        let pipeline = match self.make_stretch_pipeline() {
            Ok(pipeline) => pipeline,
            Err(why) => {
                self.device.release_texture(target);
                self.device.release_transfer(transfer);
                return Err(why);
            }
        };
        self.stretched = Some(Stretched {
            size,
            target,
            transfer,
            pipeline,
        });
        Ok(())
    }

    fn make_stretch_pipeline(&self) -> Result<Pipeline, String> {
        // The same pipeline as the video's planes (the whole target, no blending), one
        // uniform (the picture's size).
        self.make_plane_pipeline(STRETCH_FRAG, super::COLOUR_FORMAT, 1)
    }

    /// The frame's end, then its picture stretched to `size` pixels (each pixel the frame's at
    /// its place scaled down in whole numbers) and brought back as BGRA bytes, rows top first
    /// (the card's work so far is waited for). The canvas is left as `finish_frame` leaves it.
    /// `None`, with nothing done, when the frame is not `screen` pixels (the caller's picture
    /// is of another size than the card's).
    pub fn read_stretched(
        &mut self,
        rows: Option<Vec<bool>>,
        canvas: &mut [u8],
        screen: (u32, u32),
        size: (u32, u32),
    ) -> Result<Option<Vec<u8>>, String> {
        if screen != (self.width, self.height) || size.0 == 0 || size.1 == 0 {
            return Ok(None);
        }
        self.finish_frame(rows, canvas)?;
        // The frame comes as its own picture: the larger 3D of the frame so far is dropped.
        self.frame_high = None;
        self.high_shown = false;
        self.stretched_for(size)?;
        let Some(stretched) = self.stretched.as_ref() else {
            return Err("no stretched picture".into());
        };
        let (target, transfer, pipeline) =
            (stretched.target, stretched.transfer, stretched.pipeline);
        let buffer = self.device.acquire_command_buffer()?;
        let pass = buffer.begin_plain_pass(target);
        buffer.push_fragment_uniform(0, &[size.0, size.1, 0, 0, 0, 0, 0, 0]);
        pass.set_viewport(size.0 as f32, size.1 as f32);
        pass.bind_pipeline(pipeline);
        pass.bind_fragment_sampler(self.colour, self.nearest);
        pass.draw(3, 0);
        pass.end();
        let copy = buffer.begin_copy_pass();
        copy.download_texture(target, size.0, size.1, transfer);
        copy.end();
        buffer.submit_and_wait(self.device)?;
        let mut picture = vec![0u8; (size.0 * size.1 * 4) as usize];
        self.device.read_transfer(transfer, &mut picture)?;
        Ok(Some(picture))
    }
}
