//! The HUD's variables, the whole of them the HUD's: the rectangle table, the quad record, the
//! layouts' set-up values and floats, the dials' and needles' work variables, the batch's
//! vertices (`HudBlock.vertices`, 0x400 of them), indices (`HudBlock.indices`, 0x600) and the
//! two counts. Only `hud` and `d3d::draw_hud_batch` (which reads `vertices` and `indices`) read
//! or write it.
//!
//! One struct (`HudBlock`), plain fields, held by `owner::Hud` and lent `&mut` down the HUD's
//! drawing (the race keeps the owner behind one `RefCell`).

/// The HUD's quad record (14 dwords): what `add_quad` makes four vertices of.
pub struct HudQuad {
    /// The alpha (the colour's top byte under it, `add_quad`).
    pub alpha: u32,
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    /// The colour (0xFFFF_FFFF white, 0 black; its low 24 bits are used under the alpha).
    pub colour: u32,
    /// The four corners' texture places as whole numbers (u, v pairs in the order
    /// `rc2k_core::hud::quads::sprite_places` numbers them 6 .. 13).
    pub places: [u32; 8],
}

impl HudQuad {
    const fn new() -> Self {
        Self {
            alpha: 0xc0,
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
            colour: 0,
            places: [0; 8],
        }
    }

    /// The corners: left, top, right, bottom.
    pub(super) fn set_rect(&mut self, [l, t, r, b]: [u32; 4]) {
        self.left = l;
        self.top = t;
        self.right = r;
        self.bottom = b;
    }

    /// The four corners' texture places, each (u, v).
    pub(super) fn set_places(&mut self, corners: [[u32; 2]; 4]) {
        for (k, [u, v]) in corners.into_iter().enumerate() {
            self.places[2 * k] = u;
            self.places[2 * k + 1] = v;
        }
    }

    /// The record's dwords, in the order of the fields.
    pub fn words(&self) -> [u32; 14] {
        let [p0, p1, p2, p3, p4, p5, p6, p7] = self.places;
        [
            self.alpha,
            self.left,
            self.top,
            self.right,
            self.bottom,
            self.colour,
            p0,
            p1,
            p2,
            p3,
            p4,
            p5,
            p6,
            p7,
        ]
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
    /// The third player's dial disc (a split screen of three or four).
    pub const DIAL_THIRD: u32 = rc2k_core::scene::texture_levels::FURTHER_MAP_TEXTURES[0];
    /// The fourth player's dial disc.
    pub const DIAL_FOURTH: u32 = rc2k_core::scene::texture_levels::FURTHER_MAP_TEXTURES[1];
    /// The icons.
    pub const ICONS: u32 = 0xDC;
}

pub struct Rect {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    const fn new(left: u32, top: u32, width: u32, height: u32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }
}

/// A batch vertex (Direct3D's transformed and lit vertex, 0x20 bytes).
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rhw: f32,
    pub colour: u32,
    pub specular: u32,
    pub u: f32,
    pub v: f32,
}

impl Vertex {
    const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            rhw: 0.0,
            colour: 0,
            specular: 0,
            u: 0.0,
            v: 0.0,
        }
    }
}

