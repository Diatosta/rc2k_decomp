//! A frame's drawing on the GL's thread: the canvas, the quads, the batches and the pipeline state.

use super::*;

impl Gpu {
    /// Everything a quad needs off.
    pub(super) fn plain_state(&self) {
        let gl = &self.gl;
        gl.disable(DEPTH_TEST);
        gl.disable(BLEND);
        gl.disable(CULL_FACE);
        gl.disable(SCISSOR_TEST);
        gl.colour_mask(true);
        gl.bind_vertex_array(self.empty_array);
    }

    pub(super) fn draw_quad(&self, quad: &Quad, flip: bool, placed: Option<Placement>) {
        let gl = &self.gl;
        // A program made in `set_up`; three vertices of no attributes.
        gl.use_program(quad.program);
        gl.uniform_1f(quad.flip, if flip { 1.0 } else { 0.0 });
        if let Some(placed) = placed {
            gl.uniform_4fv(quad.fit, &placed.fit);
            gl.uniform_1f(quad.bars, if placed.bars { 1.0 } else { 0.0 });
            // The wash is on unit 1 (`bind_wash`).
            gl.uniform_1i(quad.wash, 1);
            gl.uniform_1f(quad.has_wash, if self.wash.is_some() { 1.0 } else { 0.0 });
        }
        gl.draw_arrays(TRIANGLES, 0, 3);
    }

