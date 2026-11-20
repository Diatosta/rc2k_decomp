//! The colour targets (the frame's own, the small one, the menu's high-resolution one), their
//! sizes, and the passes that merge them.

use super::*;

impl Gpu {
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
    pub(super) fn make_targets(
        &self,
        made: &mut Targets,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
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
    pub(super) fn release_targets(&self, targets: &Targets) {
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

    pub(in crate::render) fn make_transfer(
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

    /// Draws the commands over the target: the retained background first if one is noted, then
    /// the canvas's written rows (`rows`, one flag a row; `None` to look at every row).
    /// The small targets (at least the screen's size, or the share's when larger), made or made
    /// again.
    pub(super) fn ensure_small(&mut self, percent: u32) -> Result<(), String> {
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

    /// The written pixels of `layer` (the canvas, or the background) over the target, in an open
    /// pass.
    /// The larger targets of a menu's 3D at `percent` of the frame's size, made or made again.
    pub(super) fn ensure_menu_high(&mut self, percent: u32) -> Result<(), String> {
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
    pub(super) fn merge_into(
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

    pub(in crate::render) fn draw_overlay(&self, pass: &sdl_gpu::RenderPass, layer: Texture) {
        pass.bind_pipeline(self.overlay);
        pass.bind_fragment_sampler(layer, self.nearest);
        pass.draw(3, 0);
    }
}
