//! `RC2K_GL_BENCH=<n>`: a measurement of what the GL driver spends per draw. Every n-th part
//! of at least `LEAST` batches (six times at most) is drawn again in several ways, each
//! `ROUNDS` times between two `glFinish`es, and the times logged: as drawn, then with one kind
//! of state change left out at a time. The picture of those frames is wrong (drawn over
//! itself); the measurement is the point: it shows what the handheld's libmali spends its time

use super::*;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use std::time::Instant;

const LEAST: usize = 300;
const ROUNDS: u32 = 10;
const TIMES: u32 = 6;

/// How the pipeline's changes are made.
#[derive(Clone, Copy, PartialEq)]
enum Pipeline {
    /// Every state of the key, as `set_pipeline`.
    Whole,
    /// Only the states that differ from the last key's.
    Changed,
    /// The depth mask alone (the rest as the first batch's).
    DepthMask,
    /// Blending on / off and its factors alone.
    Blend,
    /// The first batch's, kept.
    Kept,
}

#[derive(Clone, Copy)]
struct Way {
    name: &'static str,
    pipeline: Pipeline,
    textures: bool,
    samplers: bool,
    values: bool,
    /// All the triangles in one draw.
    one_draw: bool,
}

const AS_DRAWN: Way = Way {
    name: "as drawn",
    pipeline: Pipeline::Whole,
    textures: true,
    samplers: true,
    values: true,
    one_draw: false,
};

const WAYS: [Way; 10] = [
    AS_DRAWN,
    Way {
        name: "pipeline by changed state",
        pipeline: Pipeline::Changed,
        ..AS_DRAWN
    },
    Way {
        name: "no texture binds",
        textures: false,
        ..AS_DRAWN
    },
    Way {
        name: "no sampler binds",
        samplers: false,
        ..AS_DRAWN
    },
    Way {
        name: "no uniforms",
        values: false,
        ..AS_DRAWN
    },
    Way {
        name: "no pipeline changes",
        pipeline: Pipeline::Kept,
        ..AS_DRAWN
    },
    Way {
        name: "draws and depth mask only",
        pipeline: Pipeline::DepthMask,
        textures: false,
        samplers: false,
        values: false,
        one_draw: false,
    },
    Way {
        name: "draws and blend only",
        pipeline: Pipeline::Blend,
        textures: false,
        samplers: false,
        values: false,
        one_draw: false,
    },
    Way {
        name: "draws only",
        pipeline: Pipeline::Kept,
        textures: false,
        samplers: false,
        values: false,
        one_draw: false,
    },
    Way {
        name: "one draw",
        pipeline: Pipeline::Kept,
        textures: false,
        samplers: false,
        values: false,
        one_draw: true,
    },
];

/// Whether this part (of `batches` batches) is to be measured.
pub(super) fn due(batches: usize) -> bool {
    static EVERY: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    static SEEN: AtomicU32 = AtomicU32::new(0);
    static DONE: AtomicU32 = AtomicU32::new(0);
    let Some(every) = *EVERY.get_or_init(|| {
        std::env::var("RC2K_GL_BENCH")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .filter(|&n| n > 0)
    }) else {
        return false;
    };
    if batches < LEAST || DONE.load(Relaxed) >= TIMES {
        return false;
    }
    let seen = SEEN.fetch_add(1, Relaxed) + 1;
    if !seen.is_multiple_of(every) {
        return false;
    }
    DONE.fetch_add(1, Relaxed);
    true
}

impl Gpu {
    /// Draws `batches` (whose vertices `draw` has just sent up) in each of the `WAYS`, and logs
    /// the time each takes.
    pub(super) fn bench(&mut self, commands: &[Command], batches: &Batches, height: i32) {
        self.bench_gathering(commands, batches);
        let changes =
            |f: &dyn Fn(&super::super::batches::Batch, &super::super::batches::Batch) -> bool| {
                batches
                    .batches
                    .windows(2)
                    .filter(|p| f(&p[0], &p[1]))
                    .count()
            };
        log(&format!(
            "gles bench: {} batches, {} vertices; changes: texture {}, sampler {}, values {}, depth mask {}, blend {}, other pipeline {}",
            batches.batches.len(),
            batches.vertices.len(),
            changes(&|a, b| a.texture != b.texture),
            changes(&|a, b| (a.linear, a.clamp) != (b.linear, b.clamp)),
            changes(&|a, b| a.values != b.values),
            changes(&|a, b| a.pipeline.depth_write != b.pipeline.depth_write),
            changes(&|a, b| a.pipeline.blend != b.pipeline.blend),
            changes(&|a, b| {
                (
                    a.pipeline.cull,
                    a.pipeline.depth_function,
                    a.pipeline.colour,
                ) != (
                    b.pipeline.cull,
                    b.pipeline.depth_function,
                    b.pipeline.colour,
                )
            }),
        ));
        let mut line = String::from("gles bench: ms a frame (issued / finished):");
        for way in WAYS {
            let (issued, ms) = self.timed(|| self.draw_in(batches, height, way));
            line += &format!(" {}: {issued:.1} / {ms:.1};", way.name);
        }
        log(&line);
        self.bench_buffers(batches, height);
    }

