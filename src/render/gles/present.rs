//! Putting the target on the panel or window: the viewport, the wash, the filter, the shown
//! pictures, and reading the target back.

use super::*;

impl Gpu {
    /// The whole of the panel (or the window) black, and the viewport where a picture of `width`
    /// x `height` shows. The window (the handheld's panel too) is drawn over its whole, the
    /// picture placed in it keeping its shape (`Placement`): a `menu`'s picture has its wash
    /// carried on beside it, a race's black. Where there is no window to ask, the picture is
    /// centred and shrunk to fit if it is too big.
    pub(super) fn panel_viewport(&self, width: i32, height: i32, menu: bool) -> Placement {
        let gl = &self.gl;
        let whole = Placement {
            fit: [0.0, 0.0, 1.0, 1.0],
            bars: false,
            sharp: true,
        };
        let (view, placed) = match self.drawable_size() {
            Some((w, h)) => {
                let fit = crate::platform::safe_area::frame_fit(
                    (width as u32, height as u32),
                    (w as u32, h as u32),
                    menu,
                );
                // The menus' 3D of the next frames is drawn at the size this frame's place has in
                // the window, times the 3D's share.
                crate::render::set_menu_high(if menu {
                    let enlarged = u64::from(fit.width) * 100 / (width.max(1) as u64);
                    let share = u64::from(crate::render::render_scale());
                    (enlarged * share / 100).min(400) as u32
                } else {
                    0
                });
                let placed = Placement {
                    fit: [
                        fit.x as f32 / w as f32,
                        fit.y as f32 / h as f32,
                        fit.width as f32 / w as f32,
                        fit.height as f32 / h as f32,
                    ],
                    bars: !menu || crate::render::wash::film_showing(),
                    sharp: fit.sharp,
                };
                ((0, 0, w, h), placed)
            }
            None => {
                let (w, h) = if width > PANEL.0 || height > PANEL.1 {
                    (width * PANEL.1 / height, PANEL.1)
                } else {
                    (width, height)
                };
                (((PANEL.0 - w) / 2, (PANEL.1 - h) / 2, w, h), whole)
            }
        };
        gl.bind_framebuffer(FRAMEBUFFER, 0);
        gl.clear_colour(0.0, 0.0, 0.0, 1.0);
        gl.clear(COLOUR_BIT);
        gl.viewport(view.0, view.1, view.2, view.3);
        placed
    }

    /// The size of what the GL draws on: the window's, or the handheld's panel (its context has
    /// no window to ask).
    fn drawable_size(&self) -> Option<(i32, i32)> {
        #[cfg(handheld)]
        return self.gl.drawable_size().or(Some(PANEL));
        #[cfg(not(handheld))]
        return self.gl.drawable_size();
    }

    /// The menu's wash (a small picture, 32-bit, blue first) taken as the texture beside the frame.
    pub(super) fn set_wash(&mut self, wash: &[u8], size: (i32, i32)) {
        let gl = &self.gl;
        let texture = match self.wash {
            Some((texture, _)) => texture,
            None => gl.gen_texture(),
        };
        gl.active_texture(TEXTURE0 + 1);
        gl.bind_texture(TEXTURE_2D, texture);
        gl.bind_sampler(1, 0);
        // Linear, and the edges held: the small picture is magnified over the whole window.
        gl.tex_image_2d(0x8058, size.0, size.1, RGBA, UNSIGNED_BYTE, Some(wash));
        gl.tex_parameter(TEXTURE_2D, 0x2801, 0x2601);
        gl.tex_parameter(TEXTURE_2D, 0x2800, 0x2601);
        gl.tex_parameter(TEXTURE_2D, 0x2802, 0x812F);
        gl.tex_parameter(TEXTURE_2D, 0x2803, 0x812F);
        gl.active_texture(TEXTURE0);
        self.wash = Some((texture, size));
    }