    /// A frame's part: the textures brought up, the canvas's written rows over the target, then
    /// the batches.
    pub(super) fn draw(&mut self, batched: Batched) {
        let Batched {
            part:
                Part {
                    commands,
                    uploads,
                    written,
                    quads,
                    background,
                    size: (width, height),
                    scale,
                    high,
                },
            batches,
            quad_vertices,
            opaque,
        } = batched;
        self.take_cuts();
        if let Err(why) = self.set_size(width, height) {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput(Box::leak(why.into_boxed_str())),
            );
        }
        let started = std::time::Instant::now();
        for (note, bytes) in &uploads {
            self.bring_texture_up(note, bytes);
        }
        note_if_slow(
            "textures",
            started,
            uploads.iter().map(|(_, b)| b.len()).sum(),
        );
        crate::render::GPU_DRAWS.fetch_add(
            batches.batches.len() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        crate::render::GPU_FLUSHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if !written.is_empty() {
            crate::render::CANVAS_UPLOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        let started = std::time::Instant::now();
        if !written.is_empty() {
            self.next_canvas(width, height);
        }
        for w in &written {
            self.upload_canvas_rows(&w.bytes, width, w.first, w.last);
            crate::render::CANVAS_UPLOAD_BYTES
                .fetch_add(w.bytes.len() as u64, std::sync::atomic::Ordering::Relaxed);
        }
        note_if_slow(
            "canvas rows",
            started,
            written.iter().map(|w| w.bytes.len()).sum(),
        );
        let started = std::time::Instant::now();
        self.send_atlas(&quads.uploads);
        note_if_slow(
            "atlas",
            started,
            quads.uploads.iter().map(|u| u.texels.len() * 4).sum(),
        );
        let started = std::time::Instant::now();
        if let Some(background) = &background {
            self.send_background(background, (width, height));
        }
        note_if_slow("background", started, 0);
        let started = std::time::Instant::now();
        self.send_quad_vertices(&quad_vertices);
        note_if_slow("quad vertices", started, quad_vertices.len() * 24);
        self.plain_state();
        let gl = &self.gl;
        gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        // `half`: a quarter of the target, a quarter of the fragments.
        let (view_width, view_height) = if self.cuts.half {
            (width / 2, height / 2)
        } else {
            (width, height)
        };
        gl.viewport(0, 0, view_width, view_height);
        if self.depth_is_new {
            gl.depth_mask(true);
            gl.clear_depth(1.0);
            gl.clear(DEPTH_BIT);
            self.depth_is_new = false;
        }
        // Up in clip space is the picture's top for the 2D as for the 3D; the object's
        // last row is that top, which `show_target` and `bring_back` turn over.
        if background.is_some() && !self.cuts.two_d {
            self.draw_background((width, height));
        }
        if !written.is_empty() && !self.cuts.two_d {
            gl.enable(SCISSOR_TEST);
            for w in &written {
                gl.bind_texture(TEXTURE_2D, self.canvases[self.canvas_now].texture);
                let rows = (w.last - w.first + 1) as i32;
                gl.scissor(0, height - w.last as i32 - 1, width, rows);
                gl.enable(BLEND);
                gl.blend_func(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
                self.draw_quad(&self.overlay, false, None);
                gl.disable(BLEND);
            }
            gl.disable(SCISSOR_TEST);
        }
        if !self.cuts.two_d {
            self.draw_quads(quad_vertices.len(), (width, height));
        }
        if !batches.batches.is_empty() && !self.cuts.scene {
            if high > 100 && scale == 100 && !self.cuts.half {
                if let Err(why) = self.draw_batches_high(&batches, (width, height), high) {
                    crate::fatal::raise::end(
                        "render::gles",
                        crate::errors::BadInput(Box::leak(why.into_boxed_str())),
                    );
                }
            } else if scale != 100 && !self.cuts.half {
                if let Err(why) = self.draw_batches_small(&batches, (width, height), scale) {
                    crate::fatal::raise::end(
                        "render::gles",
                        crate::errors::BadInput(Box::leak(why.into_boxed_str())),
                    );
                }
            } else {
                self.draw_batches(
                    &batches,
                    width,
                    height,
                    if self.cuts.half { 50 } else { 100 },
                );
            }
        }
        if bench::due(batches.batches.len()) {
            self.bench(&commands, &batches, &opaque, height);
        }
        crate::render::give_back(commands);
        crate::render::batches::give_back_batches(batches);
    }

    /// The part's 3D: its vertices sent up and each batch drawn, by the program for its kind.
    /// The target is bound and the quads' state set (`plain_state`).
    pub(super) fn draw_batches(
        &mut self,
        batches: &Batches,
        width: i32,
        height: i32,
        percent: u32,
    ) {
        let gl = &self.gl;
        gl.bind_vertex_array(self.scene_array);
        gl.bind_buffer(ARRAY_BUFFER, self.vertices);
        gl.buffer_data(ARRAY_BUFFER, &batches.vertices, 0x88E0);
        gl.bind_buffer(ARRAY_BUFFER, self.seconds);
        gl.buffer_data(ARRAY_BUFFER, &batches.seconds, 0x88E0);
        gl.bind_buffer(ARRAY_BUFFER, self.layers);
        gl.buffer_data(ARRAY_BUFFER, &self.vertex_layers(batches), 0x88E0);
        if !self.cuts.depth {
            gl.enable(DEPTH_TEST);
        }
        let cuts = self.cuts;
        let for_any = !kinds_wanted();
        let mut last_pipeline = None;
        // The program in use and what it is for.
        let mut in_use: Option<(ProgramKind, usize)> = None;
        let mut last_second: Option<crate::render::batches::Second> = None;
        let mut free_draws = 0;
        let mut switches = 0;
        let mut last_arrays = None;
        let mut last_sampler = u32::MAX;
        let size = |n: Option<u32>| {
            n.map(|n| {
                let note = &batches.textures[n as usize];
                (note.width, note.height)
            })
        };
        let mut scissored = false;
        if !self.mip_levels
            && batches
                .batches
                .iter()
                .any(|b| b.mips || b.second.is_some_and(|s| s.mips))
        {
            self.level_arrays();
        }
        for batch in &batches.batches {
            if cuts.drops(batch.pipeline.blend.is_some()) {
                continue;
            }
            if last_pipeline != Some(batch.pipeline) {
                self.set_pipeline(&batch.pipeline, last_pipeline.as_ref());
                last_pipeline = Some(batch.pipeline);
            }
            let kind = match in_use {
                // A depth clear draws nothing a program for every kind decides once its values
                // are set: it keeps the one in use.
                Some((kind, _)) if for_any && batch.scissor.is_some() => kind,
                _ => ProgramKind::of(batch, for_any),
            };
            let number = match in_use {
                Some((was, number)) if was == kind => number,
                _ => {
                    let number = self.program_number(cuts, kind);
                    self.gl.use_program(self.programs[number].name);
                    switches += u64::from(in_use.is_some());
                    in_use = Some((kind, number));
                    number
                }
            };
            let gl = &self.gl;
            let program = &mut self.programs[number];
            if program.size != Some((width, height)) {
                gl.uniform_2f(program.locations.size, width as f32, height as f32);
                program.size = Some((width, height));
            }
            if batch.scissor.is_none() {
                free_draws += u64::from(kind.discard_free());
            }
            set_values(
                gl,
                &program.locations.first,
                &mut program.first,
                &batch.values,
            );
            if let Some(second) = &batch.second {
                set_values(
                    gl,
                    &program.locations.second,
                    &mut program.second,
                    &second.values,
                );
            }
            // The second pass's textures are bound whichever program is in use.
            if let Some(second) = &batch.second
                && last_second != Some(*second)
            {
                if let Some(size) = size(second.texture) {
                    self.bind_arrays(2, size);
                    gl.active_texture(TEXTURE0);
                }
                let sampler = self.samplers
                    [second.linear as usize + 2 * second.clamp as usize + 4 * second.mips as usize];
                gl.bind_sampler(2, sampler);
                gl.bind_sampler(3, sampler);
                last_second = Some(*second);
            }
            // A batch's textures are all read from the arrays its first one's size names.
            if let Some(size) = size(batch.texture)
                && last_arrays != Some(size)
            {
                self.bind_arrays(0, size);
                gl.active_texture(TEXTURE0);
                last_arrays = Some(size);
            }
            let sampler = self.samplers
                [batch.linear as usize + 2 * batch.clamp as usize + 4 * batch.mips as usize];
            if sampler != last_sampler {
                gl.bind_sampler(0, sampler);
                gl.bind_sampler(1, sampler);
                last_sampler = sampler;
            }
            if let Some([left, top, right, bottom]) = batch.scissor {
                gl.enable(SCISSOR_TEST);
                // The rectangle is scaled with the viewport (`half`, a smaller 3D): its place from
                // the bottom, in the scaled picture's rows.
                let p = i64::from(percent);
                let down = |n: i32| (i64::from(n) * p / 100) as i32;
                let up = |n: i32| ((i64::from(n) * p + 99) / 100) as i32;
                let rows = up(height);
                let (x0, y0) = (down(left as i32), rows - up(bottom as i32));
                gl.scissor(
                    x0,
                    y0,
                    up(right as i32) - x0,
                    up(bottom as i32) - down(top as i32),
                );
                scissored = true;
            } else if scissored {
                gl.disable(SCISSOR_TEST);
                scissored = false;
            }
            gl.draw_arrays(TRIANGLES, batch.first_vertex as i32, batch.vertices as i32);
        }
        let gl = &self.gl;
        for unit in 0..4 {
            gl.bind_sampler(unit, 0);
        }
        crate::render::GPU_FREE_DRAWS.fetch_add(free_draws, std::sync::atomic::Ordering::Relaxed);
        crate::render::GPU_PROGRAM_SWITCHES
            .fetch_add(switches, std::sync::atomic::Ordering::Relaxed);
    }

    /// Each vertex's passes' arrays and layers (`unit_of`, the texture's layer), two
    /// sixteen-bit numbers each, as the scene's attribute 7 reads them.
    pub(super) fn vertex_layers(&self, batches: &Batches) -> Vec<[u32; 2]> {
        let places: Vec<u32> = batches
            .textures
            .iter()
            .map(|note| {
                self.textures.get(&note.pixels).map_or(0, |t| {
                    u32::from(unit_of((t.width, t.height))) | (t.layer as u32) << 16
                })
            })
            .collect();
        let place = |n: u16| {
            if n == NO_TEXTURE {
                0
            } else {
                places.get(n as usize).copied().unwrap_or(0)
            }
        };
        batches
            .vertex_textures
            .iter()
            .map(|&[first, second]| [place(first), place(second)])
            .collect()
    }

    /// The key's states that differ from `was`'s (all of them without one).
    pub(super) fn set_pipeline(&self, key: &PipelineKey, was: Option<&PipelineKey>) {
        let gl = &self.gl;
        let change = key.changes_from(was);
        if change.cull {
            match key.cull_face() {
                Some(face) => {
                    if was.is_none_or(|w| w.cull_face().is_none()) {
                        gl.enable(CULL_FACE);
                    }
                    gl.cull_face(face);
                }
                None => gl.disable(CULL_FACE),
            }
        }
        if change.depth_function {
            gl.depth_func(key.depth_func());
        }
        if change.depth_write {
            gl.depth_mask(key.depth_write);
        }
        if change.blend {
            match key.blend {
                // `blend` cut: never on.
                Some(_) if self.cuts.blend => gl.disable(BLEND),
                Some((source, target)) => {
                    if was.is_none_or(|w| w.blend.is_none()) {
                        gl.enable(BLEND);
                    }
                    gl.blend_func(blend_factor(source), blend_factor(target));
                }
                None => gl.disable(BLEND),
            }
        }
        if change.colour {
            gl.colour_mask(key.colour);
        }
    }
}