    /// `f` `ROUNDS` times: the milliseconds a round until the calls are made, and until the
    /// GPU has done them.
    fn timed(&self, mut f: impl FnMut()) -> (f64, f64) {
        self.gl.finish();
        let start = Instant::now();
        for _ in 0..ROUNDS {
            f();
        }
        let issued = start.elapsed().as_secs_f64();
        self.gl.finish();
        let done = start.elapsed().as_secs_f64();
        let per = |t: f64| t * 1000.0 / ROUNDS as f64;
        (per(issued), per(done))
    }

    /// Whether a draw's cost is the buffer's: as many draws as the frame's batches, of one
    /// triangle, from the frame's buffer and from a small one made once; and the frame's
    /// draws from its vertices sent as a static buffer instead of a stream.
    fn bench_buffers(&self, batches: &Batches, height: i32) {
        let gl = &self.gl;
        let n = batches.batches.len();
        let mut line = String::from("gles bench: buffers (issued / finished):");
        let (issued, ms) = self.timed(|| {
            for _ in 0..n {
                gl.draw_arrays(TRIANGLES, 0, 3);
            }
        });
        line += &format!(" {n} one-triangle draws, frame's buffer: {issued:.1} / {ms:.1};");
        let array = gl.gen_vertex_array();
        let buffer = gl.gen_buffer();
        gl.bind_vertex_array(array);
        gl.bind_buffer(ARRAY_BUFFER, buffer);
        gl.buffer_data_first(ARRAY_BUFFER, &batches.vertices, 96, 0x88E4);
        for (n, count, kind, normal, offset) in [
            (0u32, 4, 0x1406u32, false, 0usize),
            (1, 4, UNSIGNED_BYTE, true, 16),
            (2, 4, UNSIGNED_BYTE, true, 20),
            (3, 2, 0x1406, false, 24),
        ] {
            gl.attribute(n, count, kind, normal, 32, offset);
        }
        let (issued, ms) = self.timed(|| {
            for _ in 0..n {
                gl.draw_arrays(TRIANGLES, 0, 3);
            }
        });
        line += &format!(" the same from a small static buffer: {issued:.1} / {ms:.1};");
        gl.bind_vertex_array(self.scene_array);
        gl.bind_buffer(ARRAY_BUFFER, self.vertices);
        gl.buffer_data(ARRAY_BUFFER, &batches.vertices, 0x88E4);
        let (issued, ms) = self.timed(|| self.draw_in(batches, height, WAYS[0]));
        line += &format!(" as drawn from a static buffer: {issued:.1} / {ms:.1};");
        let (issued, ms) = self.timed(|| self.draw_in(batches, height, WAYS[8]));
        line += &format!(" draws only from it: {issued:.1} / {ms:.1};");
        gl.bind_vertex_array(array);
        gl.bind_buffer(ARRAY_BUFFER, buffer);
        let (issued, ms) = self.timed(|| gl.draw_arrays(TRIANGLES, 0, 3));
        line += &format!(" one triangle alone: {issued:.2} / {ms:.2};");
        gl.bind_vertex_array(self.scene_array);
        gl.bind_buffer(ARRAY_BUFFER, self.vertices);
        gl.delete_buffer(buffer);
        gl.delete_vertex_array(array);
        log(&line);
    }