    /// The wash on texture unit 1, where the frame's quads read it.
    pub(super) fn bind_wash(&self) {
        if let Some((texture, _)) = self.wash {
            let gl = &self.gl;
            gl.active_texture(TEXTURE0 + 1);
            gl.bind_texture(TEXTURE_2D, texture);
            gl.bind_sampler(1, 0);
            gl.active_texture(TEXTURE0);
        }
    }

    /// The bound picture magnified with no filter (sharp) or a linear one.
    pub(super) fn set_filter(&self, sharp: bool) {
        let filter = if sharp { 0x2600 } else { 0x2601 };
        self.gl.tex_parameter(TEXTURE_2D, 0x2801, filter);
        self.gl.tex_parameter(TEXTURE_2D, 0x2800, filter);
    }

    /// The target on the panel.
    ///
    /// With `wait`, the GPU is waited for before the frame goes to the panel (the GPU-time
    /// diagnostic, `gpu_time`); the wait's time comes back.
    pub(super) fn show_target(&mut self, wait: bool, menu: bool) -> Option<std::time::Duration> {
        self.plain_state();
        let placed = self.panel_viewport(self.width, self.height, menu);
        let gl = &self.gl;
        if let Some(held) = self.frame_high.take() {
            // A menu with its 3D drawn larger: the picture is the merge of it and the frame's.
            self.bind_merge_inputs(held);
            self.set_filter(placed.sharp);
            self.draw_quad(&self.merge_quad, true, Some(placed));
            self.set_filter(true);
        } else {
            gl.bind_sampler(0, 0);
            gl.bind_texture(TEXTURE_2D, self.colour);
            self.set_filter(placed.sharp);
            self.bind_wash();
            self.draw_quad(&self.target_quad, true, Some(placed));
            self.set_filter(true);
        }
        let gl = &self.gl;
        let waited = wait.then(|| {
            let start = std::time::Instant::now();
            gl.finish();
            start.elapsed()
        });
        self.window_shot();
        gl.swap_buffers();
        waited
    }

    /// A picture of the game's bytes (`width` x `height`, blue first) on the panel: the frames the
    /// rasteriser made.
    pub(super) fn show_picture(&mut self, frame: &[u8], width: i32, height: i32, menu: bool) {
        self.upload_canvas(frame, width, height);
        self.plain_state();
        let placed = self.panel_viewport(width, height, menu);
        self.set_filter(placed.sharp);
        self.bind_wash();
        self.draw_quad(&self.canvas_quad, false, Some(placed));
        self.set_filter(true);
        self.window_shot();
        self.gl.swap_buffers();
    }

    /// A test run's shot of the window as it is drawn (`remote`'s `wshot`).
    pub(super) fn window_shot(&self) {
        if !crate::remote::window_shot_due() {
            return;
        }
        let Some((width, height)) = self.drawable_size() else {
            return;
        };
        let mut rgba = vec![0u8; width as usize * height as usize * 4];
        self.gl
            .read_pixels(width, height, RGBA, UNSIGNED_BYTE, &mut rgba);
        // Bottom row first and red first as the GL gives them: top first, blue first.
        let stride = width as usize * 4;
        let mut bgra = Vec::with_capacity(rgba.len());
        for row in rgba.chunks_exact(stride).rev() {
            for pixel in row.as_chunks::<4>().0 {
                bgra.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
            }
        }
        crate::remote::window_taken(width as u32, height as u32, bgra);
    }

    /// The target's rows as the GL holds them.
    pub(super) fn read_target(&mut self) -> Vec<u8> {
        // The frame comes back as its own picture: the larger 3D of the frame so far is dropped.
        self.frame_high = None;
        let (width, height) = (self.width, self.height);
        let mut out = vec![0u8; width as usize * height as usize * 4];
        self.gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        self.gl
            .read_pixels(width, height, RGBA, UNSIGNED_BYTE, &mut out);
        out
    }
}
