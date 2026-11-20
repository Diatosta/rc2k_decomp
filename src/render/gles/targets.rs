//! The render targets (the small frame, the high-resolution one), their draw passes, the
//! merge, and the canvas textures.

use super::*;

impl Gpu {
    /// The frame buffer object at the screen's size.
    pub(super) fn set_size(&mut self, width: i32, height: i32) -> Result<(), String> {
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }
        let gl = &self.gl;
        // No pixels given.
        gl.bind_texture(TEXTURE_2D, self.colour);
        gl.tex_image_2d(0x8058, width, height, RGBA, UNSIGNED_BYTE, None);
        gl.tex_parameter(TEXTURE_2D, 0x2801, 0x2600);
        gl.tex_parameter(TEXTURE_2D, 0x2800, 0x2600);
        gl.bind_renderbuffer(0x8D41, self.depth);
        gl.renderbuffer_storage(0x8D41, 0x81A6, width, height);
        gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        gl.framebuffer_texture_2d(FRAMEBUFFER, 0x8CE0, self.colour);
        gl.framebuffer_renderbuffer(FRAMEBUFFER, 0x8D00, self.depth);
        if gl.framebuffer_status(FRAMEBUFFER) != 0x8CD5 {
            return Err("the frame buffer object is not complete".into());
        }
        (self.width, self.height) = (width, height);
        self.depth_is_new = true;
        Ok(())
    }

    /// The small target at `width` x `height`, made or made again.
    pub(super) fn ensure_small(&mut self, width: i32, height: i32) -> Result<(), String> {
        if self
            .small
            .as_ref()
            .is_some_and(|s| s.size == (width, height))
        {
            return Ok(());
        }
        let gl = &self.gl;
        let (framebuffer, colour, depth) = match self.small.take() {
            Some(old) => (old.framebuffer, old.colour, old.depth),
            None => (
                gl.gen_framebuffer(),
                gl.gen_texture(),
                gl.gen_renderbuffer(),
            ),
        };
        gl.bind_texture(TEXTURE_2D, colour);
        gl.tex_image_2d(0x8058, width, height, RGBA, UNSIGNED_BYTE, None);
        gl.tex_parameter(TEXTURE_2D, 0x2801, 0x2600);
        gl.tex_parameter(TEXTURE_2D, 0x2800, 0x2600);
        gl.bind_renderbuffer(0x8D41, depth);
        gl.renderbuffer_storage(0x8D41, 0x81A6, width, height);
        gl.bind_framebuffer(FRAMEBUFFER, framebuffer);
        gl.framebuffer_texture_2d(FRAMEBUFFER, 0x8CE0, colour);
        gl.framebuffer_renderbuffer(FRAMEBUFFER, 0x8D00, depth);
        let complete = gl.framebuffer_status(FRAMEBUFFER) == 0x8CD5;
        gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        if !complete {
            return Err("the small frame buffer object is not complete".into());
        }
        self.small = Some(SmallTarget {
            framebuffer,
            colour,
            depth,
            size: (width, height),
            depth_is_new: true,
        });
        Ok(())
    }

    /// The part's 3D at `percent` of the screen's size: the picture so far made smaller (or
    /// larger), the batches drawn over it, the result brought back over the target.
    pub(super) fn draw_batches_small(
        &mut self,
        batches: &Batches,
        (width, height): (i32, i32),
        percent: u32,
    ) -> Result<(), String> {
        let part = (
            rc2k_core::gfx::render_scale::scaled(width as usize, percent) as i32,
            rc2k_core::gfx::render_scale::scaled(height as usize, percent) as i32,
        );
        // The target holds the screen's picture too (a share under 100 draws in the corner).
        self.ensure_small(part.0.max(width), part.1.max(height))?;
        let gl = &self.gl;
        let small = self.small.as_mut().expect("made above");
        gl.disable(SCISSOR_TEST);
        gl.bind_framebuffer(READ_FRAMEBUFFER, self.framebuffer);
        gl.bind_framebuffer(DRAW_FRAMEBUFFER, small.framebuffer);
        gl.blit_colour((width, height), part, false);
        gl.bind_framebuffer(FRAMEBUFFER, small.framebuffer);
        gl.viewport(0, 0, part.0, part.1);
        if std::mem::replace(&mut small.depth_is_new, false) {
            gl.depth_mask(true);
            gl.clear_depth(1.0);
            gl.clear(DEPTH_BIT);
        }
        let small_framebuffer = small.framebuffer;
        self.draw_batches(batches, width, height, percent);
        self.plain_state();
        let gl = &self.gl;
        gl.bind_framebuffer(READ_FRAMEBUFFER, small_framebuffer);
        gl.bind_framebuffer(DRAW_FRAMEBUFFER, self.framebuffer);
        // Reduced with the bilinear filter (the mean of four pixels at 200 percent).
        gl.blit_colour(part, (width, height), percent > 100);
        gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        gl.viewport(0, 0, width, height);
        Ok(())
    }

    /// The larger targets of a menu's 3D at `size`, made or made again.
    pub(super) fn ensure_high(
        &mut self,
        size: (i32, i32),
        frame: (i32, i32),
    ) -> Result<(), String> {
        if self
            .high
            .as_ref()
            .is_some_and(|h| h.size == size && h.kept_size == frame)
        {
            return Ok(());
        }
        let gl = &self.gl;
        let mut made = match self.high.take() {
            Some(old) => old,
            None => HighTarget {
                framebuffer: [gl.gen_framebuffer(), gl.gen_framebuffer()],
                colour: [gl.gen_texture(), gl.gen_texture()],
                depth: gl.gen_renderbuffer(),
                kept_framebuffer: gl.gen_framebuffer(),
                kept: gl.gen_texture(),
                size,
                kept_size: frame,
                depth_is_new: true,
            },
        };
        gl.bind_renderbuffer(0x8D41, made.depth);
        gl.renderbuffer_storage(0x8D41, 0x81A6, size.0, size.1);
        let mut complete = true;
        for n in 0..2 {
            gl.bind_texture(TEXTURE_2D, made.colour[n]);
            gl.tex_image_2d(0x8058, size.0, size.1, RGBA, UNSIGNED_BYTE, None);
            // Linear: it is shown at the window's size.
            gl.tex_parameter(TEXTURE_2D, 0x2801, 0x2601);
            gl.tex_parameter(TEXTURE_2D, 0x2800, 0x2601);
            gl.tex_parameter(TEXTURE_2D, 0x2802, 0x812F);
            gl.tex_parameter(TEXTURE_2D, 0x2803, 0x812F);
            gl.bind_framebuffer(FRAMEBUFFER, made.framebuffer[n]);
            gl.framebuffer_texture_2d(FRAMEBUFFER, 0x8CE0, made.colour[n]);
            gl.framebuffer_renderbuffer(FRAMEBUFFER, 0x8D00, made.depth);
            complete &= gl.framebuffer_status(FRAMEBUFFER) == 0x8CD5;
        }
        gl.bind_texture(TEXTURE_2D, made.kept);
        gl.tex_image_2d(0x8058, frame.0, frame.1, RGBA, UNSIGNED_BYTE, None);
        gl.tex_parameter(TEXTURE_2D, 0x2801, 0x2600);
        gl.tex_parameter(TEXTURE_2D, 0x2800, 0x2600);
        gl.bind_framebuffer(FRAMEBUFFER, made.kept_framebuffer);
        gl.framebuffer_texture_2d(FRAMEBUFFER, 0x8CE0, made.kept);
        complete &= gl.framebuffer_status(FRAMEBUFFER) == 0x8CD5;
        gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        made.size = size;
        made.kept_size = frame;
        made.depth_is_new = true;
        self.frame_high = None;
        self.high = Some(made);
        if complete {
            Ok(())
        } else {
            Err("the menu's larger frame buffer object is not complete".into())
        }
    }

    /// A menu part's 3D at `percent` of the frame's size: the picture so far enlarged (or, when an
    /// earlier part of the frame drew 3D, what that left, with the 2D drawn since in its place),
    /// the batches drawn over it; and the same batches in the frame's own picture, which the 2D
    /// after them is drawn over, kept to tell where that 2D is (`MERGE_FS`).
    pub(super) fn draw_batches_high(
        &mut self,
        batches: &Batches,
        (width, height): (i32, i32),
        percent: u32,
    ) -> Result<(), String> {
        let size = (
            rc2k_core::gfx::render_scale::scaled(width as usize, percent) as i32,
            rc2k_core::gfx::render_scale::scaled(height as usize, percent) as i32,
        );
        self.ensure_high(size, (width, height))?;
        let gl = &self.gl;
        gl.disable(SCISSOR_TEST);
        let high = self.high.as_ref().expect("made above");
        let into = match self.frame_high {
            None => {
                gl.bind_framebuffer(READ_FRAMEBUFFER, self.framebuffer);
                gl.bind_framebuffer(DRAW_FRAMEBUFFER, high.framebuffer[0]);
                gl.blit_colour((width, height), size, true);
                0
            }
            Some(held) => {
                let into = 1 - held;
                gl.bind_framebuffer(FRAMEBUFFER, high.framebuffer[into]);
                gl.viewport(0, 0, size.0, size.1);
                self.bind_merge_inputs(held);
                self.draw_quad(
                    &self.merge_quad,
                    true,
                    Some(Placement {
                        fit: [0.0, 0.0, 1.0, 1.0],
                        bars: true,
                        sharp: true,
                    }),
                );
                into
            }
        };
        let gl = &self.gl;
        let high = self.high.as_mut().expect("made above");
        gl.bind_framebuffer(FRAMEBUFFER, high.framebuffer[into]);
        gl.viewport(0, 0, size.0, size.1);
        if std::mem::replace(&mut high.depth_is_new, false) {
            gl.depth_mask(true);
            gl.clear_depth(1.0);
            gl.clear(DEPTH_BIT);
        }
        let kept_framebuffer = high.kept_framebuffer;
        self.draw_batches(batches, width, height, percent);
        let gl = &self.gl;
        gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        gl.viewport(0, 0, width, height);
        self.draw_batches(batches, width, height, 100);
        self.plain_state();
        let gl = &self.gl;
        gl.bind_framebuffer(READ_FRAMEBUFFER, self.framebuffer);
        gl.bind_framebuffer(DRAW_FRAMEBUFFER, kept_framebuffer);
        gl.blit_colour((width, height), (width, height), false);
        gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        self.frame_high = Some(into);
        Ok(())
    }

    /// The merge's textures bound: the frame's picture on unit 0, the kept one on 2 and the larger
    /// 3D `held` on 3, the wash on 1.
    pub(super) fn bind_merge_inputs(&self, held: usize) {
        let gl = &self.gl;
        let high = self.high.as_ref().expect("made before");
        self.plain_state();
        for (unit, texture) in [(0, self.colour), (2, high.kept), (3, high.colour[held])] {
            gl.active_texture(TEXTURE0 + unit);
            gl.bind_texture(TEXTURE_2D, texture);
            gl.bind_sampler(unit, 0);
        }
        gl.active_texture(TEXTURE0);
        self.bind_wash();
    }

    /// The next of the canvas's textures becomes the one uploads go into and draws read, bound
    /// and made at `width` x `height` (with no pixels) if it is not of that size yet. Once for
    /// each flush that uploads: every flush draws only what it uploaded, so nothing is carried
    /// from one texture to the next.
    pub(super) fn next_canvas(&mut self, width: i32, height: i32) {
        self.canvas_now = (self.canvas_now + 1) % CANVAS_TEXTURES;
        let canvas = &mut self.canvases[self.canvas_now];
        let gl = &self.gl;
        gl.bind_texture(TEXTURE_2D, canvas.texture);
        gl.bind_sampler(0, 0);
        if canvas.size != (width, height) {
            gl.tex_image_2d(0x8058, width, height, RGBA, UNSIGNED_BYTE, None);
            gl.tex_parameter(TEXTURE_2D, 0x2801, 0x2600);
            gl.tex_parameter(TEXTURE_2D, 0x2800, 0x2600);
            canvas.size = (width, height);
        }
    }

    /// The canvas's bytes into the next of its textures.
    pub(super) fn upload_canvas(&mut self, canvas: &[u8], width: i32, height: i32) {
        self.next_canvas(width, height);
        self.gl
            .tex_sub_image_2d((0, 0), (width, height), RGBA, UNSIGNED_BYTE, canvas);
    }

    /// Rows `first ..= last` of the canvas (`rows`, those alone) into the texture `next_canvas`
    /// chose and left bound. What the texture holds outside the rows is cut off by the overlay's
    /// scissor.
    pub(super) fn upload_canvas_rows(&self, rows: &[u8], width: i32, first: usize, last: usize) {
        self.gl.tex_sub_image_2d(
            (0, first as i32),
            (width, (last - first + 1) as i32),
            RGBA,
            UNSIGNED_BYTE,
            rows,
        );
    }
}
