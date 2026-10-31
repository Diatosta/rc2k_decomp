//! The HUD's variables, out of RAL.EXE's image (phase 5; typed in phase 7,
//! `notes/conversion.md`): IDA 0x75B940 .. 0x764F2C, the whole of it the HUD's: the
//! rectangle table (`g_hudRects`), the quad record, the layouts' set-up values and floats,
//! the dials' and needles' work variables, the batch's vertices (`unk_75C324`, 0x400 of
//! them), indices (`unk_764324`, 0x600) and the two counts (`g_hudVertexCount`,
//! `g_hudIndexCount`). Only `hud` and `d3d::draw_hud_batch` (which reads `vertices` and
//! `indices`) read or write it. The image keeps the two pointers (0x764F2C, 0x764F30) the
//! original's batch drawer followed; nothing reads them now.
//!
//! One `#[repr(C)]` struct with the image's layout (`Hud`, the static `HUD`); the first
//! values are the image's (`tests::first_values_are_the_games`), and `moved::MOVED` points
//! IDA's addresses of the range at it.

use crate::session::{Dword, Float, Word};

/// IDA's address of the block's first byte.
pub const START: usize = 0x75B940;
/// The block's length.
pub const LEN: usize = 0x764F2C - START;

/// One of the texture's rectangles (`g_hudRects`).
#[repr(C)]
pub struct Rect {
    pub left: Dword,
    pub top: Dword,
    pub width: Dword,
    pub height: Dword,
}

impl Rect {
    const fn new(left: u32, top: u32, width: u32, height: u32) -> Self {
        Self {
            left: Dword::new(left),
            top: Dword::new(top),
            width: Dword::new(width),
            height: Dword::new(height),
        }
    }
}

/// A batch vertex (Direct3D's transformed and lit vertex, 0x20 bytes).
#[repr(C)]
pub struct Vertex {
    pub x: Float,
    pub y: Float,
    pub z: Float,
    pub rhw: Float,
    pub colour: Dword,
    pub specular: Dword,
    pub u: Float,
    pub v: Float,
}

impl Vertex {
    const fn zero() -> Self {
        Self {
            x: Float::from_bits(0),
            y: Float::from_bits(0),
            z: Float::from_bits(0),
            rhw: Float::from_bits(0),
            colour: Dword::new(0),
            specular: Dword::new(0),
            u: Float::from_bits(0),
            v: Float::from_bits(0),
        }
    }
}

