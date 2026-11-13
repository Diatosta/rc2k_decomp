//! The HUD's variables, out of RAL.EXE's image (phase 5; typed in phase 7,
//! `notes/conversion.md`): IDA 0x75B940 .. 0x764F2C, the whole of it the HUD's: the
//! rectangle table (`g_hudRects`), the quad record, the layouts' set-up values and floats,
//! the dials' and needles' work variables, the batch's vertices (`HudBlock.vertices[0].x`, 0x400 of
//! them), indices (`HudBlock.indices[0]`, 0x600) and the two counts (`g_hudVertexCount`,
//! `g_hudIndexCount`). Only `hud` and `d3d::draw_hud_batch` (which reads `vertices` and
//! `indices`) read or write it. The image keeps the two pointers (0x764F2C, 0x764F30) the
//! original's batch drawer followed; nothing reads them now.
//!
//! One struct (`HudBlock`, the static `HUD`). Its records keep the game's layout (`repr(C)`, `ByteView`):
//! the recorded-assembly tests of the disc and quad code (`disc_vertex`) read it by byte offset.

use crate::session::{PlainDword, PlainFloat, PlainWord};

/// The HUD's quad record (`g_hudQuad`, 14 dwords): what `add_quad` makes four vertices of.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct HudQuad {
    /// The alpha (the colour's top byte under it, `add_quad`).
    pub alpha: PlainDword,
    pub left: PlainDword,
    pub top: PlainDword,
    pub right: PlainDword,
    pub bottom: PlainDword,
    /// The colour (0xFFFF_FFFF white, 0 black; its low 24 bits are used under the alpha).
    pub colour: PlainDword,
    /// The four corners' texture places as whole numbers (u, v pairs in the order
    /// `rc2k_core::hud::quads::sprite_places` numbers them 6 .. 13).
    pub places: [PlainDword; 8],
}

impl HudQuad {
    const fn new() -> Self {
        Self {
            alpha: PlainDword::new(0xc0),
            left: PlainDword::new(0),
            top: PlainDword::new(0),
            right: PlainDword::new(0),
            bottom: PlainDword::new(0),
            colour: PlainDword::new(0),
            places: [const { PlainDword::new(0) }; 8],
        }
    }

    /// The record's dword `n` (0 .. 14, the order of the fields).
    pub fn word(&self, n: usize) -> &PlainDword {
        match n {
            0 => &self.alpha,
            1 => &self.left,
            2 => &self.top,
            3 => &self.right,
            4 => &self.bottom,
            5 => &self.colour,
            _ => &self.places[n - 6],
        }
    }

    /// The corners: left, top, right, bottom.
    pub fn set_rect(&self, [l, t, r, b]: [u32; 4]) {
        self.left.set(l);
        self.top.set(t);
        self.right.set(r);
        self.bottom.set(b);
    }

    /// The whole record in order.
    pub fn words(&self) -> [u32; 14] {
        std::array::from_fn(|n| self.word(n).get())
    }
}

/// The HUD's texture numbers (`HudBlock::texture`, the batch textures 0xC0 .. 0xDC).
pub mod texture {
    /// The sprite sheet of the markers, signs, digits and the stage bar's panels.
    pub const SPRITES: u32 = 0xC0;
    /// The sheet of the speed and gear digits, the needle and the time panels.
    pub const NEEDLES_AND_DIGITS: u32 = 0xC1;
    /// The first player's dial disc.
    pub const DIAL_FIRST: u32 = 0xC6;
    /// The second player's dial disc.
    pub const DIAL_SECOND: u32 = 0xC7;
    /// The icons.
    pub const ICONS: u32 = 0xDC;
}

/// One of the texture's rectangles (`g_hudRects`).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct Rect {
    pub left: PlainDword,
    pub top: PlainDword,
    pub width: PlainDword,
    pub height: PlainDword,
}

impl Rect {
    const fn new(left: u32, top: u32, width: u32, height: u32) -> Self {
        Self {
            left: PlainDword::new(left),
            top: PlainDword::new(top),
            width: PlainDword::new(width),
            height: PlainDword::new(height),
        }
    }
}

