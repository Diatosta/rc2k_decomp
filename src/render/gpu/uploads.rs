//! What goes to the GPU between draws: the transfer buffer, the white texture, the room for a
//! frame's vertices, the pipeline changes noted, and the menu's background.

use super::*;

impl Gpu {
    /// `bytes` into the transfer buffer `transfer` from `at` (cycled when `at` is 0: the last
    /// frame's may be in use).
    pub(super) fn fill_transfer(
        &self,
        transfer: Transfer,
        at: usize,
        bytes: &[u8],
    ) -> Result<(), String> {
        self.device.fill_transfer(transfer, at, bytes, at == 0)
    }

    pub(super) fn upload_white(&self) -> Result<(), String> {
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
    pub(super) fn ensure_room(
        &mut self,
        vertex_bytes: u32,
        transfer_bytes: u32,
    ) -> Result<(), String> {
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
    pub(super) fn note_cuts(&mut self, cuts: Cuts) {
        if cuts != self.cuts {
            log(&format!("render: GPU cuts now on: {}", cuts.names()));
            self.cuts = cuts;
        }
    }

    /// Forgets what is known of textures whose memory has changed, and lets go of those whose
    /// memory is gone.
    pub(super) fn take_changes(&mut self) {
        let changes = crate::render::take_changed_textures();
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
    pub(super) fn bring_up_background(
        &mut self,
        picture_copy: &mut Option<Vec<u8>>,
    ) -> Result<(bool, bool), String> {
        use crate::render::background::Pending;
        let Some(pending) = crate::render::background::take() else {
            return Ok((false, false));
        };
        let bytes = self.width as usize * self.height as usize * 4;
        let buffers = crate::d3d::device::handles().pictures;
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
        let upload = crate::render::background::bring_up(
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
            crate::render::BACKGROUND_UPLOAD_BYTES
                .fetch_add(bytes as u64, std::sync::atomic::Ordering::Relaxed);
        }
        Ok((true, upload))
    }
}