/// The HUD's variables (IDA 0x75B940 .. 0x764F2C, the image's layout).
#[repr(C)]
pub struct Hud {
    /// `g_hudRects` (92 rectangles) and `dword_75BF00` (the last five): the texture's rectangles, 16 bytes each (left, top, width, height), picked by number (`hud::layout_*` sprites, markers, signs). Sign `v` is rectangle 0x4C + (v - 1) & 0xF.
    pub rects: [Rect; 0x61],
    /// `dword_75BF50`, `dword_75BF54` (the stage marker's two colours, `draw_markers_1`) and the six dwords after them (colours too, a reading; nothing ported reads them).
    pub marker_colours: [Dword; 8],
    /// `dword_75BF70`: the stage bar's four part colours, read at the byte offset the kind table gives (`draw_stage_bar`).
    pub stage_colours: [Dword; 4],
    /// `dword_75BF80`: the widths of the eight signs (`draw_signs`: centres a sign in its place).
    pub sign_widths: [Dword; 8],
    /// `dword_75BFA0`: the part colour of the one-player stage bar.
    pub bar_colour: Dword,
    /// `dword_75BFA4`, `dword_75BFA8`: the part colours of the two-player stage bars.
    pub bar_colours: [Dword; 2],
    /// `dword_75BFAC`: the needle's way (the record's +0xCC / +0xD0; 0 or not, `draw_needle`).
    pub needle_kind: Dword,
    /// `g_hudQuad` (`dword_75BFB0` .. `dword_75BFE4`): the quad record `add_quad` makes four vertices of: alpha, left, top, right, bottom, colour, then the four corners' texture places (whole numbers, `rc2k_core::hud::quads::RECORD` dwords).
    pub quad: [Dword; 14],
    /// `flt_75BFE8`: the batch's depth (z of every vertex, `set_batch_depth`).
    pub depth: Float,
    /// `flt_75BFEC`: the batch's rhw (`set_batch_depth`).
    pub rhw: Float,
    /// `dword_75BFF0`, `dword_75BFF4`: the stage bar's part places (whole numbers) and a turned shape's point (floats' bits, `turned_vertex`).
    pub point: [Dword; 2],
    /// `dword_75BFF8`, `dword_75BFFC`: the stage bar's part start and end (miles, as floats).
    pub ends: [Float; 2],
    /// `dword_75C000`: the stage bar's width in places.
    pub bar_width: Dword,
    /// `0x75C004`, `0x75C008`: nothing reads or writes them (a gap).
    pub unused_c004: [Dword; 2],
    /// `flt_75C00C`: the texture places over it make the u (`add_quad`, `turned_vertex`, markers).
    pub uv_scale_x: Float,
    /// `flt_75C010`: the texture places over it make the v; also the needle's first turn factor (`draw_needle`).
    pub uv_scale_y: Float,
    /// `flt_75C014`: the needle's second turn factor.
    pub turn_k: Float,
    /// `flt_75C018`: the needle's least value.
    pub least: Float,
    /// `flt_75C01C`: added to a disc entry's texture place (`disc_vertex`).
    pub texture_base: Float,
    /// `flt_75C020`: a disc entry's texture place is divided by it (`disc_vertex`).
    pub texture_size: Float,
    /// `flt_75C024`: the second factor of the disc texture's turn (`draw_disc`).
    pub disc_turn_b: Float,
    /// `flt_75C028`: the first factor of the disc texture's turn (`draw_disc`).
    pub disc_turn_a: Float,
    /// `flt_75C02C`, `flt_75C030`: the virtual screen's width and height the HUD is drawn in (`add_polygon` scales to the real one).
    pub virtual_screen: [Float; 2],
    /// `dword_75C034`: the texture asked for.
    pub texture: Dword,
    /// `dword_75C038`: the batch's texture (-1 none).
    pub batch_texture: Dword,
    /// `dword_75C03C`: 1 while the leading digits of a time are 0 (`layout_0::leading_time`).
    pub leading: Dword,
    /// `dword_75C040`: the needle sign's rectangle.
    pub pick: Dword,
    /// `dword_75C044`, `dword_75C048`: the disc's / needle's centre (x, y), copied from a set-up.
    pub disc_centre: [Float; 2],
    /// `dword_75C04C`: the disc entry's texture scale (a reading).
    pub disc_texture_scale: Float,
    /// `dword_75C050`: the disc's size (the ring entries are times it).
    pub disc_size: Float,
    /// `dword_75C054`, `dword_75C058`: the disc texture's shift (u, v), from the record.
    pub disc_texture_shift: [Float; 2],
    /// `dword_75C05C`: the disc texture's turn, from the record (+0x3C / +0x64).
    pub disc_turn: Float,
    /// `dword_75C060`: the original's `fistp` scratch (a sprite's height, the needle's turn, the marker's x).
    pub scratch: Dword,
    /// `dword_75C064`, `dword_75C068`: the icon's width and drop, scaled.
    pub icon_size: [Dword; 2],
    /// `dword_75C06C`: the icon scale (from `icon_scales`).
    pub icon_scale: Float,
    /// `dword_75C070`: the icon's shift.
    pub icon_shift: Dword,
    /// `dword_75C074` .. `dword_75C088`: six record times in tenths (`tenths`): the two clocks, the lap time, the two layout-1 clocks (`dword_75C080`, `dword_75C084`) and the second lap time (`dword_75C088`).
    pub times: [Dword; 6],
    /// `dword_75C08C` .. `dword_75C0A0`: a time made digits (`layout_0::time_digits`).
    pub digits: [Dword; 6],
    /// `dword_75C0A4`: a time's sign rectangle.
    pub sign: Dword,
    /// `flt_75C0A8`, `flt_75C0AC`: the stage marker's position factors (`draw_marker`).
    pub marker_k: [Float; 2],
    /// `0x75C0B0`, `0x75C0B4`: nothing reads or writes them (a gap; 0.96875 and 0.03125).
    pub unused_c0b0: [Float; 2],
    /// `dword_75C0B8`, `dword_75C0BC`, `dword_75C0C0`: the icon scale of layouts 0, 2 and 1 (`icons_*`).
    pub icon_scales: [Float; 3],
    /// `dword_75C0C4` .. `0x75C120`: whole numbers in fours (a table of places, a reading); nothing ported reads them.
    pub unused_c0c4: [Dword; 0x18],
    /// `dword_75C124` .. `dword_75C130`: nothing reads or writes them.
    pub unused_c124: [Dword; 4],
    /// `dword_75C134` .. `dword_75C144`: the set-up of layout 2's first disc (copied by `dials::set_up`; the fifth only with `dword_6D1502` 0).
    pub disc_2a: [Float; 5],
    /// `dword_75C148` .. `dword_75C154`: the set-up of layout 2's second disc.
    pub disc_2b: [Float; 4],
    /// `dword_75C158` .. `dword_75C164`: the set-up of layout 1's first disc.
    pub disc_1a: [Float; 4],
    /// `dword_75C168` .. `dword_75C174`: the set-up of layout 1's second disc.
    pub disc_1b: [Float; 4],
    /// `dword_75C178` .. `dword_75C184`: the set-up of layout 0's disc.
    pub disc_0: [Float; 4],
    /// `flt_75C188`: the disc's ring, sines of the entries (1 .. 16 written; entry 0 stays 0).
    pub ring_sines: [Float; 17],
    /// `flt_75C1CC`: the ring's cosines.
    pub ring_cosines: [Float; 17],
    /// `flt_75C210`: the texture ring's sines.
    pub texture_ring_sines: [Float; 17],
    /// `flt_75C254`: the texture ring's cosines.
    pub texture_ring_cosines: [Float; 17],
    /// `flt_75C298`, `flt_75C29C`: added to a turned shape's point to make its texture place (`turned_vertex`).
    pub uv_shift: [Float; 2],
    /// `dword_75C2A0`, `dword_75C2A4`: the needle's centre (x, y), copied from a set-up.
    pub needle_centre: [Float; 2],
    /// `flt_75C2A8` .. `flt_75C2B4`: the needle's corner offsets (`needle_corners`).
    pub needle_corners: [Float; 4],
    /// `dword_75C2B8`: the needle's size, copied from a set-up.
    pub needle_size: Float,
    /// `flt_75C2BC` .. `flt_75C2D4`: the needle's base, base 2, split, scale, scale 2, most, most 2.
    pub needle_k: [Float; 7],
    /// `0x75C2D8`, `0x75C2DC`: nothing reads or writes them (a gap).
    pub unused_c2d8: [Float; 2],
    /// `dword_75C2E0`: the needle's value (a whole number, from the record).
    pub needle_value: Dword,
    /// `dword_75C2E4` .. `dword_75C2EC`: the set-up of layout 0's needle (centre x, y, size).
    pub needle_0: [Float; 3],
    /// `dword_75C2F0` .. `dword_75C2FC`: the set-up of layout 2's first needle (the third only with `dword_6D1502` 0).
    pub needle_2a: [Float; 4],
    /// `dword_75C300` .. `dword_75C308`: the set-up of layout 2's second needle.
    pub needle_2b: [Float; 3],
    /// `dword_75C30C` .. `dword_75C314`: the set-up of layout 1's first needle.
    pub needle_1a: [Float; 3],
    /// `dword_75C318` .. `dword_75C320`: the set-up of layout 1's second needle.
    pub needle_1b: [Float; 3],
    /// `unk_75C324`: the batch's vertices (transformed and lit, 0x20 bytes each).
    pub vertices: [Vertex; 0x400],
    /// `unk_764324`: the batch's indices (16 bits, a triangle list).
    pub indices: [Word; 0x600],
    /// `g_hudVertexCount`: the vertices in the batch.
    pub vertex_count: Dword,
    /// `g_hudIndexCount`: the indices in the batch.
    pub index_count: Dword,
}

