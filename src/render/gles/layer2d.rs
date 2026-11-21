//! The handheld GPU's 2D layer: the atlas texture glyphs and sprites are drawn from, the quads
//! noted for a part (`gfx::draw2d::quads`), and the retained background drawn under the canvas's
//! rows. The same pictures as `render/gpu/quads.rs` makes: the vertices are the same records
//! (`quads::vertices`), the shader reads the atlas the same way.

use super::{ARRAY_BUFFER, Gpu, RGBA, TEXTURE_2D, TEXTURE0, TRIANGLES, UNSIGNED_BYTE};
use crate::gfx::draw2d::quads::{ATLAS_SIDE, PALETTE_ATLAS_ROWS, Upload, VERTEX_WORDS};
use crate::render::gl_batching::Background;
use std::ffi::CStr;

const FLOAT: u32 = 0x1406;
const RGBA8: i32 = 0x8058;
const NEAREST: i32 = 0x2600;
const MIN_FILTER: u32 = 0x2801;
const MAG_FILTER: u32 = 0x2800;
const STREAM_DRAW: u32 = 0x88E0;

/// A rectangle of the atlas at whole pixels of the screen: the corner's place on the screen in
/// pixels and its atlas texel, and the first texel of its palette (x of at most -2: a solid
/// colour).
const QUAD_VS: &CStr = c"#version 300 es
layout(location=0) in vec4 corner;
layout(location=1) in vec2 palette;
uniform vec2 size;
out highp vec2 v_texel;
flat out highp vec2 v_palette;
void main() {
    gl_Position = vec4(corner.x / size.x * 2.0 - 1.0, 1.0 - corner.y / size.y * 2.0, 0.0, 1.0);
    v_texel = corner.zw;
    v_palette = palette;
}
";

/// The atlas' texel of the rectangle's pixel (`quad.frag`): a gap (alpha 0) draws nothing; a
/// palette glyph's texel holds an index, looked up in the palette's row of the palettes texture (the atlas' first rows, kept apart: they are rewritten every frame the highlighted font pulses, and writing a texture the last frame still reads stalls the driver); a
/// direct colour is the texel; a solid rectangle's colour is in its palette. The atlas holds the
/// game's bytes, blue first, and the target red first. The values reach 2048, which half
/// precision cannot tell apart: all of it is `highp`.
const QUAD_FS: &CStr = c"#version 300 es
precision highp float;
precision highp int;
in highp vec2 v_texel;
flat in highp vec2 v_palette;
out vec4 result;
uniform highp sampler2D atlas;
uniform highp sampler2D palettes;
void main() {
    if (v_palette.x <= -2.0) {
        int colour = int(-v_palette.x) - 2;
        result = vec4(float(colour >> 8) / 255.0, float(colour & 255) / 255.0,
                      v_palette.y / 255.0, 1.0);
        return;
    }
    vec4 value = texelFetch(atlas, ivec2(floor(v_texel)), 0).bgra;
    if (value.a < 0.5) discard;
    if (v_palette.x >= 0.0) {
        int index = int(value.r * 255.0 + 0.5);
        value = texelFetch(palettes, ivec2(int(v_palette.x) + index, int(v_palette.y)), 0).bgra;
    }
    result = vec4(value.rgb, 1.0);
}
";

/// What the 2D layer keeps in the GL: names, and where the screen-sized background stands.
#[derive(Default)]
pub(super) struct Layer2d {
    program: u32,
    /// The program's `size` uniform.
    size: i32,
    /// The atlas texture, `ATLAS_SIDE` square.
    atlas: u32,
    /// The palettes' rows, `ATLAS_SIDE` wide.
    palettes: u32,
    /// The attributes' layout over `vertices`.
    array: u32,
    vertices: u32,
    /// The retained background's texture, and the size it was made at.
    /// Two, taken in turn: a new picture is written to the one the last frame did not read.
    background: [u32; 2],
    /// The one drawn.
    background_now: usize,
    background_size: (i32, i32),
}

impl Gpu {
    /// The program, the atlas and the vertex buffer, at start-up: a shader the driver refuses
    /// stops the backend there, not at the first menu.
    pub(super) fn make_layer2d(&self) -> Result<Layer2d, String> {
        let program = self.program(QUAD_VS, QUAD_FS)?;
        let gl = &self.gl;
        gl.use_program(program);
        // The atlas is read from unit 0.
        gl.uniform_1i(self.uniform(program, "atlas"), 0);
        gl.uniform_1i(self.uniform(program, "palettes"), 1);
        let atlas = gl.gen_texture();
        gl.bind_texture(TEXTURE_2D, atlas);
        gl.tex_image_2d(
            RGBA8,
            ATLAS_SIDE as i32,
            ATLAS_SIDE as i32,
            RGBA,
            UNSIGNED_BYTE,
            None,
        );
        // One level only, read by `texelFetch`: no filtering to ask for.
        gl.tex_parameter(TEXTURE_2D, MIN_FILTER, NEAREST);
        gl.tex_parameter(TEXTURE_2D, MAG_FILTER, NEAREST);
        let palettes = gl.gen_texture();
        gl.bind_texture(TEXTURE_2D, palettes);
        gl.tex_image_2d(
            RGBA8,
            ATLAS_SIDE as i32,
            PALETTE_ATLAS_ROWS as i32,
            RGBA,
            UNSIGNED_BYTE,
            None,
        );
        gl.tex_parameter(TEXTURE_2D, MIN_FILTER, NEAREST);
        gl.tex_parameter(TEXTURE_2D, MAG_FILTER, NEAREST);
        let (array, vertices) = (gl.gen_vertex_array(), gl.gen_buffer());
        gl.bind_vertex_array(array);
        gl.bind_buffer(ARRAY_BUFFER, vertices);
        let stride = (VERTEX_WORDS * 4) as i32;
        gl.attribute(0, 4, FLOAT, false, stride, 0);
        gl.attribute(1, 2, FLOAT, false, stride, 16);
        Ok(Layer2d {
            program,
            size: self.uniform(program, "size"),
            atlas,
            palettes,
            array,
            vertices,
            background: [gl.gen_texture(), gl.gen_texture()],
            background_now: 0,
            background_size: (0, 0),
        })
    }