/// A batch vertex (Direct3D's transformed and lit vertex, 0x20 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct Vertex {
    pub x: PlainFloat,
    pub y: PlainFloat,
    pub z: PlainFloat,
    pub rhw: PlainFloat,
    pub colour: PlainDword,
    pub specular: PlainDword,
    pub u: PlainFloat,
    pub v: PlainFloat,
}

impl Vertex {
    const fn zero() -> Self {
        Self {
            x: PlainFloat::new(0.0),
            y: PlainFloat::new(0.0),
            z: PlainFloat::new(0.0),
            rhw: PlainFloat::new(0.0),
            colour: PlainDword::new(0),
            specular: PlainDword::new(0),
            u: PlainFloat::new(0.0),
            v: PlainFloat::new(0.0),
        }
    }
}

/// The HUD's variables (IDA 0x75B940 .. 0x764F2C).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct HudBlock {
    /// `g_hudRects` (92 rectangles) and `HudBlock.rects[92].left` (the last five): the texture's rectangles, 16 bytes each (left, top, width, height), picked by number (`hud::layout_*` sprites, markers, signs). Sign `v` is rectangle 0x4C + (v - 1) & 0xF.
    pub rects: [Rect; 0x61],
    /// `dword_75BF50`, `dword_75BF54` (the stage marker's two colours, `draw_markers`) and the six dwords after them (probably colours too; nothing ported reads them).
    pub marker_colours: [PlainDword; 8],
    /// `dword_75BF70`: the stage bar's four part colours, read at the byte offset the kind table gives (`draw_stage_bar`).
    pub stage_colours: [PlainDword; 4],
    /// `dword_75BF80`: the widths of the eight signs (`draw_signs`: centres a sign in its place).
    pub sign_widths: [PlainDword; 8],
    /// `dword_75BFA0`: the part colour of the one-player stage bar.
    pub bar_colour: PlainDword,
    /// `dword_75BFA4`, `dword_75BFA8`: the part colours of the two-player stage bars.
    pub bar_colours: [PlainDword; 2],
    /// `dword_75BFAC`: the needle's way (the record's +0xCC / +0xD0; 0 or not, `draw_needle`).
    pub needle_kind: PlainDword,
    /// `g_hudQuad` (`HudBlock.quad.alpha` .. `HudBlock.quad.places[7]`): the quad record `add_quad` makes four vertices of: alpha, left, top, right, bottom, colour, then the four corners' texture places (whole numbers, `rc2k_core::hud::quads::RECORD` dwords).
    pub quad: HudQuad,
    /// `flt_75BFE8`: the batch's depth (z of every vertex, `set_batch_depth`).
    pub depth: PlainFloat,
    /// `flt_75BFEC`: the batch's rhw (`set_batch_depth`).
    pub rhw: PlainFloat,
    /// `dword_75BFF0`, `dword_75BFF4`: the stage bar's part places (whole numbers) and a turned shape's point (floats' bits, `turned_vertex`).
    pub point: [PlainDword; 2],
    /// `dword_75BFF8`, `dword_75BFFC`: the stage bar's part start and end (miles, as floats).
    pub ends: [PlainFloat; 2],
    /// `dword_75C000`: the stage bar's width in places.
    pub bar_width: PlainDword,
    /// `0x75C004`, `0x75C008`: nothing reads or writes them (a gap).
    pub unused_c004: [PlainDword; 2],
    /// `flt_75C00C`: the texture places over it make the u (`add_quad`, `turned_vertex`, markers).
    pub uv_scale_x: PlainFloat,
    /// `flt_75C010`: the texture places over it make the v; also the needle's first turn factor (`draw_needle`).
    pub uv_scale_y: PlainFloat,
    /// `flt_75C014`: the needle's second turn factor.
    pub turn_k: PlainFloat,
    /// `flt_75C018`: the needle's least value.
    pub least: PlainFloat,
    /// `flt_75C01C`: added to a disc entry's texture place (`disc_vertex`).
    pub disc_place_offset: PlainFloat,
    /// `flt_75C020`: a disc entry's texture place is divided by it (`disc_vertex`).
    pub disc_place_divisor: PlainFloat,
    /// `flt_75C024`: the second factor of the disc texture's turn (`draw_disc`).
    pub disc_turn_b: PlainFloat,
    /// `flt_75C028`: the first factor of the disc texture's turn (`draw_disc`).
    pub disc_turn_a: PlainFloat,
    /// `flt_75C02C`, `flt_75C030`: the virtual screen's width and height the HUD is drawn in (`add_polygon` scales to the real one).
    pub virtual_screen: [PlainFloat; 2],
    /// `dword_75C034`: the texture asked for.
    pub texture: PlainDword,
    /// `dword_75C038`: the batch's texture (-1 none).
    pub batch_texture: PlainDword,
    /// `dword_75C03C`: 1 while the leading digits of a time are 0 (`layout_0::leading_time`).
    pub leading: PlainDword,
    /// `dword_75C040`: the needle sign's rectangle.
    pub pick: PlainDword,
    /// `dword_75C044`, `dword_75C048`: the disc's / needle's centre (x, y), copied from a set-up.
    pub disc_centre: [PlainFloat; 2],
    /// `dword_75C04C`: the disc's texture scale: the ring's (sine x size) is multiplied by it to get the texture offsets (`hud::disc_vertex`); set from the layout's disc set-up (`dials::set_up_disc`).
    pub disc_texture_scale: PlainFloat,
    /// `dword_75C050`: the disc's size (the ring entries are times it).
    pub disc_size: PlainFloat,
    /// `dword_75C054`, `dword_75C058`: the disc texture's shift (u, v), from the record.
    pub disc_texture_shift: [PlainFloat; 2],
    /// `dword_75C05C`: the disc texture's turn, from the record (+0x3C / +0x64).
    pub disc_turn: PlainFloat,
    /// `dword_75C060`: the original's `fistp` scratch (a sprite's height, the needle's turn, the marker's x).
    pub fistp_scratch: PlainDword,
    /// `dword_75C064`, `dword_75C068`: the icon's width and drop, scaled.
    pub icon_size: [PlainDword; 2],
    /// `dword_75C06C`: the icon scale (from `icon_scales`).
    pub icon_scale: PlainFloat,
    /// `dword_75C070`: the icon's shift.
    pub icon_shift: PlainDword,
    /// `dword_75C074` .. `dword_75C088`: six record times in tenths (`tenths`): the two clocks, the lap time, the two layout-1 clocks (`dword_75C080`, `dword_75C084`) and the second lap time (`dword_75C088`).
    pub times: [PlainDword; 6],
    /// `dword_75C08C` .. `dword_75C0A0`: a time made digits (`layout_0::time_digits`).
    pub digits: [PlainDword; 6],
    /// `dword_75C0A4`: a time's sign rectangle.
    pub sign: PlainDword,
    /// `flt_75C0A8`, `flt_75C0AC`: the stage marker's position factors (`draw_marker`).
    pub marker_k: [PlainFloat; 2],
    /// `0x75C0B0`, `0x75C0B4`: nothing reads or writes them (a gap; 0.96875 and 0.03125).
    pub unused_c0b0: [PlainFloat; 2],
    /// `dword_75C0B8`, `dword_75C0BC`, `dword_75C0C0`: the icon scale of layouts 0, 2 and 1 (`icons_*`).
    pub icon_scales: [PlainFloat; 3],
    /// `dword_75C0C4` .. `0x75C120`: whole numbers in fours, probably six screen rectangles (x, y, width, height; e.g. 0x25d, 0x30, 0xd6, 0x24); nothing ported reads them; in IDA only the first two are touched, written by code outside any function (0x4B6968).
    pub unused_c0c4: [PlainDword; 0x18],
    /// `dword_75C124` .. `dword_75C130`: nothing reads or writes them.
    pub unused_c124: [PlainDword; 4],
    /// `dword_75C134` .. `dword_75C144`: the set-up of layout 2's first disc (copied by `dials::set_up`; the fifth only with `` 0).
    pub disc_setup_layout2_first: [PlainFloat; 5],
    /// `dword_75C148` .. `dword_75C154`: the set-up of layout 2's second disc.
    pub disc_setup_layout2_second: [PlainFloat; 4],
    /// `dword_75C158` .. `dword_75C164`: the set-up of layout 1's first disc.
    pub disc_setup_layout1_first: [PlainFloat; 4],
    /// `dword_75C168` .. `dword_75C174`: the set-up of layout 1's second disc.
    pub disc_setup_layout1_second: [PlainFloat; 4],
    /// `dword_75C178` .. `dword_75C184`: the set-up of layout 0's disc.
    pub disc_setup_layout0: [PlainFloat; 4],
    /// `flt_75C188`: the disc's ring, sines of the entries (1 .. 16 written; entry 0 stays 0).
    pub ring_sines: [PlainFloat; 17],
    /// `flt_75C1CC`: the ring's cosines.
    pub ring_cosines: [PlainFloat; 17],
    /// `flt_75C210`: the texture ring's sines.
    pub texture_ring_sines: [PlainFloat; 17],
    /// `flt_75C254`: the texture ring's cosines.
    pub texture_ring_cosines: [PlainFloat; 17],
    /// `flt_75C298`, `flt_75C29C`: added to a turned shape's point to make its texture place (`turned_vertex`).
    pub uv_shift: [PlainFloat; 2],
    /// `dword_75C2A0`, `dword_75C2A4`: the needle's centre (x, y), copied from a set-up.
    pub needle_centre: [PlainFloat; 2],
    /// `flt_75C2A8` .. `flt_75C2B4`: the needle's corner offsets (`needle_corners`).
    pub needle_corners: [PlainFloat; 4],
    /// `dword_75C2B8`: the needle's size, copied from a set-up.
    pub needle_size: PlainFloat,
    /// `flt_75C2BC` .. `flt_75C2D4`: the needle's base, base 2, split, scale, scale 2, most, most 2.
    pub needle_k: [PlainFloat; 7],
    /// `0x75C2D8`, `0x75C2DC`: nothing reads or writes them (a gap).
    pub unused_c2d8: [PlainFloat; 2],
    /// `dword_75C2E0`: the needle's value (a whole number, from the record).
    pub needle_value: PlainDword,
    /// `dword_75C2E4` .. `dword_75C2EC`: the set-up of layout 0's needle (centre x, y, size).
    pub needle_setup_layout0: [PlainFloat; 3],
    /// `dword_75C2F0` .. `dword_75C2FC`: the set-up of layout 2's first needle (the third only with `` 0).
    pub needle_setup_layout2_first: [PlainFloat; 4],
    /// `dword_75C300` .. `dword_75C308`: the set-up of layout 2's second needle.
    pub needle_setup_layout2_second: [PlainFloat; 3],
    /// `dword_75C30C` .. `dword_75C314`: the set-up of layout 1's first needle.
    pub needle_setup_layout1_first: [PlainFloat; 3],
    /// `dword_75C318` .. `dword_75C320`: the set-up of layout 1's second needle.
    pub needle_setup_layout1_second: [PlainFloat; 3],
    /// `HudBlock.vertices[0].x`: the batch's vertices (transformed and lit, 0x20 bytes each).
    pub vertices: [Vertex; 0x400],
    /// `unk_764324`: the batch's indices (16 bits, a triangle list).
    pub indices: [PlainWord; 0x600],
    /// `g_hudVertexCount`: the vertices in the batch.
    pub vertex_count: PlainDword,
    /// `g_hudIndexCount`: the indices in the batch.
    pub index_count: PlainDword,
}