/// The HUD's variables; first values the image's.
pub static HUD: Hud = Hud {
    rects: [
        Rect::new(0xad, 0x68, 0x52, 0x2a),
        Rect::new(0xad, 0x94, 0x52, 0x2a),
        Rect::new(0x1, 0x1, 0x2c, 0x23),
        Rect::new(0x2f, 0x1, 0x2c, 0x23),
        Rect::new(0x5d, 0x1, 0x2c, 0x23),
        Rect::new(0x8b, 0x1, 0x2c, 0x23),
        Rect::new(0xb9, 0x1, 0x2c, 0x23),
        Rect::new(0x1, 0x26, 0x2c, 0x23),
        Rect::new(0x2f, 0x26, 0x2c, 0x23),
        Rect::new(0x5d, 0x26, 0x2c, 0x23),
        Rect::new(0x8b, 0x26, 0x2c, 0x23),
        Rect::new(0xb9, 0x26, 0x2c, 0x23),
        Rect::new(0x1, 0x4b, 0x13, 0xf),
        Rect::new(0x16, 0x4b, 0x13, 0xf),
        Rect::new(0x2b, 0x4b, 0x13, 0xf),
        Rect::new(0x40, 0x4b, 0x13, 0xf),
        Rect::new(0x55, 0x4b, 0x13, 0xf),
        Rect::new(0x6a, 0x4b, 0x13, 0xf),
        Rect::new(0x7f, 0x4b, 0x13, 0xf),
        Rect::new(0x94, 0x4b, 0x13, 0xf),
        Rect::new(0xa9, 0x4b, 0x13, 0xf),
        Rect::new(0xbe, 0x4b, 0x13, 0xf),
        Rect::new(0xd3, 0x4b, 0x8, 0xf),
        Rect::new(0xdd, 0x4b, 0x8, 0xf),
        Rect::new(0x4d, 0x5c, 0xc, 0xa),
        Rect::new(0x5b, 0x5c, 0xc, 0xa),
        Rect::new(0x69, 0x5c, 0xc, 0xa),
        Rect::new(0x77, 0x5c, 0xc, 0xa),
        Rect::new(0x85, 0x5c, 0xc, 0xa),
        Rect::new(0x93, 0x5c, 0xc, 0xa),
        Rect::new(0xa1, 0x5c, 0xc, 0xa),
        Rect::new(0xaf, 0x5c, 0xc, 0xa),
        Rect::new(0xbd, 0x5c, 0xc, 0xa),
        Rect::new(0xcb, 0x5c, 0xc, 0xa),
        Rect::new(0xd9, 0x5c, 0xc, 0xa),
        Rect::new(0x73, 0x68, 0xc, 0xa),
        Rect::new(0x81, 0x68, 0x6, 0xa),
        Rect::new(0xdd, 0xc3, 0x6, 0xa),
        Rect::new(0xe7, 0x1, 0x18, 0x20),
        Rect::new(0xe7, 0x23, 0x18, 0x20),
        Rect::new(0xe7, 0x45, 0x18, 0x20),
        Rect::new(0x1, 0x5c, 0x24, 0xd),
        Rect::new(0x1, 0x6b, 0x24, 0xd),
        Rect::new(0x1, 0x7a, 0x24, 0xd),
        Rect::new(0x1, 0x89, 0x24, 0xd),
        Rect::new(0x1, 0x98, 0x24, 0xd),
        Rect::new(0x1, 0xa7, 0x24, 0xd),
        Rect::new(0x1, 0xb6, 0x24, 0xd),
        Rect::new(0x1, 0xc5, 0x24, 0xd),
        Rect::new(0x1, 0xd4, 0x24, 0xd),
        Rect::new(0x1, 0xe3, 0x24, 0xd),
        Rect::new(0x1, 0xf2, 0x24, 0xd),
        Rect::new(0x27, 0x5c, 0x24, 0xd),
        Rect::new(0x27, 0x6b, 0x24, 0xd),
        Rect::new(0x27, 0x7a, 0x24, 0xd),
        Rect::new(0x27, 0x89, 0x24, 0xd),
        Rect::new(0x27, 0x98, 0x24, 0xd),
        Rect::new(0x27, 0xa7, 0x24, 0xd),
        Rect::new(0x27, 0xb6, 0x24, 0xd),
        Rect::new(0x27, 0xc5, 0x24, 0xd),
        Rect::new(0x27, 0xd4, 0x24, 0xd),
        Rect::new(0x27, 0xe3, 0x24, 0xd),
        Rect::new(0x27, 0xf2, 0x24, 0xd),
        Rect::new(0x4d, 0x67, 0x24, 0xd),
        Rect::new(0x4d, 0x77, 0x1e, 0x2c),
        Rect::new(0x6d, 0x77, 0x1e, 0x2c),
        Rect::new(0x8d, 0x77, 0x1e, 0x2c),
        Rect::new(0x4d, 0xa5, 0x1e, 0x2c),
        Rect::new(0x6d, 0xa5, 0x1e, 0x2c),
        Rect::new(0x8d, 0xa5, 0x1e, 0x2c),
        Rect::new(0x4d, 0xd3, 0x1e, 0x2c),
        Rect::new(0x6d, 0xd3, 0x1e, 0x2c),
        Rect::new(0x8d, 0xd3, 0x1e, 0x2c),
        Rect::new(0xad, 0xc3, 0x2e, 0x3c),
        Rect::new(0xdd, 0xdd, 0x18, 0x22),
        Rect::new(0xfc, 0xca, 0x3, 0x35),
        Rect::new(0x1, 0x1, 0x20, 0x3a),
        Rect::new(0x23, 0x1, 0x22, 0x3a),
        Rect::new(0x47, 0x1, 0x2d, 0x3a),
        Rect::new(0x76, 0x1, 0x36, 0x3a),
        Rect::new(0xae, 0x1, 0x38, 0x3a),
        Rect::new(0x1, 0x3d, 0x33, 0x3a),
        Rect::new(0x36, 0x3d, 0x32, 0x3a),
        Rect::new(0x6a, 0x3d, 0x35, 0x3a),
        Rect::new(0x21, 0x1, 0xffffffe0, 0x3a),
        Rect::new(0x45, 0x1, 0xffffffde, 0x3a),
        Rect::new(0x74, 0x1, 0xffffffd3, 0x3a),
        Rect::new(0xac, 0x1, 0xffffffca, 0x3a),
        Rect::new(0xe6, 0x1, 0xffffffc8, 0x3a),
        Rect::new(0x34, 0x3d, 0xffffffcd, 0x3a),
        Rect::new(0x68, 0x3d, 0xffffffce, 0x3a),
        Rect::new(0x9f, 0x3d, 0xffffffcb, 0x3a),
        Rect::new(0xed, 0x11, 0x12, 0x20),
        Rect::new(0xf1, 0x1, 0xe, 0xe),
        Rect::new(0x0, 0x7f, 0x80, 0x80),
        Rect::new(0x82, 0x7f, 0x7e, 0x80),
        Rect::new(0x0, 0x0, 0x80, 0x80),
    ],
    marker_colours: [
        Dword::new(0xffffff),
        Dword::new(0xff0300),
        Dword::new(0xff8d00),
        Dword::new(0xf0ff00),
        Dword::new(0xca1b),
        Dword::new(0x2700ff),
        Dword::new(0x21dcde),
        Dword::new(0x8b00ff),
    ],
    stage_colours: [
        Dword::new(0x212463),
        Dword::new(0xe12626),
        Dword::new(0x32a126),
        Dword::new(0x227edc),
    ],
    sign_widths: [
        Dword::new(0x20),
        Dword::new(0x22),
        Dword::new(0x2d),
        Dword::new(0x36),
        Dword::new(0x38),
        Dword::new(0x33),
        Dword::new(0x32),
        Dword::new(0x35),
    ],
    bar_colour: Dword::new(0x0),
    bar_colours: [const { Dword::new(0x0) }; 2],
    needle_kind: Dword::new(0x0),
    quad: [
        Dword::new(0xc0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
    ],
    depth: Float::from_bits(0x3c23d70a),
    rhw: Float::from_bits(0x3f800000),
    point: [const { Dword::new(0x0) }; 2],
    ends: [const { Float::from_bits(0x0) }; 2],
    bar_width: Dword::new(0x0),
    unused_c004: [const { Dword::new(0x3e000000) }; 2],
    uv_scale_x: Float::from_bits(0x43800000),
    uv_scale_y: Float::from_bits(0x47800000),
    turn_k: Float::from_bits(0x45800000),
    least: Float::from_bits(0x0),
    texture_base: Float::from_bits(0x42400000),
    texture_size: Float::from_bits(0x43000000),
    disc_turn_b: Float::from_bits(0x44800000),
    disc_turn_a: Float::from_bits(0x3f22f983),
    virtual_screen: [Float::from_bits(0x44480000), Float::from_bits(0x44160000)],
    texture: Dword::new(0x0),
    batch_texture: Dword::new(0x0),
    leading: Dword::new(0x0),
    pick: Dword::new(0x0),
    disc_centre: [Float::from_bits(0x44308000), Float::from_bits(0x43fc0000)],
    disc_texture_scale: Float::from_bits(0x3f400000),
    disc_size: Float::from_bits(0x42780000),
    disc_texture_shift: [const { Float::from_bits(0x0) }; 2],
    disc_turn: Float::from_bits(0x0),
    scratch: Dword::new(0x0),
    icon_size: [const { Dword::new(0x0) }; 2],
    icon_scale: Float::from_bits(0x0),
    icon_shift: Dword::new(0x0),
    times: [const { Dword::new(0x0) }; 6],
    digits: [const { Dword::new(0x0) }; 6],
    sign: Dword::new(0x0),
    marker_k: [Float::from_bits(0x44098000), Float::from_bits(0x42ee0000)],
    unused_c0b0: [Float::from_bits(0x3f780000), Float::from_bits(0x3d000000)],
    icon_scales: [
        Float::from_bits(0x3f800000),
        Float::from_bits(0x3f400000),
        Float::from_bits(0x3f800000),
    ],
    unused_c0c4: [
        Dword::new(0x25d),
        Dword::new(0x30),
        Dword::new(0xd6),
        Dword::new(0x24),
        Dword::new(0x25d),
        Dword::new(0x4),
        Dword::new(0x25d),
        Dword::new(0x30),
        Dword::new(0x268),
        Dword::new(0x24),
        Dword::new(0x25d),
        Dword::new(0x141),
        Dword::new(0x25d),
        Dword::new(0x10),
        Dword::new(0xd6),
        Dword::new(0x4),
        Dword::new(0x25d),
        Dword::new(0x4),
        Dword::new(0x25d),
        Dword::new(0x10),
        Dword::new(0x268),
        Dword::new(0x4),
        Dword::new(0x25d),
        Dword::new(0x131),
    ],
    unused_c124: [const { Dword::new(0x0) }; 4],
    disc_2a: [
        Float::from_bits(0x42380000),
        Float::from_bits(0x3f400000),
        Float::from_bits(0x4435c000),
        Float::from_bits(0x43530000),
        Float::from_bits(0x43630000),
    ],
    disc_2b: [
        Float::from_bits(0x42380000),
        Float::from_bits(0x3f400000),
        Float::from_bits(0x4435c000),
        Float::from_bits(0x44038000),
    ],
    disc_1a: [
        Float::from_bits(0x42380000),
        Float::from_bits(0x3f400000),
        Float::from_bits(0x43a30000),
        Float::from_bits(0x44038000),
    ],
    disc_1b: [
        Float::from_bits(0x42380000),
        Float::from_bits(0x3f400000),
        Float::from_bits(0x4435c000),
        Float::from_bits(0x44038000),
    ],
    disc_0: [
        Float::from_bits(0x42780000),
        Float::from_bits(0x3f400000),
        Float::from_bits(0x44308000),
        Float::from_bits(0x43fc0000),
    ],
    ring_sines: [const { Float::from_bits(0x0) }; 17],
    ring_cosines: [const { Float::from_bits(0x0) }; 17],
    texture_ring_sines: [const { Float::from_bits(0x0) }; 17],
    texture_ring_cosines: [const { Float::from_bits(0x0) }; 17],
    uv_shift: [Float::from_bits(0x437d8000), Float::from_bits(0x43728000)],
    needle_centre: [Float::from_bits(0x42c00000), Float::from_bits(0x43fb8000)],
    needle_corners: [
        Float::from_bits(0x40200000),
        Float::from_bits(0x41580000),
        Float::from_bits(0xc0200000),
        Float::from_bits(0xc2260000),
    ],
    needle_size: Float::from_bits(0x0),
    needle_k: [
        Float::from_bits(0x3ec00000),
        Float::from_bits(0x3ee00000),
        Float::from_bits(0x457a0000),
        Float::from_bits(0x45dac000),
        Float::from_bits(0x45fa0000),
        Float::from_bits(0x477a0000),
        Float::from_bits(0x46147000),
    ],
    unused_c2d8: [Float::from_bits(0x453b8000), Float::from_bits(0x473b8000)],
    needle_value: Dword::new(0x0),
    needle_0: [
        Float::from_bits(0x42c00000),
        Float::from_bits(0x43fb8000),
        Float::from_bits(0x3f800000),
    ],
    needle_2a: [
        Float::from_bits(0x42900000),
        Float::from_bits(0x43520000),
        Float::from_bits(0x43620000),
        Float::from_bits(0x3f400000),
    ],
    needle_2b: [
        Float::from_bits(0x42900000),
        Float::from_bits(0x44034000),
        Float::from_bits(0x3f400000),
    ],
    needle_1a: [
        Float::from_bits(0x42900000),
        Float::from_bits(0x44034000),
        Float::from_bits(0x3f400000),
    ],
    needle_1b: [
        Float::from_bits(0x43ec8000),
        Float::from_bits(0x44034000),
        Float::from_bits(0x3f400000),
    ],
    vertices: [const { Vertex::zero() }; 0x400],
    indices: [const { Word::new(0) }; 0x600],
    vertex_count: Dword::new(0x0),
    index_count: Dword::new(0x0),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x75B940).
pub fn hud_addr() -> u32 {
    crate::pointers::game_address(&raw const HUD)
}

/// The rectangle `n` of the table as an address (`add_sprite` is handed rectangles by
/// address: the record's numbers are not checked, as in the original).
pub fn rect_addr(n: u32) -> u32 {
    (crate::pointers::game_address(HUD.rects.as_ptr())).wrapping_add(n.wrapping_mul(0x10))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        let _g = crate::store::test_lock();
        assert_eq!(size_of::<Hud>(), LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(crate::pointers::host(hud_addr()).cast_const(), LEN)
        };
        let image = crate::moved::image_bytes(START, LEN);
        assert!(
            mine == &image[..],
            "the block's bytes differ from the image's"
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(0x75C060), HUD.scratch.addr() as usize);
        assert_eq!(crate::moved::place(START), hud_addr() as usize);
        assert_eq!(core::mem::offset_of!(Hud, vertices), 0x75C324 - START);
        assert_eq!(core::mem::offset_of!(Hud, indices), 0x764324 - START);
        assert_eq!(
            HUD.vertex_count.addr(),
            hud_addr() + 0x764F24 - START as u32
        );
    }
}