    /// The atlas texels waiting, into the atlas, in the order they were made.
    pub(super) fn send_atlas(&self, uploads: &[Upload]) {
        if uploads.is_empty() {
            return;
        }
        let gl = &self.gl;
        gl.active_texture(TEXTURE0);
        gl.bind_sampler(0, 0);
        for upload in uploads {
            // Palette rows are above the glyphs' shelves.
            let palette = upload.y < PALETTE_ATLAS_ROWS;
            let texture = if palette {
                self.layer2d.palettes
            } else {
                self.layer2d.atlas
            };
            gl.bind_texture(TEXTURE_2D, texture);
            gl.tex_sub_image_2d(
                (upload.x as i32, upload.y as i32),
                (upload.width as i32, upload.height as i32),
                RGBA,
                UNSIGNED_BYTE,
                upload.texels.as_flattened(),
            );
        }
    }

    /// The background's picture (`bytes`: the screen's rows) into its texture, made first at a
    /// new size.
    pub(super) fn send_background(&mut self, background: &Background, (width, height): (i32, i32)) {
        let Background::Upload(bytes) = background else {
            return;
        };
        let gl = &self.gl;
        gl.active_texture(TEXTURE0);
        gl.bind_sampler(0, 0);
        if self.layer2d.background_size != (width, height) {
            // Both made at the new size, with no pixels.
            for texture in self.layer2d.background {
                gl.bind_texture(TEXTURE_2D, texture);
                gl.tex_image_2d(RGBA8, width, height, RGBA, UNSIGNED_BYTE, None);
                gl.tex_parameter(TEXTURE_2D, MIN_FILTER, NEAREST);
                gl.tex_parameter(TEXTURE_2D, MAG_FILTER, NEAREST);
            }
            self.layer2d.background_size = (width, height);
        }
        let next = 1 - self.layer2d.background_now;
        gl.bind_texture(TEXTURE_2D, self.layer2d.background[next]);
        gl.tex_sub_image_2d((0, 0), (width, height), RGBA, UNSIGNED_BYTE, bytes);
        self.layer2d.background_now = next;
        crate::render::BACKGROUND_UPLOAD_BYTES
            .fetch_add(bytes.len() as u64, std::sync::atomic::Ordering::Relaxed);
    }

    /// The quads' vertices into their buffer.
    pub(super) fn send_quad_vertices(&self, vertices: &[[u32; VERTEX_WORDS]]) {
        if vertices.is_empty() {
            return;
        }
        let gl = &self.gl;
        gl.bind_vertex_array(self.layer2d.array);
        gl.bind_buffer(ARRAY_BUFFER, self.layer2d.vertices);
        gl.buffer_data(ARRAY_BUFFER, vertices, STREAM_DRAW);
    }

    /// The background's pixels over the target, in the state `plain_state` leaves: the written
    /// ones, as the canvas's.
    pub(super) fn draw_background(&self, (width, height): (i32, i32)) {
        if self.layer2d.background_size != (width, height) {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput("the retained background is not of the screen's size"),
            );
        }
        let gl = &self.gl;
        gl.active_texture(TEXTURE0);
        gl.bind_texture(
            TEXTURE_2D,
            self.layer2d.background[self.layer2d.background_now],
        );
        gl.bind_sampler(0, 0);
        self.draw_quad(&self.overlay, false, None);
    }

    /// `vertices` quad vertices drawn over the target, in the state `plain_state` leaves.
    pub(super) fn draw_quads(&self, vertices: usize, (width, height): (i32, i32)) {
        if vertices == 0 {
            return;
        }
        let gl = &self.gl;
        gl.use_program(self.layer2d.program);
        gl.uniform_2f(self.layer2d.size, width as f32, height as f32);
        gl.active_texture(TEXTURE0);
        gl.bind_texture(TEXTURE_2D, self.layer2d.atlas);
        gl.bind_sampler(0, 0);
        gl.active_texture(TEXTURE0 + 1);
        gl.bind_texture(TEXTURE_2D, self.layer2d.palettes);
        gl.bind_sampler(1, 0);
        gl.active_texture(TEXTURE0);
        gl.bind_vertex_array(self.layer2d.array);
        gl.draw_arrays(TRIANGLES, 0, vertices as i32);
    }
}