    /// The context is current, with the scene's program, array and vertices bound by `draw`.
    fn draw_in(&self, batches: &Batches, height: i32, way: Way) {
        let gl = &self.gl;
        let Some(first) = batches.batches.first() else {
            return;
        };
        self.set_pipeline(&first.pipeline);
        if way.one_draw {
            gl.draw_arrays(TRIANGLES, 0, batches.vertices.len() as i32);
            return;
        }
        let mut last: Option<&super::super::batches::Batch> = None;
        let mut scissored = false;
        for batch in &batches.batches {
            let changed = |f: &dyn Fn(&super::super::batches::Batch) -> bool| {
                last.is_none_or(|l| f(l) != f(batch))
            };
            let key = &batch.pipeline;
            match way.pipeline {
                Pipeline::Whole => {
                    if last.is_none_or(|l| l.pipeline != *key) {
                        self.set_pipeline(key);
                    }
                }
                Pipeline::Changed => {
                    if changed(&|b| b.pipeline.cull != 1) || changed(&|b| b.pipeline.cull == 2) {
                        self.set_pipeline(key);
                    } else {
                        if changed(&|b| b.pipeline.depth_function != 0)
                            && last.is_some_and(|l| l.pipeline.depth_function != key.depth_function)
                        {
                            gl.depth_func(match key.depth_function {
                                f @ 1..=7 => 0x0200 + f - 1,
                                _ => 0x0207,
                            });
                        }
                        if last.is_some_and(|l| l.pipeline.depth_write != key.depth_write) {
                            gl.depth_mask(key.depth_write);
                        }
                        if last.is_some_and(|l| l.pipeline.blend.is_some() != key.blend.is_some()) {
                            if key.blend.is_some() {
                                gl.enable(BLEND);
                            } else {
                                gl.disable(BLEND);
                            }
                        }
                        if let Some((source, target)) = key.blend
                            && last.is_some_and(|l| l.pipeline.blend != key.blend)
                        {
                            gl.blend_func(blend_factor(source), blend_factor(target));
                        }
                        if last.is_some_and(|l| l.pipeline.colour != key.colour) {
                            gl.colour_mask(key.colour);
                        }
                    }
                }
                Pipeline::DepthMask => {
                    if last.is_some_and(|l| l.pipeline.depth_write != key.depth_write) {
                        gl.depth_mask(key.depth_write);
                    }
                }
                Pipeline::Blend => {
                    if last.is_some_and(|l| l.pipeline.blend != key.blend) {
                        match key.blend {
                            Some((source, target)) => {
                                gl.enable(BLEND);
                                gl.blend_func(blend_factor(source), blend_factor(target));
                            }
                            None => gl.disable(BLEND),
                        }
                    }
                }
                Pipeline::Kept => {}
            }
            if way.values
                && changed(&|b| b.values != first.values)
                    | last.is_some_and(|l| l.values != batch.values)
            {
                let v = &batch.values;
                let l = &self.locations.first;
                gl.uniform_4fv(l.fog_colour, &v.fog_colour);
                gl.uniform_1f(l.alpha_reference, v.alpha_reference);
                gl.uniform_1i(l.alpha_function, v.alpha_function);
                gl.uniform_1i(l.mixing, v.mixing);
                gl.uniform_1i(l.flags, v.flags);
            }
            if way.textures
                && last.is_none_or(|l| l.texture != batch.texture)
                && let Some(n) = batch.texture
            {
                let note = &batches.textures[n as usize];
                self.bind_arrays(0, (note.width, note.height));
                gl.active_texture(TEXTURE0);
            }
            if way.samplers
                && last.is_none_or(|l| (l.linear, l.clamp) != (batch.linear, batch.clamp))
            {
                let sampler = self.samplers[batch.linear as usize + 2 * batch.clamp as usize];
                gl.bind_sampler(0, sampler);
                gl.bind_sampler(1, sampler);
            }
            if let Some([left, top, right, bottom]) = batch.scissor {
                gl.enable(SCISSOR_TEST);
                gl.scissor(
                    left as i32,
                    height - bottom as i32,
                    (right - left) as i32,
                    (bottom - top) as i32,
                );
                scissored = true;
            } else if scissored {
                gl.disable(SCISSOR_TEST);
                scissored = false;
            }
            gl.draw_arrays(TRIANGLES, batch.first_vertex as i32, batch.vertices as i32);
            last = Some(batch);
        }
        if scissored {
            gl.disable(SCISSOR_TEST);
        }
        gl.bind_sampler(0, 0);
        gl.bind_sampler(1, 0);
    }
}

impl Gpu {
    /// How many draws the frame takes with its textures as layers, against one texture a
    /// draw (`Batches::of`).
    fn bench_gathering(&self, commands: &[Command], batches: &Batches) {
        let opaque = |note: &TextureNote| self.textures.get(&note.pixels).is_some_and(|t| t.opaque);
        let alone = Batches::of(
            commands,
            self.width as u32,
            self.height as u32,
            Some(&opaque),
        );
        log(&format!(
            "gles bench: gathering: {} draws with the textures as layers, {} with one texture a draw",
            batches.batches.len(),
            alone.batches.len(),
        ));
    }
}