impl HudBlock {
    /// The HUD's variables; first values the image's.
    pub const fn new() -> Self {
        HudBlock {
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
                PlainDword::new(0xffffff),
                PlainDword::new(0xff0300),
                PlainDword::new(0xff8d00),
                PlainDword::new(0xf0ff00),
                PlainDword::new(0xca1b),
                PlainDword::new(0x2700ff),
                PlainDword::new(0x21dcde),
                PlainDword::new(0x8b00ff),
            ],
            stage_colours: [
                PlainDword::new(0x212463),
                PlainDword::new(0xe12626),
                PlainDword::new(0x32a126),
                PlainDword::new(0x227edc),
            ],
            sign_widths: [
                PlainDword::new(0x20),
                PlainDword::new(0x22),
                PlainDword::new(0x2d),
                PlainDword::new(0x36),
                PlainDword::new(0x38),
                PlainDword::new(0x33),
                PlainDword::new(0x32),
                PlainDword::new(0x35),
            ],
            bar_colour: PlainDword::new(0x0),
            bar_colours: [const { PlainDword::new(0x0) }; 2],
            needle_kind: PlainDword::new(0x0),
            quad: HudQuad::new(),
            depth: PlainFloat::new(0.01),
            rhw: PlainFloat::new(1.0),
            point: [const { PlainDword::new(0x0) }; 2],
            ends: [const { PlainFloat::new(0.0) }; 2],
            bar_width: PlainDword::new(0x0),
            unused_c004: [const { PlainDword::new(0x3e000000) }; 2],
            uv_scale_x: PlainFloat::new(256.0),
            uv_scale_y: PlainFloat::new(65536.0),
            turn_k: PlainFloat::new(4096.0),
            least: PlainFloat::new(0.0),
            disc_place_offset: PlainFloat::new(48.0),
            disc_place_divisor: PlainFloat::new(128.0),
            disc_turn_b: PlainFloat::new(1024.0),
            disc_turn_a: PlainFloat::new(0.63661975),
            virtual_screen: [PlainFloat::new(800.0), PlainFloat::new(600.0)],
            texture: PlainDword::new(0x0),
            batch_texture: PlainDword::new(0x0),
            leading: PlainDword::new(0x0),
            pick: PlainDword::new(0x0),
            disc_centre: [PlainFloat::new(706.0), PlainFloat::new(504.0)],
            disc_texture_scale: PlainFloat::new(0.75),
            disc_size: PlainFloat::new(62.0),
            disc_texture_shift: [const { PlainFloat::new(0.0) }; 2],
            disc_turn: PlainFloat::new(0.0),
            fistp_scratch: PlainDword::new(0x0),
            icon_size: [const { PlainDword::new(0x0) }; 2],
            icon_scale: PlainFloat::new(0.0),
            icon_shift: PlainDword::new(0x0),
            times: [const { PlainDword::new(0x0) }; 6],
            digits: [const { PlainDword::new(0x0) }; 6],
            sign: PlainDword::new(0x0),
            marker_k: [PlainFloat::new(550.0), PlainFloat::new(119.0)],
            unused_c0b0: [PlainFloat::new(0.96875), PlainFloat::new(0.03125)],
            icon_scales: [
                PlainFloat::new(1.0),
                PlainFloat::new(0.75),
                PlainFloat::new(1.0),
            ],
            unused_c0c4: [
                PlainDword::new(0x25d),
                PlainDword::new(0x30),
                PlainDword::new(0xd6),
                PlainDword::new(0x24),
                PlainDword::new(0x25d),
                PlainDword::new(0x4),
                PlainDword::new(0x25d),
                PlainDword::new(0x30),
                PlainDword::new(0x268),
                PlainDword::new(0x24),
                PlainDword::new(0x25d),
                PlainDword::new(0x141),
                PlainDword::new(0x25d),
                PlainDword::new(0x10),
                PlainDword::new(0xd6),
                PlainDword::new(0x4),
                PlainDword::new(0x25d),
                PlainDword::new(0x4),
                PlainDword::new(0x25d),
                PlainDword::new(0x10),
                PlainDword::new(0x268),
                PlainDword::new(0x4),
                PlainDword::new(0x25d),
                PlainDword::new(0x131),
            ],
            unused_c124: [const { PlainDword::new(0x0) }; 4],
            disc_setup_layout2_first: [
                PlainFloat::new(46.0),
                PlainFloat::new(0.75),
                PlainFloat::new(727.0),
                PlainFloat::new(211.0),
                PlainFloat::new(227.0),
            ],
            disc_setup_layout2_second: [
                PlainFloat::new(46.0),
                PlainFloat::new(0.75),
                PlainFloat::new(727.0),
                PlainFloat::new(526.0),
            ],
            disc_setup_layout1_first: [
                PlainFloat::new(46.0),
                PlainFloat::new(0.75),
                PlainFloat::new(326.0),
                PlainFloat::new(526.0),
            ],
            disc_setup_layout1_second: [
                PlainFloat::new(46.0),
                PlainFloat::new(0.75),
                PlainFloat::new(727.0),
                PlainFloat::new(526.0),
            ],
            disc_setup_layout0: [
                PlainFloat::new(62.0),
                PlainFloat::new(0.75),
                PlainFloat::new(706.0),
                PlainFloat::new(504.0),
            ],
            ring_sines: [const { PlainFloat::new(0.0) }; 17],
            ring_cosines: [const { PlainFloat::new(0.0) }; 17],
            texture_ring_sines: [const { PlainFloat::new(0.0) }; 17],
            texture_ring_cosines: [const { PlainFloat::new(0.0) }; 17],
            uv_shift: [PlainFloat::new(253.5), PlainFloat::new(242.5)],
            needle_centre: [PlainFloat::new(96.0), PlainFloat::new(503.0)],
            needle_corners: [
                PlainFloat::new(2.5),
                PlainFloat::new(13.5),
                PlainFloat::new(-2.5),
                PlainFloat::new(-41.5),
            ],
            needle_size: PlainFloat::new(0.0),
            needle_k: [
                PlainFloat::new(0.375),
                PlainFloat::new(0.4375),
                PlainFloat::new(4000.0),
                PlainFloat::new(7000.0),
                PlainFloat::new(8000.0),
                PlainFloat::new(64000.0),
                PlainFloat::new(9500.0),
            ],
            unused_c2d8: [PlainFloat::new(3000.0), PlainFloat::new(48000.0)],
            needle_value: PlainDword::new(0x0),
            needle_setup_layout0: [
                PlainFloat::new(96.0),
                PlainFloat::new(503.0),
                PlainFloat::new(1.0),
            ],
            needle_setup_layout2_first: [
                PlainFloat::new(72.0),
                PlainFloat::new(210.0),
                PlainFloat::new(226.0),
                PlainFloat::new(0.75),
            ],
            needle_setup_layout2_second: [
                PlainFloat::new(72.0),
                PlainFloat::new(525.0),
                PlainFloat::new(0.75),
            ],
            needle_setup_layout1_first: [
                PlainFloat::new(72.0),
                PlainFloat::new(525.0),
                PlainFloat::new(0.75),
            ],
            needle_setup_layout1_second: [
                PlainFloat::new(473.0),
                PlainFloat::new(525.0),
                PlainFloat::new(0.75),
            ],
            vertices: [const { Vertex::zero() }; 0x400],
            indices: [const { PlainWord::new(0) }; 0x600],
            vertex_count: PlainDword::new(0x0),
            index_count: PlainDword::new(0x0),
        }
    }
}

