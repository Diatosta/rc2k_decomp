//! A frame's draw: the canvas and the batches over the target, and the frame's end.

use super::*;

impl Gpu {
    pub fn draw(
        &mut self,
        canvas: &mut [u8],
        rows: Option<Vec<bool>>,
        commands: &CommandList,
    ) -> Result<(), String> {
        self.take_changes();
        let cuts = crate::render::dev::gpu_cut::live();
        self.note_cuts(cuts);
        let two_d = self.prepare_2d(crate::gfx::draw2d::quads::take())?;
        let mut background_bytes = None;
        let (background, upload_background) = self.bring_up_background(&mut background_bytes)?;
        self.ensure_check()?;
        let stride = self.width as usize * 4;
        let rows = rows
            .unwrap_or_else(|| crate::render::canvas_rows::rows_holding_written(canvas, stride));
        let runs = crate::render::canvas_rows::written_runs(&rows);
        self.check_note_canvas(canvas, &rows, stride, background_bytes.as_deref())?;
        crate::render::dev::seethru::note_list(commands, self.width, self.height);
        let batches = Batches::of(commands, self.width, self.height, None);
        let vertex_bytes = batches.vertices.len() as u32 * VERTEX_BYTES;
        // The 3D of a race at a share of the screen's size: drawn into the small targets and
        // enlarged over the screen (the debug cuts and the check draw it whole).
        let percent = crate::render::flush_scale();
        let smaller = percent != 100
            && !batches.batches.is_empty()
            && !cuts.scene
            && !cuts.half
            && self.check.is_none();
        if smaller {
            self.ensure_small(percent)?;
        }
        // A menu's 3D at the window's size, not the frame's.
        let high = crate::render::menu_high_percent().filter(|_| {
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
                let mut bytes = crate::render::texture_as_bgra(note);
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
        crate::render::GPU_FLUSHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if !parts.is_empty() {
            crate::render::CANVAS_UPLOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.device
                .fill_transfer_parts(self.canvas_transfer, &parts)?;
            let sent: usize = parts.iter().map(|(_, bytes)| bytes.len()).sum();
            crate::render::CANVAS_UPLOAD_BYTES
                .fetch_add(sent as u64, std::sync::atomic::Ordering::Relaxed);
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
        let check_targets = self.check_targets();
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
        if let Some(targets) = check_targets {
            let same_overlay = |pass: &sdl_gpu::RenderPass| {
                if background {
                    self.draw_overlay(pass, self.background);
                }
                for &(first, last) in &runs {
                    let rows = (last + 1 - first) as i32;
                    pass.set_scissor(0, first as i32, self.width as i32, rows);
                    self.draw_overlay(pass, self.canvas);
                }
                pass.set_scissor(0, 0, self.width as i32, self.height as i32);
                self.draw_2d(&buffer, pass, &two_d);
            };
            self.check_pass(&buffer, targets, &same_overlay, &|pass| {
                draw_batches(pass, 100);
            });
        }
        buffer.submit()?;
        for &(first, last) in &runs {
            mark_unwritten(&mut canvas[first * stride..(last + 1) * stride]);
        }
        Ok(())
    }

    /// The frame's end: what the canvas has written goes over the target.
    pub fn finish_frame(
        &mut self,
        rows: Option<Vec<bool>>,
        canvas: &mut [u8],
    ) -> Result<(), String> {
        self.draw(canvas, rows, &CommandList::new())?;
        crate::render::dev::seethru::frame_end();
        // The next frame starts with a fresh depth buffer. The game clears the depth only where
        // it needs the clear, and the GPU keeps the depths between frames: what the last frame
        // wrote (the rear-view mirror's, a car's) would otherwise fail the depth test of the
        // next one in patches.
        self.depth_is_new = true;
        if let Some(small) = self.small.as_mut() {
            small.depth_is_new = true;
        }
        if let Some(high) = self.menu_high.as_mut() {
            high.depth_is_new = true;
        }
        if let Some(check) = self.check.as_mut() {
            check.depth_is_new = true;
        }
        self.compare_check()
    }
}