pub struct HudBlock {
    /// The texture's rectangles, 16 bytes each (left, top, width, height), picked by number
    /// (sprites, markers, signs). Sign `v` is rectangle 0x4C + (v - 1) & 0xF.
    pub rects: [Rect; 0x61],
    /// The stage marker's two colours (`draw_markers`) and the six dwords after them (probably
    /// colours too; nothing reads them).
    pub marker_colours: [u32; 8],
    /// The stage bar's four part colours, read at the byte position the kind table gives
    /// (`draw_stage_bar`).
    pub stage_colours: [u32; 4],
    /// The widths of the eight signs (`draw_signs`: centres a sign in its place).
    pub sign_widths: [u32; 8],
    /// The needle's way (taken from the dial's set-up record; 0 or not, `draw_needle`).
    pub needle_kind: u32,
    /// The quad record `add_quad` makes four vertices of.
    pub quad: HudQuad,
    /// The batch's depth (z of every vertex, `set_batch_depth`).
    pub depth: f32,
    /// The batch's rhw (`set_batch_depth`).
    pub rhw: f32,
    /// The stage bar's part places (whole numbers) and a turned shape's point (floats' bits,
    /// `turned_vertex`).
    pub point: [u32; 2],
    /// The stage bar's part start and end (miles, as floats).
    pub ends: [f32; 2],
    /// The stage bar's width in places.
    pub bar_width: u32,
    /// The texture places over it make the u (`add_quad`, `turned_vertex`, markers).
    pub uv_scale_x: f32,
    /// The texture places over it make the v; also the needle's first turn factor (`draw_needle`).
    pub uv_scale_y: f32,
    /// The needle's second turn factor.
    pub turn_k: f32,
    /// The needle's least value.
    pub least: f32,
    /// Added to a disc entry's texture place (`disc_vertex`).
    pub disc_place_offset: f32,
    /// A disc entry's texture place is divided by it (`disc_vertex`).
    pub disc_place_divisor: f32,
    /// The second factor of the disc texture's turn (`draw_disc`).
    pub disc_turn_b: f32,
    /// The first factor of the disc texture's turn (`draw_disc`).
    pub disc_turn_a: f32,
    /// The virtual screen's width and height the HUD is drawn in (`add_polygon` scales to the
    /// real one).
    pub virtual_screen: [f32; 2],
    /// The texture asked for.
    pub texture: u32,
    /// The batch's texture (-1 none).
    pub batch_texture: u32,
    /// 1 while the leading digits of a time are 0 (`layout_0::leading_time`).
    pub leading: u32,
    /// The needle sign's rectangle.
    pub pick: u32,
    /// The disc's / needle's centre (x, y), copied from a set-up.
    pub disc_centre: [f32; 2],
    /// The disc's texture scale: the ring's (sine x size) is multiplied by it to get the texture
    /// offsets (`hud::disc_vertex`).
    pub disc_texture_scale: f32,
    /// The disc's size (the ring entries are times it).
    pub disc_size: f32,
    /// The disc texture's shift (u, v), from the record.
    pub disc_texture_shift: [f32; 2],
    /// The disc texture's turn, from the dial's set-up record.
    pub disc_turn: f32,
    /// A scratch for the x87's integer stores (a sprite's height, the needle's turn, the
    /// marker's x).
    pub fistp_scratch: u32,
    /// The icon's width and drop, scaled.
    pub icon_size: [u32; 2],
    /// The icon scale (from `icon_scales`).
    pub icon_scale: f32,
    /// The icon's shift.
    pub icon_shift: u32,
    /// Six record times in tenths (`tenths`): the two clocks, the lap time, the two layout-1
    /// clocks and the second lap time.
    pub times: [u32; 6],
    /// A time made digits (`layout_0::time_digits`).
    pub digits: [u32; 6],
    /// A time's sign rectangle.
    pub sign: u32,
    /// The stage marker's position factors (`draw_marker`).
    pub marker_k: [f32; 2],
    /// The icon scale of layouts 0, 2 and 1 (`icons_*`).
    pub icon_scales: [f32; 3],
    /// The set-up of layout 2's first disc (copied by `dials::set_up`; the fifth only with
    /// `Panel::flat` 0).
    pub disc_setup_layout2_first: [f32; 5],
    pub disc_setup_layout2_second: [f32; 4],
    pub disc_setup_layout1_first: [f32; 4],
    pub disc_setup_layout1_second: [f32; 4],
    pub disc_setup_layout0: [f32; 4],
    /// The disc's ring, sines of the entries (1 .. 16 written; entry 0 stays 0).
    pub ring_sines: [f32; 17],
    pub ring_cosines: [f32; 17],
    pub texture_ring_sines: [f32; 17],
    pub texture_ring_cosines: [f32; 17],
    /// Added to a turned shape's point to make its texture place (`turned_vertex`).
    pub uv_shift: [f32; 2],
    /// The needle's centre (x, y), copied from a set-up.
    pub needle_centre: [f32; 2],
    /// The needle's corner offsets (`needle_corners`).
    pub needle_corners: [f32; 4],
    /// The needle's size, copied from a set-up.
    pub needle_size: f32,
    /// The needle's base, base 2, split, scale, scale 2, most, most 2.
    pub needle_k: [f32; 7],
    /// The needle's value (a whole number, from the record).
    pub needle_value: u32,
    /// The set-up of layout 0's needle (centre x, y, size).
    pub needle_setup_layout0: [f32; 3],
    /// The set-up of layout 2's first needle (the third only with `Panel::flat` 0).
    pub needle_setup_layout2_first: [f32; 4],
    /// The set-up of layout 2's second needle.
    pub needle_setup_layout2_second: [f32; 3],
    /// The set-up of layout 1's first needle.
    pub needle_setup_layout1_first: [f32; 3],
    /// The set-up of layout 1's second needle.
    pub needle_setup_layout1_second: [f32; 3],
    /// `HudBlock.vertices[0].x`: the batch's vertices (transformed and lit, 0x20 bytes each).
    pub vertices: [Vertex; 0x400],
    /// The batch's indices (16 bits, a triangle list).
    pub indices: [u16; 0x600],
    /// The vertices in the batch.
    pub vertex_count: u32,
    /// The indices in the batch.
    pub index_count: u32,
    // Our own, after the fields above (the kept assembly reaches those by byte position).
    /// How the pieces drawn now keep to the screen on a wide one (`layout_common::Keep`).
    pub anchor_rule: u32,
    /// The rule the layout starts with and goes back to.
    pub base_rule: u32,
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
                0xffffff, 0xff0300, 0xff8d00, 0xf0ff00, 0xca1b, 0x2700ff, 0x21dcde, 0x8b00ff,
            ],
            stage_colours: [0x212463, 0xe12626, 0x32a126, 0x227edc],
            sign_widths: [0x20, 0x22, 0x2d, 0x36, 0x38, 0x33, 0x32, 0x35],
            needle_kind: 0x0,
            quad: HudQuad::new(),
            depth: 0.01,
            rhw: 1.0,
            point: [0x0; 2],
            ends: [0.0; 2],
            bar_width: 0x0,
            uv_scale_x: 256.0,
            uv_scale_y: 65536.0,
            turn_k: 4096.0,
            least: 0.0,
            disc_place_offset: 48.0,
            disc_place_divisor: 128.0,
            disc_turn_b: 1024.0,
            disc_turn_a: 0.63661975,
            virtual_screen: [800.0, 600.0],
            texture: 0x0,
            batch_texture: 0x0,
            leading: 0x0,
            pick: 0x0,
            disc_centre: [706.0, 504.0],
            disc_texture_scale: 0.75,
            disc_size: 62.0,
            disc_texture_shift: [0.0; 2],
            disc_turn: 0.0,
            fistp_scratch: 0x0,
            icon_size: [0x0; 2],
            icon_scale: 0.0,
            icon_shift: 0x0,
            times: [0x0; 6],
            digits: [0x0; 6],
            sign: 0x0,
            marker_k: [550.0, 119.0],
            icon_scales: [1.0, 0.75, 1.0],
            disc_setup_layout2_first: [46.0, 0.75, 727.0, 211.0, 227.0],
            disc_setup_layout2_second: [46.0, 0.75, 727.0, 526.0],
            disc_setup_layout1_first: [46.0, 0.75, 326.0, 526.0],
            disc_setup_layout1_second: [46.0, 0.75, 727.0, 526.0],
            disc_setup_layout0: [62.0, 0.75, 706.0, 504.0],
            ring_sines: [0.0; 17],
            ring_cosines: [0.0; 17],
            texture_ring_sines: [0.0; 17],
            texture_ring_cosines: [0.0; 17],
            uv_shift: [253.5, 242.5],
            needle_centre: [96.0, 503.0],
            needle_corners: [2.5, 13.5, -2.5, -41.5],
            needle_size: 0.0,
            needle_k: [0.375, 0.4375, 4000.0, 7000.0, 8000.0, 64000.0, 9500.0],
            needle_value: 0x0,
            needle_setup_layout0: [96.0, 503.0, 1.0],
            needle_setup_layout2_first: [72.0, 210.0, 226.0, 0.75],
            needle_setup_layout2_second: [72.0, 525.0, 0.75],
            needle_setup_layout1_first: [72.0, 525.0, 0.75],
            needle_setup_layout1_second: [473.0, 525.0, 0.75],
            vertices: [const { Vertex::zero() }; 0x400],
            indices: [0; 0x600],
            vertex_count: 0x0,
            index_count: 0x0,
            anchor_rule: 0x0,
            base_rule: 0x0,
        }
    }
}

/// A rectangle of the table, by its number. Every number the drawers pass lies in the table:
/// the layouts' fixed ones (at most 0x5F), the needle's sign (0x5E or 0x60), the stage panel's
/// (0x29 .. 0x3F), the time digits (a digit at most 9 over the panel's base, 0xC, 0x18 or 2),
/// the signs (0x4C .. 0x5B: the value is masked) and the gear (0x41 over the gear, which
/// physics holds to -1 ..= 7; only a damaged replay file brings one outside, and the gear's
/// drawer, `layout_common::gear_rect`, clamps it and logs it). The game does not check and reads on into the
/// fields after the table for a number past it; that is refused.
#[derive(Clone, Copy)]
pub(super) struct RectAt(u32);

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
        Ok([rect.left, rect.top, rect.width, rect.height])
    }
}