/// A rectangle of the table, by its number. Every number the drawers pass lies in the table:
/// the layouts' fixed ones (at most 0x5F), the needle's sign (0x5E or 0x60), the stage panel's
/// (0x29 .. 0x3F), the time digits (a digit at most 9 over the panel's base, 0xC, 0x18 or 2),
/// the signs (0x4C .. 0x5B: the value is masked) and the gear (0x41 over the gear, which
/// physics holds to -1 ..= 7: every gear change refuses one outside, and the table takes any
/// from -0x41 to 0x1F). The original does not check, and reads on into the fields after the
/// table for a number past it; that is refused.
#[derive(Clone, Copy)]
pub struct RectAt(u32);

impl RectAt {
    /// Rectangle `n` of the table.
    pub fn number(n: u32) -> RectAt {
        RectAt(n)
    }

    /// Its left, top, width and height.
    pub fn read(self, hud: &crate::hud::owner::Hud) -> Result<[u32; 4], crate::errors::Refused> {
        let rect = hud
            .block
            .rects
            .get(self.0 as usize)
            .ok_or(crate::errors::BadInput(
                "a HUD sprite's rectangle past the rectangle table",
            ))?;
        Ok([
            rect.left.get(),
            rect.top.get(),
            rect.width.get(),
            rect.height.get(),
        ])
    }
}
