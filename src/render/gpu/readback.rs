//! The frame and the films' pictures read back from the GPU.

use super::*;

impl Gpu {
    /// The picture of a texture of the screen's size (the GPU's end of the frame is waited for)
    /// into the start of `out`.
    pub(super) fn read_texture_into(
        &mut self,
        texture: Texture,
        out: &mut [u8],
    ) -> Result<(), String> {
        let bytes = (self.width * self.height * 4) as usize;
        let picture = out
            .get_mut(..bytes)
            .ok_or("the canvas is smaller than the screen")?;
        if self.download.is_none() || self.download.size() as usize != bytes {
            if !self.download.is_none() {
                self.device.release_transfer(self.download);
            }
            self.download = Transfer::NONE;
            self.download =
                self.make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_DOWNLOAD, bytes as u32)?;
        }
        let buffer = self.device.acquire_command_buffer()?;
        let copy = buffer.begin_copy_pass();
        copy.download_texture(texture, self.width, self.height, self.download);
        copy.end();
        buffer.submit_and_wait(self.device)?;
        self.device.read_transfer(self.download, picture)
    }

    /// The target's picture, with the canvas's written pixels, into the canvas.
    pub fn bring_back(&mut self, rows: Option<Vec<bool>>, canvas: &mut [u8]) -> Result<(), String> {
        self.finish_frame(rows, canvas)?;
        // The frame comes back as its own picture: the larger 3D of the frame so far is dropped.
        self.frame_high = None;
        self.high_shown = false;
        self.read_texture_into(self.colour, canvas)
    }

    /// The frame's end, and the colour target's picture started on its way to the CPU: the
    /// copy is queued behind the frame's drawing and not waited for, so the game goes on with
    /// the next frame; `collect_video_read` takes the picture when it is there.
    ///
    /// With `nv12` the picture is converted on the card first and comes as the planes of NV12
    /// at the even size (`nv12::even_size`), 1.5 bytes a pixel.
    #[cfg(not(handheld))]
    pub fn start_video_read(
        &mut self,
        rows: Option<Vec<bool>>,
        canvas: &mut [u8],
        nv12: bool,
    ) -> Result<(), String> {
        debug_assert!(
            self.video_read.is_none(),
            "the last frame was not collected"
        );
        self.finish_frame(rows, canvas)?;
        // The frame comes as its own picture: the larger 3D of the frame so far is dropped.
        self.frame_high = None;
        self.high_shown = false;
        let fence = if nv12 {
            self.queue_nv12_read()?
        } else {
            self.video_transfer_of(self.width * self.height * 4)?;
            let buffer = self.device.acquire_command_buffer()?;
            let copy = buffer.begin_copy_pass();
            copy.download_texture(self.colour, self.width, self.height, self.video_transfer);
            copy.end();
            buffer.submit_with_fence()?
        };
        self.video_read = Some((self.video_transfer, fence));
        Ok(())
    }

    /// `video_transfer` made of `bytes` bytes (kept when it already is).
    pub(super) fn video_transfer_of(&mut self, bytes: u32) -> Result<(), String> {
        if self.video_transfer.is_none() || self.video_transfer.size() != bytes {
            if !self.video_transfer.is_none() {
                self.device.release_transfer(self.video_transfer);
            }
            self.video_transfer = Transfer::NONE;
            self.video_transfer =
                self.make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_DOWNLOAD, bytes)?;
        }
        Ok(())
    }

    /// The picture `start_video_read` started, waited for and copied into `out` (resized to
    /// it); false when none was started.
    pub fn collect_video_read(&mut self, out: &mut Vec<u8>) -> Result<bool, String> {
        let Some((transfer, fence)) = self.video_read.take() else {
            return Ok(false);
        };
        self.device.wait_fence(fence);
        out.resize(transfer.size() as usize, 0);
        self.device.read_transfer(transfer, out)?;
        Ok(true)
    }

    /// The colour target's picture (the GPU's end of the frame is waited for).
    pub(in crate::render) fn read_colour(&mut self) -> Result<Vec<u8>, String> {
        self.read_texture(self.colour)
    }

    /// The picture of a texture of the screen's size, as a new buffer.
    pub(in crate::render) fn read_texture(&mut self, texture: Texture) -> Result<Vec<u8>, String> {
        let mut picture = vec![0u8; (self.width * self.height * 4) as usize];
        self.read_texture_into(texture, &mut picture)?;
        Ok(picture)
    }
}
