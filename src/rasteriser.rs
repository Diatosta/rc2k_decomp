//! A software rasteriser for the standalone program: what Direct3D 5 did with the
//! game's polygons. The game hands over triangle fans of `D3DTLVERTEX` (x, y, z, 1/w
//! as floats, the colour and the specular as ARGB, u, v), already projected, lit and
//! clipped (`d3d::draw_fan_unclipped`); this draws them into the screen's memory with
//! the render states the game set, and a depth buffer of floats.
//!
//! What it does, by the states (DirectX 5's numbers; those the game sets are listed
//! in section 14, part N of the project's notes):
//! - culling: the game never sets 0x16, so Direct3D's default holds, triangles wound
//!   counter-clockwise on the screen are not drawn;
//! - the depth test (0x07 on, 0x17 the comparison) and its writing (0x0E);
//! - the texture (state 1 by `d3d::set_texture`), sampled at the nearest texel, or from
//!   the four around it when the filters ask for linear (0x11 at 2, 0x12 at 2, 5 or 6), with
//!   the perspective put right (u / w, v / w and 1 / w interpolated), wrapped (0x03
//!   at 1) or clamped (3); mixed with the colour by 0x15 (2 modulate, 4 modulate
//!   alpha, 1 and 7 the texture alone);
//! - the alpha test (0x0F on, 0x19 the comparison with 0x18);
//! - fog (0x1C on): the specular's alpha is how much of the colour stays, the rest is
//!   the fog colour (0x22);
//! - blending (0x1B on, 0x13 and 0x14 the factors).
//!
//! Pixels are sampled at their integer places, with the top-left rule for edges, as
//! Direct3D 5 did. Colours, the depth and 1/w are interpolated across the screen.

#![allow(unsafe_code)]

use std::cell::Cell;

/// A vertex: x, y, z, 1/w, colour, specular, u, v (floats as their bits but the two
/// colours).
pub type Vertex = [u32; 8];

/// A frame's colours: 16 (5-6-5) or 32 bits (XRGB) a pixel.
pub enum Pixels<'a> {
    Bits16(&'a mut [u16]),
    Bits32(&'a mut [u32]),
}

impl Pixels<'_> {
    fn len(&self) -> usize {
        match self {
            Pixels::Bits16(p) => p.len(),
            Pixels::Bits32(p) => p.len(),
        }
    }
}

/// Where it draws: the colours, rows `pitch` pixels apart, and the depths, a float a pixel.
/// Drawn into through its tiles (`tiles`), bands of whole rows, each holding its own rows
/// of both, so tiles can be drawn at once.
pub struct Frame<'a> {
    pixels: Pixels<'a>,
    pitch: usize,
    width: usize,
    height: usize,
    depth: &'a mut [f32],
}

impl<'a> Frame<'a> {
    /// The frame on `pixels` (`height` rows `pitch` pixels apart, `width` of them drawn) and
    /// `depth` (`width` x `height`). Too small a buffer for those ends the program.
    pub fn new(
        pixels: Pixels<'a>,
        pitch: usize,
        width: usize,
        height: usize,
        depth: &'a mut [f32],
    ) -> Self {
        let rows = if height == 0 {
            0
        } else {
            pitch * (height - 1) + width
        };
        assert!(
            pitch >= width && pixels.len() >= rows && depth.len() >= width * height,
            "rasteriser: a frame larger than its buffers"
        );
        Self {
            pixels,
            pitch,
            width,
            height,
            depth,
        }
    }

    /// The frame cut into bands of `rows` rows (one tile when `rows` is 0).
    pub fn tiles(&mut self, rows: usize) -> Vec<Tile<'_>> {
        let (pitch, width, height) = (self.pitch, self.width, self.height);
        let rows = if rows == 0 { height.max(1) } else { rows };
        let mut tiles = Vec::with_capacity(height.div_ceil(rows));
        let mut depth: &mut [f32] = &mut self.depth[..width * height];
        let mut pixels = match &mut self.pixels {
            Pixels::Bits16(p) => Pixels::Bits16(&mut p[..]),
            Pixels::Bits32(p) => Pixels::Bits32(&mut p[..]),
        };
        let mut top = 0;
        while top < height {
            let bottom = (top + rows).min(height);
            // The band's rows: the last only as far as its drawn pixels go.
            let len = pitch * (bottom - top - 1) + width;
            let take = if bottom == height {
                len
            } else {
                pitch * (bottom - top)
            };
            let (band, rest) = match pixels {
                Pixels::Bits16(p) => {
                    let (a, b) = p.split_at_mut(take);
                    (Pixels::Bits16(a), Pixels::Bits16(b))
                }
                Pixels::Bits32(p) => {
                    let (a, b) = p.split_at_mut(take);
                    (Pixels::Bits32(a), Pixels::Bits32(b))
                }
            };
            pixels = rest;
            let (d, rest) = std::mem::take(&mut depth).split_at_mut(width * (bottom - top));
            depth = rest;
            tiles.push(Tile {
                pixels: band,
                depth: d,
                pitch,
                width,
                height,
                clip: Clip {
                    left: 0,
                    top,
                    right: width,
                    bottom,
                },
            });
            top = bottom;
        }
        tiles
    }
}

/// A band of a frame's rows, its colours and depths its own.
pub struct Tile<'a> {
    pixels: Pixels<'a>,
    depth: &'a mut [f32],
    pitch: usize,
    width: usize,
    height: usize,
    clip: Clip,
}

impl Tile<'_> {
    pub fn clip(&self) -> &Clip {
        &self.clip
    }

    /// The tile's places as cells, for the drawing (which reads and writes them through
    /// shared references).
    fn cells(&mut self) -> Cells<'_> {
        let bits = match self.pixels {
            Pixels::Bits16(_) => 16,
            Pixels::Bits32(_) => 32,
        };
        Cells {
            pixels: match &mut self.pixels {
                Pixels::Bits16(p) => {
                    CellPixels::Bits16(Cell::from_mut(&mut p[..]).as_slice_of_cells())
                }
                Pixels::Bits32(p) => {
                    CellPixels::Bits32(Cell::from_mut(&mut p[..]).as_slice_of_cells())
                }
            },
            depth: Cell::from_mut(&mut self.depth[..]).as_slice_of_cells(),
            pitch: self.pitch,
            width: self.width,
            height: self.height,
            bits,
            clip: self.clip,
        }
    }

    /// `draw_fan` within the tile only (tiles drawn at once give the same pixels as the
    /// whole, each pixel's triangles still in their order). Returns the pixels written.
    pub fn draw_fan(
        &mut self,
        vertices: &[Vertex],
        states: &States,
        texture: Option<&Texture>,
    ) -> u64 {
        draw_fan_in(vertices, &self.cells(), states, texture)
    }

    /// The depths of `[left, top, right, bottom)` within the tile set to 1 (far).
    pub fn clear_depth(&mut self, [left, top, right, bottom]: [usize; 4]) {
        let cells = self.cells();
        let c = cells.clip;
        for y in top.max(c.top)..bottom.min(c.bottom) {
            for x in left.max(c.left)..right.min(c.right) {
                cells.set_depth(x, y, 1.0);
            }
        }
    }
}

/// A tile's colours as cells.
enum CellPixels<'a> {
    Bits16(&'a [Cell<u16>]),
    Bits32(&'a [Cell<u32>]),
}

/// A tile's places while it is drawn: its colours and depths as cells (from the tile's own
/// `&mut`), every place checked to lie in its band.
struct Cells<'a> {
    pixels: CellPixels<'a>,
    depth: &'a [Cell<f32>],
    pitch: usize,
    width: usize,
    height: usize,
    bits: u32,
    clip: Clip,
}

impl Cells<'_> {
    /// Whether `n` places from (x, y) on its row lie in the tile; a place outside is a slip
    /// of the drawing code, which ends the program.
    #[inline(always)]
    fn check(&self, x: usize, y: usize, n: usize) {
        let c = &self.clip;
        assert!(
            y >= c.top && y < c.bottom && x >= c.left && x + n <= c.right,
            "rasteriser: a place outside its tile"
        );
    }

    #[inline(always)]
    fn depth(&self, x: usize, y: usize) -> f32 {
        self.check(x, y, 1);
        self.depth[(y - self.clip.top) * self.width + x].get()
    }

    #[inline(always)]
    fn set_depth(&self, x: usize, y: usize, z: f32) {
        self.check(x, y, 1);
        self.depth[(y - self.clip.top) * self.width + x].set(z);
    }

    /// The raw pixel at (x, y): 16 or 32 bits.
    #[inline(always)]
    fn pixel(&self, x: usize, y: usize) -> u32 {
        self.check(x, y, 1);
        let at = (y - self.clip.top) * self.pitch + x;
        match self.pixels {
            CellPixels::Bits16(p) => u32::from(p[at].get()),
            CellPixels::Bits32(p) => p[at].get(),
        }
    }

    /// The raw pixel at (x, y) written: 16 or 32 bits.
    #[inline(always)]
    fn put_pixel(&self, x: usize, y: usize, v: u32) {
        self.check(x, y, 1);
        let at = (y - self.clip.top) * self.pitch + x;
        match self.pixels {
            CellPixels::Bits16(p) => p[at].set(v as u16),
            CellPixels::Bits32(p) => p[at].set(v),
        }
    }

    fn read(&self, x: usize, y: usize) -> Colour {
        let p = self.pixel(x, y);
        if self.bits == 16 {
            [part(p, 0x001F), part(p, 0x07E0), part(p, 0xF800), 1.0]
        } else {
            let mut colour = unpack(p);
            colour[3] = 1.0;
            colour
        }
    }

    fn write(&self, x: usize, y: usize, colour: &Colour) {
        let to = |c: f32, most: f32| (c.clamp(0.0, 1.0) * most + 0.5) as i32 as u32;
        let p = if self.bits == 16 {
            to(colour[2], 31.0) << 11 | to(colour[1], 63.0) << 5 | to(colour[0], 31.0)
        } else {
            to(colour[2], 255.0) << 16 | to(colour[1], 255.0) << 8 | to(colour[0], 255.0)
        };
        self.put_pixel(x, y, p);
    }
}

/// The 16-bit texel at byte `at` of a texture's bytes (a place past them ends the program).
#[inline(always)]
fn texel16(pixels: &[u8], at: usize) -> u32 {
    u32::from(u16::from_ne_bytes(
        *pixels[at..]
            .first_chunk()
            .expect("rasteriser: a texel past its texture"),
    ))
}

/// The 32-bit texel at byte `at` of a texture's bytes.
#[inline(always)]
fn texel32(pixels: &[u8], at: usize) -> u32 {
    u32::from_ne_bytes(
        *pixels[at..]
            .first_chunk()
            .expect("rasteriser: a texel past its texture"),
    )
}

/// A texture's first level: rows `pitch` bytes apart, `bytes` a pixel, the four masks
/// blue, green, red, alpha.
pub struct Texture<'a> {
    pub pixels: &'a [u8],
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bytes: usize,
    pub masks: [u32; 4],
}

/// The render states, by their numbers, as the game last set them.
pub struct States<'a>(pub &'a [u32]);

impl States<'_> {
    fn get(&self, state: usize) -> u32 {
        self.0.get(state).copied().unwrap_or(0)
    }
}

/// A colour as four floats from 0 to 1: blue, green, red, alpha.
type Colour = [f32; 4];

fn unpack(argb: u32) -> Colour {
    [
        (argb & 0xFF) as f32 * INV_255,
        (argb >> 8 & 0xFF) as f32 * INV_255,
        (argb >> 16 & 0xFF) as f32 * INV_255,
        (argb >> 24) as f32 * INV_255,
    ]
}

/// A part of a pixel by its mask, from 0 to 1; 1 for a mask of nothing.
fn part(pixel: u32, mask: u32) -> f32 {
    if mask == 0 {
        return 1.0;
    }
    let most = mask >> mask.trailing_zeros();
    ((pixel & mask) >> mask.trailing_zeros()) as f32 * (1.0 / most as f32)
}

impl Texture<'_> {
    /// A texel place along an axis of `size`: wrapped or clamped.
    fn place(n: i32, size: usize, clamp: bool) -> usize {
        if clamp {
            n.clamp(0, size as i32 - 1) as usize
        } else if size.is_power_of_two() {
            (n & (size as i32 - 1)) as usize
        } else {
            n.rem_euclid(size as i32) as usize
        }
    }

    /// The raw pixel of texel (x, y), wrapped or clamped.
    fn texel(&self, x: i32, y: i32, clamp: bool) -> u32 {
        let (x, y) = (
            Self::place(x, self.width, clamp),
            Self::place(y, self.height, clamp),
        );
        let place = y * self.pitch + x * self.bytes;
        match self.bytes {
            2 => self
                .pixels
                .get(place..place + 2)
                .map_or(0, |b| u16::from_le_bytes([b[0], b[1]]) as u32),
            _ => self
                .pixels
                .get(place..place + 4)
                .map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        }
    }

    fn has_alpha(&self) -> bool {
        self.masks[3] != 0
    }
}

fn compare(function: u32, value: f32, reference: f32) -> bool {
    match function {
        1 => false,
        2 => value < reference,
        3 => value == reference,
        4 => value <= reference,
        5 => value > reference,
        6 => value != reference,
        7 => value >= reference,
        _ => true,
    }
}

fn factor(blend: u32, source: &Colour, target: &Colour) -> Colour {
    match blend {
        1 => [0.0; 4],
        3 => *source,
        4 => source.map(|c| 1.0 - c),
        5 => [source[3]; 4],
        6 => [1.0 - source[3]; 4],
        7 => [target[3]; 4],
        8 => [1.0 - target[3]; 4],
        9 => *target,
        10 => target.map(|c| 1.0 - c),
        11 => {
            let f = source[3].min(1.0 - target[3]);
            [f, f, f, 1.0]
        }
        _ => [1.0; 4],
    }
}

#[derive(Clone, Copy, Default)]
struct Point {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    u: f32,
    v: f32,
    colour: Colour,
    specular: Colour,
}

fn point(v: &Vertex) -> Point {
    let f = |n: usize| f32::from_bits(v[n]);
    Point {
        x: f(0),
        y: f(1),
        z: f(2),
        w: f(3),
        u: f(6),
        v: f(7),
        colour: unpack(v[4]),
        specular: unpack(v[5]),
    }
}

/// An edge's function and whether a point on it counts (the top-left rule).
fn edge(a: &Point, b: &Point, x: f32, y: f32) -> f32 {
    (b.x - a.x) * (y - a.y) - (b.y - a.y) * (x - a.x)
}

/// For a triangle wound clockwise on the screen: a top edge runs to the right, a left
/// edge upwards.
fn top_left(a: &Point, b: &Point) -> bool {
    (a.y == b.y && b.x > a.x) || b.y < a.y
}

/// A rectangle of the target: places from (left, top) up to (right, bottom).
#[derive(Clone, Copy)]
pub struct Clip {
    pub left: usize,
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
}

/// `Tile::draw_fan`.
fn draw_fan_in(
    vertices: &[Vertex],
    tile: &Cells,
    states: &States,
    texture: Option<&Texture>,
) -> u64 {
    if vertices.len() < 3 {
        return 0;
    }
    // On the stack for a fan of up to 8 corners (the game's are), else on the heap.
    let mut few = [Point::default(); 8];
    let many: Vec<Point>;
    let points: &[Point] = if vertices.len() <= few.len() {
        for (to, v) in few.iter_mut().zip(vertices) {
            *to = point(v);
        }
        &few[..vertices.len()]
    } else {
        many = vertices.iter().map(point).collect();
        &many
    };
    let mut written = 0;
    for n in 1..points.len() - 1 {
        written += draw_triangle(
            [&points[0], &points[n], &points[n + 1]],
            tile,
            states,
            texture,
        );
    }
    written
}

fn draw_triangle(
    [a, b, c]: [&Point; 3],
    tile: &Cells,
    states: &States,
    texture: Option<&Texture>,
) -> u64 {
    let (clip, width, height) = (&tile.clip, tile.width, tile.height);
    // Positive: clockwise on the screen (y goes down).
    let area = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
    if area == 0.0 || !area.is_finite() {
        return 0;
    }
    let cull = match states.get(0x16) {
        0 => 3,
        cull => cull,
    };
    if (cull == 3 && area < 0.0) || (cull == 2 && area > 0.0) {
        return 0;
    }
    // Wound the same way whatever the input, for the edges' signs.
    let (a, b, c, area) = if area < 0.0 {
        (a, c, b, -area)
    } else {
        (a, b, c, area)
    };
    let left = (a.x.min(b.x).min(c.x).floor().max(0.0) as usize).max(clip.left);
    let right =
        ((a.x.max(b.x).max(c.x).ceil() as i32).clamp(0, width as i32) as usize).min(clip.right);
    let top = (a.y.min(b.y).min(c.y).floor().max(0.0) as usize).max(clip.top);
    let bottom =
        ((a.y.max(b.y).max(c.y).ceil() as i32).clamp(0, height as i32) as usize).min(clip.bottom);
    if left >= right || top >= bottom {
        return 0;
    }
    let edges = [(b, c), (c, a), (a, b)];
    let triangle = Triangle {
        points: [a, b, c],
        inverse_area: 1.0 / area,
        uw: [a.u * a.w, b.u * b.w, c.u * c.w],
        vw: [a.v * a.w, b.v * b.w, c.v * c.w],
        edges,
        biases: edges.map(|(p, q)| top_left(p, q)),
        left,
        right,
        depth_test: states.get(0x07) != 0,
        depth_function: states.get(0x17),
        depth_write: states.get(0x0E) != 0,
        alpha_test: states.get(0x0F) != 0,
        alpha_function: states.get(0x19),
        alpha_reference: (states.get(0x18) & 0xFF) as f32 / 255.0,
        blending: states.get(0x1B) != 0,
        source_blend: states.get(0x13),
        target_blend: states.get(0x14),
        fog: states.get(0x1C) != 0,
        fog_colour: unpack(states.get(0x22)),
        mixing: states.get(0x15),
        // linear when either filter asks for it (0x11 magnifying 2, 0x12 minifying 2, 5
        // or 6): which applies would be the pixel's footprint; the game sets both alike
        linear: states.get(0x11) == 2 || matches!(states.get(0x12), 2 | 5 | 6),
        texture: texture.map(|t| Texels::of(t, states.get(0x03) == 3)),
        tile,
    };
    let mut written = 0;
    for y in top..bottom {
        written += triangle.row(y);
    }
    written
}

/// `x.floor()` as a whole number, inline (`floor` is a library call on processors without
/// SSE4.1, which the 32-bit program assumes): the same for |x| below 2^31.
#[inline(always)]
fn floor(x: f32) -> i32 {
    let i = x as i32;
    if (i as f32) > x { i - 1 } else { i }
}

/// 1 / 255: channels are scaled by multiplying (a division is several times dearer).
const INV_255: f32 = 1.0 / 255.0;

/// 0 .. 255 as 0 .. 1.
const UNIT: [f32; 256] = {
    let mut t = [0.0f32; 256];
    let mut n = 0;
    while n < 256 {
        t[n] = n as f32 * INV_255;
        n += 1;
    }
    t
};

/// The 16-bit formats' tables met so far: each pixel's 0xAARRGGBB, by the four masks.
type Table = std::sync::Arc<Vec<u32>>;
static TABLES: std::sync::Mutex<Vec<([u32; 4], Table)>> = std::sync::Mutex::new(Vec::new());

/// A pixel of `masks` as 0xAARRGGBB (a channel without a mask: 0xFF).
pub(crate) fn to_argb(pixel: u32, masks: &[u32; 4]) -> u32 {
    let mut out = 0;
    // masks: blue, green, red, alpha; out's bytes: blue, green, red, alpha
    for (n, &mask) in masks.iter().enumerate() {
        let c = if mask == 0 {
            255
        } else {
            let shift = mask.trailing_zeros();
            let most = mask >> shift;
            (((pixel & mask) >> shift) * 255 + most / 2) / most
        };
        out |= c << (8 * n);
    }
    out
}

pub(crate) fn table_for(masks: &[u32; 4]) -> Table {
    let mut tables = TABLES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((_, t)) = tables.iter().find(|(m, _)| m == masks) {
        return t.clone();
    }
    let t: Table = std::sync::Arc::new((0..0x1_0000u32).map(|p| to_argb(p, masks)).collect());
    tables.push((*masks, t.clone()));
    t
}

/// How a texture's raw pixel becomes 0xAARRGGBB.
enum Kind {
    /// A 16-bit format: its table.
    Table(Table),
    /// A 32-bit A8R8G8B8 (or X8R8G8B8: the alpha made full by this).
    Usual(u32),
    /// Any other: channel by channel.
    Other,
}

/// A texture read as 0xAARRGGBB, everything a texel's read needs worked out once for the
/// triangle: its place, wrapped (by masks for sizes of powers of two) or clamped, read
/// without a check when the texture's bytes hold every place (else checked).
struct Texels<'a> {
    texture: &'a Texture<'a>,
    kind: Kind,
    clamp: bool,
    /// The masks of the wrap, for sizes of powers of two.
    masks: Option<(i32, i32)>,
    /// Every place is within the bytes.
    whole: bool,
    /// The pitch's and a texel's bytes as shifts, when both are powers of two.
    #[cfg_attr(
        not(any(target_arch = "x86", target_arch = "x86_64")),
        allow(dead_code)
    )] // the SSE path's
    shifts: Option<(i32, i32)>,
}

impl<'a> Texels<'a> {
    fn of(texture: &'a Texture<'a>, clamp: bool) -> Self {
        let m = texture.masks;
        let kind = if texture.bytes == 2 {
            Kind::Table(table_for(&m))
        } else if texture.bytes == 4
            && m[0] == 0xFF
            && m[1] == 0xFF00
            && m[2] == 0xFF_0000
            && (m[3] == 0 || m[3] == 0xFF00_0000)
        {
            Kind::Usual(if m[3] == 0 { 0xFF00_0000 } else { 0 })
        } else {
            Kind::Other
        };
        let (w, h) = (texture.width.max(1), texture.height.max(1));
        let needed = texture.pitch * (h - 1) + w * texture.bytes.max(1);
        Self {
            texture,
            kind,
            clamp,
            masks: (w.is_power_of_two() && h.is_power_of_two())
                .then_some((w as i32 - 1, h as i32 - 1)),
            whole: matches!(texture.bytes, 2 | 4) && texture.pixels.len() >= needed,
            shifts: (texture.pitch.is_power_of_two() && texture.bytes.is_power_of_two()).then(
                || {
                    (
                        texture.pitch.trailing_zeros() as i32,
                        texture.bytes.trailing_zeros() as i32,
                    )
                },
            ),
        }
    }

    /// Texel (x, y), wrapped or clamped, as 0xAARRGGBB.
    #[inline(always)]
    fn argb(&self, x: i32, y: i32) -> u32 {
        let t = self.texture;
        let (x, y) = match (self.clamp, self.masks) {
            (true, _) => (
                x.clamp(0, t.width as i32 - 1) as usize,
                y.clamp(0, t.height as i32 - 1) as usize,
            ),
            (false, Some((mx, my))) => ((x & mx) as usize, (y & my) as usize),
            (false, None) => (
                x.rem_euclid(t.width.max(1) as i32) as usize,
                y.rem_euclid(t.height.max(1) as i32) as usize,
            ),
        };
        let place = y * t.pitch + x * t.bytes;
        let raw = if self.whole {
            // `whole`: the bytes hold every place (checked in `of`), so the reads are inside.
            if t.bytes == 2 {
                texel16(t.pixels, place)
            } else {
                texel32(t.pixels, place)
            }
        } else {
            t.texel(x as i32, y as i32, true)
        };
        match &self.kind {
            Kind::Table(table) => table[raw as usize & 0xFFFF],
            Kind::Usual(full) => raw | full,
            Kind::Other => to_argb(raw, &t.masks),
        }
    }

    /// Four texels' 0xAARRGGBB (the lanes of `live`; the others 0): `argb` four times, but
    /// the usual cases (a whole texture, wrapped by powers of two or clamped, 32-bit
    /// A8R8G8B8 or a 16-bit table) chosen once for the four.
    #[inline(always)]
    #[cfg_attr(
        not(any(target_arch = "x86", target_arch = "x86_64")),
        allow(dead_code)
    )] // the SSE path's
    fn fetch4(&self, xs: &[i32; 4], ys: &[i32; 4], live: i32) -> [u32; 4] {
        let t = self.texture;
        let mut out = [0u32; 4];
        let place = |x: i32, y: i32| -> usize {
            match (self.clamp, self.masks) {
                (true, _) => {
                    y.clamp(0, t.height as i32 - 1) as usize * t.pitch
                        + x.clamp(0, t.width as i32 - 1) as usize * t.bytes
                }
                (false, Some((mx, my))) => {
                    (y & my) as usize * t.pitch + (x & mx) as usize * t.bytes
                }
                (false, None) => 0,
            }
        };
        let fast = self.whole && (self.clamp || self.masks.is_some());
        match (&self.kind, fast) {
            (Kind::Usual(full), true) => {
                for lane in 0..4 {
                    if live >> lane & 1 != 0 {
                        // `whole`, and the place wrapped or clamped into the texture.
                        out[lane] = texel32(t.pixels, place(xs[lane], ys[lane])) | full;
                    }
                }
            }
            (Kind::Table(table), true) => {
                for lane in 0..4 {
                    if live >> lane & 1 != 0 {
                        out[lane] = table[texel16(t.pixels, place(xs[lane], ys[lane])) as usize];
                    }
                }
            }
            _ => {
                for lane in 0..4 {
                    if live >> lane & 1 != 0 {
                        out[lane] = self.argb(xs[lane], ys[lane]);
                    }
                }
            }
        }
        out
    }

    fn colour(argb: u32) -> Colour {
        [
            UNIT[(argb & 0xFF) as usize],
            UNIT[(argb >> 8 & 0xFF) as usize],
            UNIT[(argb >> 16 & 0xFF) as usize],
            UNIT[(argb >> 24) as usize],
        ]
    }

    /// The texel at (u, v), the nearest one.
    fn nearest(&self, u: f32, v: f32) -> Colour {
        let t = self.texture;
        let x = floor(u * t.width as f32);
        let y = floor(v * t.height as f32);
        Self::colour(self.argb(x, y))
    }

    /// The colour at (u, v) filtered from the four texels around it (Direct3D's linear
    /// filter: texel centres at half places), with 8-bit weights, two channels at a time.
    fn bilinear(&self, u: f32, v: f32) -> Colour {
        let t = self.texture;
        let tx = u * t.width as f32 - 0.5;
        let ty = v * t.height as f32 - 0.5;
        let (x, y) = (floor(tx), floor(ty));
        let wx = ((tx - x as f32) * 256.0) as i32 as u32;
        let wy = ((ty - y as f32) * 256.0) as i32 as u32;
        let lerp = |a: u32, b: u32, w: u32| -> u32 {
            let rb = ((a & 0x00FF_00FF) * (256 - w) + (b & 0x00FF_00FF) * w) >> 8 & 0x00FF_00FF;
            let ag =
                ((a >> 8 & 0x00FF_00FF) * (256 - w) + (b >> 8 & 0x00FF_00FF) * w) & 0xFF00_FF00;
            rb | ag
        };
        let top = lerp(self.argb(x, y), self.argb(x + 1, y), wx);
        let bottom = lerp(self.argb(x, y + 1), self.argb(x + 1, y + 1), wx);
        Self::colour(lerp(top, bottom, wy))
    }
}

struct Triangle<'a> {
    points: [&'a Point; 3],
    /// 1 over twice the area: an edge's function times it is the opposite corner's weight.
    inverse_area: f32,
    /// Each corner's u / w and v / w (u and v times its 1/w).
    uw: [f32; 3],
    vw: [f32; 3],
    edges: [(&'a Point, &'a Point); 3],
    biases: [bool; 3],
    left: usize,
    right: usize,
    depth_test: bool,
    depth_function: u32,
    depth_write: bool,
    alpha_test: bool,
    alpha_function: u32,
    alpha_reference: f32,
    blending: bool,
    linear: bool,
    source_blend: u32,
    target_blend: u32,
    fog: bool,
    fog_colour: Colour,
    mixing: u32,
    texture: Option<Texels<'a>>,
    tile: &'a Cells<'a>,
}

impl Triangle<'_> {
    /// The places of row `y` that may be inside: where each edge's function, linear in x,
    /// is not below 0, widened by a pixel each way (the exact test decides each place).
    fn span(&self, py: f32) -> (usize, usize) {
        let (mut lo, mut hi) = (self.left as f32, self.right as f32);
        for (p, q) in self.edges {
            // e(x) = c - d (x - p.x)
            let (c, d) = ((q.x - p.x) * (py - p.y), q.y - p.y);
            if d > 0.0 {
                let bound = p.x + c / d;
                if bound.is_finite() {
                    hi = hi.min(bound.ceil() + 1.0);
                }
            } else if d < 0.0 {
                let bound = p.x + c / d;
                if bound.is_finite() {
                    lo = lo.max(bound.floor() - 1.0);
                }
            } else if c < 0.0 {
                return (0, 0);
            }
        }
        let lo = (lo.max(self.left as f32)) as usize;
        let hi = (hi.min(self.right as f32)).max(0.0) as usize;
        (lo, hi.max(lo))
    }

    /// The weights of the three corners at (px, py) if the place is inside (the exact
    /// test, with the top-left rule), else None.
    fn inside(&self, px: f32, py: f32) -> Option<[f32; 3]> {
        let mut weights = [0.0f32; 3];
        for (n, ((p, q), &bias)) in self.edges.iter().zip(&self.biases).enumerate() {
            let e = edge(p, q, px, py);
            if e < 0.0 || (e == 0.0 && !bias) {
                return None;
            }
            weights[n] = e * self.inverse_area;
        }
        Some(weights)
    }

    /// Returns the pixels written.
    ///
    /// The places inside a row are one run (the triangle is convex): the exact test finds
    /// its first and last place; between them every value the corners give is linear in x,
    /// so each is its value at the first place and its change a pixel (`at(k)`, k pixels on:
    /// no error adds up along the row).
    fn row(&self, y: usize) -> u64 {
        let [a, b, c] = self.points;
        let mut written = 0;
        let tile = self.tile;
        let py = y as f32;
        let (from, to) = self.span(py);
        let Some((first, weights)) =
            (from..to).find_map(|x| self.inside(x as f32, py).map(|w| (x, w)))
        else {
            return 0;
        };
        let last = (first..to)
            .rev()
            .find(|&x| self.inside(x as f32, py).is_some())
            .unwrap_or(first);
        // The weights' change a pixel: d e / d x = -(q.y - p.y).
        let steps: [f32; 3] = [0, 1, 2].map(|n| {
            let (p, q) = self.edges[n];
            -(q.y - p.y) * self.inverse_area
        });
        let [wa, wb, wc] = weights;
        let [sa, sb, sc] = steps;
        // (value at `first`, change a pixel)
        let line = |fa: f32, fb: f32, fc: f32| -> (f32, f32) {
            (wa * fa + wb * fb + wc * fc, sa * fa + sb * fb + sc * fc)
        };
        let z = line(a.z, b.z, c.z);
        let colour: [(f32, f32); 4] =
            [0, 1, 2, 3].map(|n| line(a.colour[n], b.colour[n], c.colour[n]));
        let fog = line(a.specular[3], b.specular[3], c.specular[3]);
        let w = line(a.w, b.w, c.w);
        let uw = line(self.uw[0], self.uw[1], self.uw[2]);
        let vw = line(self.vw[0], self.vw[1], self.vw[2]);
        let u = line(a.u, b.u, c.u);
        let v = line(a.v, b.v, c.v);
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
        if !SCALAR_ONLY.load(std::sync::atomic::Ordering::Relaxed) {
            let lines = Lines {
                z,
                colour,
                fog,
                w,
                uw,
                vw,
                u,
                v,
            };
            // NEON's are safe calls; SSE2's `target_feature` asks for `unsafe`.
            #[cfg_attr(target_arch = "aarch64", allow(unused_unsafe))]
            // SAFETY: SSE2 is in both x86 targets' baseline (i686 and x86_64).
            return unsafe { self.row4(y, first, last, &lines) };
        }
        for x in first..=last {
            let k = (x - first) as f32;
            let at = |(v0, d): (f32, f32)| v0 + d * k;
            let z = at(z);
            if self.depth_test && !compare(self.depth_function, z, tile.depth(x, y)) {
                continue;
            }
            let colour: Colour = colour.map(at);
            let mut out = colour;
            if let Some(texels) = &self.texture {
                let texture = texels.texture;
                let w = at(w);
                let (u, v) = if w != 0.0 {
                    let over = 1.0 / w;
                    (at(uw) * over, at(vw) * over)
                } else {
                    (at(u), at(v))
                };
                let texel = if self.linear {
                    texels.bilinear(u, v)
                } else {
                    texels.nearest(u, v)
                };
                out = match self.mixing {
                    1 | 7 => texel,
                    4 => [0, 1, 2, 3].map(|n| texel[n] * colour[n]),
                    _ => {
                        let alpha = if texture.has_alpha() {
                            texel[3]
                        } else {
                            colour[3]
                        };
                        [
                            texel[0] * colour[0],
                            texel[1] * colour[1],
                            texel[2] * colour[2],
                            alpha,
                        ]
                    }
                };
            }
            if self.alpha_test && !compare(self.alpha_function, out[3], self.alpha_reference) {
                continue;
            }
            if self.fog {
                let kept = (fog.0 + fog.1 * k).clamp(0.0, 1.0);
                for (o, f) in out.iter_mut().zip(self.fog_colour).take(3) {
                    *o = *o * kept + f * (1.0 - kept);
                }
            }
            if self.blending {
                let under = tile.read(x, y);
                let (s, d) = (
                    factor(self.source_blend, &out, &under),
                    factor(self.target_blend, &out, &under),
                );
                out = [0, 1, 2, 3].map(|n| out[n] * s[n] + under[n] * d[n]);
            }
            tile.write(x, y, &out);
            written += 1;
            if self.depth_write {
                tile.set_depth(x, y, z);
            }
        }
        written
    }
}

/// Draw every row one place at a time (the four-at-a-time path off): for comparing the
/// two, which give the same pixels.
#[cfg_attr(
    not(any(target_arch = "x86", target_arch = "x86_64")),
    allow(dead_code)
)] // the SSE path's
pub static SCALAR_ONLY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A row's values: each (at its first place, its change a pixel).
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
struct Lines {
    z: (f32, f32),
    colour: [(f32, f32); 4],
    fog: (f32, f32),
    w: (f32, f32),
    uw: (f32, f32),
    vw: (f32, f32),
    u: (f32, f32),
    v: (f32, f32),
}

#[cfg(target_arch = "aarch64")]
use crate::rasteriser_neon::*;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// A vector's lanes as an array and back, lane by lane (the compilers make these the one
// move they were as transmutes).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
#[target_feature(enable = "sse2")]
fn i32s(v: __m128i) -> [i32; 4] {
    [
        _mm_cvtsi128_si32(v),
        _mm_cvtsi128_si32(_mm_srli_si128::<4>(v)),
        _mm_cvtsi128_si32(_mm_srli_si128::<8>(v)),
        _mm_cvtsi128_si32(_mm_srli_si128::<12>(v)),
    ]
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
#[target_feature(enable = "sse2")]
fn from_u32s(a: [u32; 4]) -> __m128i {
    _mm_set_epi32(a[3] as i32, a[2] as i32, a[1] as i32, a[0] as i32)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
#[target_feature(enable = "sse2")]
fn f32s(v: __m128) -> [f32; 4] {
    [
        _mm_cvtss_f32(v),
        _mm_cvtss_f32(_mm_shuffle_ps::<0x55>(v, v)),
        _mm_cvtss_f32(_mm_shuffle_ps::<0xAA>(v, v)),
        _mm_cvtss_f32(_mm_shuffle_ps::<0xFF>(v, v)),
    ]
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
#[target_feature(enable = "sse2")]
fn from_f32s(a: [f32; 4]) -> __m128 {
    _mm_set_ps(a[3], a[2], a[1], a[0])
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn i32s(v: __m128i) -> [i32; 4] {
    lanes_i32(v)
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn from_u32s(a: [u32; 4]) -> __m128i {
    from_lanes_i32(a.map(|x| x as i32))
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn f32s(v: __m128) -> [f32; 4] {
    lanes_f32(v)
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn from_f32s(a: [f32; 4]) -> __m128 {
    from_lanes_f32(a)
}

/// `compare` on four lanes: their mask (all ones where it holds).
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
fn compare4(function: u32, value: __m128, reference: __m128) -> __m128 {
    match function {
        1 => _mm_setzero_ps(),
        2 => _mm_cmplt_ps(value, reference),
        3 => _mm_cmpeq_ps(value, reference),
        4 => _mm_cmple_ps(value, reference),
        5 => _mm_cmpgt_ps(value, reference),
        6 => _mm_cmpneq_ps(value, reference),
        7 => _mm_cmpge_ps(value, reference),
        _ => _mm_castsi128_ps(_mm_set1_epi32(-1)),
    }
}

/// `floor` on four lanes, as the scalar one: truncated, one less where that is above.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
fn floor4(x: __m128) -> __m128i {
    let i = _mm_cvttps_epi32(x);
    let above = _mm_castps_si128(_mm_cmpgt_ps(_mm_cvtepi32_ps(i), x));
    // above is -1 where the truncation is above: add it
    _mm_add_epi32(i, above)
}

/// Four lanes' 0xAARRGGBB as four channels (blue, green, red, alpha) of 0 .. 1, each
/// n x (1 / 255) as `UNIT` has it.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
fn channels4(argb: __m128i) -> [__m128; 4] {
    let byte = _mm_set1_epi32(0xFF);
    let most = _mm_set1_ps(INV_255);
    [
        _mm_mul_ps(_mm_cvtepi32_ps(_mm_and_si128(argb, byte)), most),
        _mm_mul_ps(
            _mm_cvtepi32_ps(_mm_and_si128(_mm_srli_epi32(argb, 8), byte)),
            most,
        ),
        _mm_mul_ps(
            _mm_cvtepi32_ps(_mm_and_si128(_mm_srli_epi32(argb, 16), byte)),
            most,
        ),
        _mm_mul_ps(_mm_cvtepi32_ps(_mm_srli_epi32(argb, 24)), most),
    ]
}

/// The scalar `lerp` of `Texels::bilinear` on four lanes: two channels at a time in 16-bit
/// halves (each product at most 255 x 256, so the same numbers).
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
fn lerp4(a: __m128i, b: __m128i, w: __m128i) -> __m128i {
    let mask = _mm_set1_epi32(0x00FF_00FF);
    // w in both halves of each lane, and 256 - w
    let w2 = _mm_or_si128(w, _mm_slli_epi32(w, 16));
    let rest = _mm_sub_epi16(_mm_set1_epi16(256), w2);
    let blend =
        |x: __m128i, y: __m128i| _mm_add_epi16(_mm_mullo_epi16(x, rest), _mm_mullo_epi16(y, w2));
    let rb = _mm_and_si128(
        _mm_srli_epi16(blend(_mm_and_si128(a, mask), _mm_and_si128(b, mask)), 8),
        mask,
    );
    let ag = _mm_andnot_si128(
        mask,
        blend(
            _mm_and_si128(_mm_srli_epi32(a, 8), mask),
            _mm_and_si128(_mm_srli_epi32(b, 8), mask),
        ),
    );
    _mm_or_si128(rb, ag)
}

/// `factor` on four lanes: each channel's factor (blue, green, red, alpha).
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
fn factor4(blend: u32, source: &[__m128; 4], target: &[__m128; 4]) -> [__m128; 4] {
    let one = _mm_set1_ps(1.0);
    let less = |x: __m128| _mm_sub_ps(one, x);
    match blend {
        1 => [_mm_setzero_ps(); 4],
        3 => *source,
        4 => source.map(less),
        5 => [source[3]; 4],
        6 => [less(source[3]); 4],
        7 => [target[3]; 4],
        8 => [less(target[3]); 4],
        9 => *target,
        10 => target.map(less),
        11 => {
            let f = _mm_min_ps(source[3], less(target[3]));
            [f, f, f, one]
        }
        _ => [one; 4],
    }
}

/// `read` on four places from x (lanes past `n` read as 0): the channels as 0 .. 1,
/// alpha 1, each by the same multiplication as the scalar one.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
impl Cells<'_> {
    #[inline]
    #[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
    fn read4(&self, x: usize, y: usize, n: usize) -> [__m128; 4] {
        let mut p = [0u32; 4];
        for (lane, p) in p.iter_mut().enumerate().take(n) {
            *p = self.pixel(x + lane, y);
        }
        let p = from_u32s(p);
        let part = |mask: i32, shift: i32, most: f32| {
            let v = _mm_and_si128(p, _mm_set1_epi32(mask));
            let v = match shift {
                0 => v,
                5 => _mm_srli_epi32(v, 5),
                8 => _mm_srli_epi32(v, 8),
                11 => _mm_srli_epi32(v, 11),
                _ => _mm_srli_epi32(v, 16),
            };
            _mm_mul_ps(_mm_cvtepi32_ps(v), _mm_set1_ps(1.0 / most))
        };
        let one = _mm_set1_ps(1.0);
        if self.bits == 16 {
            [
                part(0x1F, 0, 31.0),
                part(0x07E0, 5, 63.0),
                part(0xF800, 11, 31.0),
                one,
            ]
        } else {
            [
                part(0xFF, 0, 255.0),
                part(0xFF00, 8, 255.0),
                part(0xFF_0000, 16, 255.0),
                one,
            ]
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
impl Triangle<'_> {
    /// `row`'s places `first ..= last` four at a time with SSE2: the same
    /// numbers as the scalar loop, but the texels' reads, which stay one by one.
    #[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
    fn row4(&self, y: usize, first: usize, last: usize, l: &Lines) -> u64 {
        let tile = self.tile;
        let mut written = 0u64;
        let lanes = _mm_set_ps(3.0, 2.0, 1.0, 0.0);
        let mut x0 = first;
        while x0 <= last {
            let n = (last + 1 - x0).min(4);
            let k = _mm_add_ps(_mm_set1_ps((x0 - first) as f32), lanes);
            let at =
                |(v0, d): (f32, f32)| _mm_add_ps(_mm_set1_ps(v0), _mm_mul_ps(_mm_set1_ps(d), k));
            let mut live: i32 = (1 << n) - 1;
            let z = at(l.z);
            let mut old = [1.0f32; 4];
            for (lane, old) in old.iter_mut().enumerate().take(n) {
                *old = tile.depth(x0 + lane, y);
            }
            if self.depth_test {
                live &= _mm_movemask_ps(compare4(self.depth_function, z, from_f32s(old)));
            }
            if live == 0 {
                x0 += 4;
                continue;
            }
            let colour = l.colour.map(at);
            let mut out = colour;
            if let Some(texels) = &self.texture {
                let texture = texels.texture;
                let w = at(l.w);
                let zero_w = _mm_cmpeq_ps(w, _mm_setzero_ps());
                let over = _mm_div_ps(_mm_set1_ps(1.0), w);
                let pick = |perspective: __m128, plain: __m128| {
                    _mm_or_ps(
                        _mm_and_ps(zero_w, plain),
                        _mm_andnot_ps(zero_w, perspective),
                    )
                };
                let u = pick(_mm_mul_ps(at(l.uw), over), at(l.u));
                let v = pick(_mm_mul_ps(at(l.vw), over), at(l.v));
                let (tw, th) = (
                    _mm_set1_ps(texture.width as f32),
                    _mm_set1_ps(texture.height as f32),
                );
                let argb = if self.linear {
                    let half = _mm_set1_ps(0.5);
                    let tx = _mm_sub_ps(_mm_mul_ps(u, tw), half);
                    let ty = _mm_sub_ps(_mm_mul_ps(v, th), half);
                    let (ix, iy) = (floor4(tx), floor4(ty));
                    let scale = _mm_set1_ps(256.0);
                    let wx =
                        _mm_cvttps_epi32(_mm_mul_ps(_mm_sub_ps(tx, _mm_cvtepi32_ps(ix)), scale));
                    let wy =
                        _mm_cvttps_epi32(_mm_mul_ps(_mm_sub_ps(ty, _mm_cvtepi32_ps(iy)), scale));
                    let (xs, ys) = (i32s(ix), i32s(iy));
                    let [t00, t10, t01, t11] = match texels.quad4(ix, iy) {
                        Some(quad) => quad,
                        None => {
                            let (xs1, ys1) = (xs.map(|x| x + 1), ys.map(|y| y + 1));
                            [
                                texels.fetch4(&xs, &ys, live),
                                texels.fetch4(&xs1, &ys, live),
                                texels.fetch4(&xs, &ys1, live),
                                texels.fetch4(&xs1, &ys1, live),
                            ]
                        }
                    };
                    let load = |t: &[u32; 4]| from_u32s(*t);
                    let top = lerp4(load(&t00), load(&t10), wx);
                    let bottom = lerp4(load(&t01), load(&t11), wx);
                    lerp4(top, bottom, wy)
                } else {
                    let (ix, iy) = (floor4(_mm_mul_ps(u, tw)), floor4(_mm_mul_ps(v, th)));
                    let (xs, ys) = (i32s(ix), i32s(iy));
                    let t = texels.fetch4(&xs, &ys, live);
                    from_u32s(t)
                };
                let texel = channels4(argb);
                out = match self.mixing {
                    1 | 7 => texel,
                    4 => [0, 1, 2, 3].map(|c| _mm_mul_ps(texel[c], colour[c])),
                    _ => [
                        _mm_mul_ps(texel[0], colour[0]),
                        _mm_mul_ps(texel[1], colour[1]),
                        _mm_mul_ps(texel[2], colour[2]),
                        if texture.has_alpha() {
                            texel[3]
                        } else {
                            colour[3]
                        },
                    ],
                };
            }
            if self.alpha_test {
                live &= _mm_movemask_ps(compare4(
                    self.alpha_function,
                    out[3],
                    _mm_set1_ps(self.alpha_reference),
                ));
                if live == 0 {
                    x0 += 4;
                    continue;
                }
            }
            if self.fog {
                let kept = _mm_min_ps(_mm_max_ps(at(l.fog), _mm_setzero_ps()), _mm_set1_ps(1.0));
                let rest = _mm_sub_ps(_mm_set1_ps(1.0), kept);
                for (o, &f) in out.iter_mut().zip(&self.fog_colour).take(3) {
                    *o = _mm_add_ps(_mm_mul_ps(*o, kept), _mm_mul_ps(_mm_set1_ps(f), rest));
                }
            }
            if self.blending {
                let under = tile.read4(x0, y, n);
                let (sf, df) = (
                    factor4(self.source_blend, &out, &under),
                    factor4(self.target_blend, &out, &under),
                );
                out = [0, 1, 2, 3]
                    .map(|c| _mm_add_ps(_mm_mul_ps(out[c], sf[c]), _mm_mul_ps(under[c], df[c])));
            }
            // write_target's numbers: (clamped * most + 0.5) truncated
            let to = |c: __m128, most: f32| {
                let c = _mm_min_ps(_mm_max_ps(c, _mm_setzero_ps()), _mm_set1_ps(1.0));
                _mm_cvttps_epi32(_mm_add_ps(
                    _mm_mul_ps(c, _mm_set1_ps(most)),
                    _mm_set1_ps(0.5),
                ))
            };
            let packed = if tile.bits == 16 {
                _mm_or_si128(
                    _mm_or_si128(
                        _mm_slli_epi32(to(out[2], 31.0), 11),
                        _mm_slli_epi32(to(out[1], 63.0), 5),
                    ),
                    to(out[0], 31.0),
                )
            } else {
                _mm_or_si128(
                    _mm_or_si128(
                        _mm_slli_epi32(to(out[2], 255.0), 16),
                        _mm_slli_epi32(to(out[1], 255.0), 8),
                    ),
                    to(out[0], 255.0),
                )
            };
            let pixels = i32s(packed).map(|p| p as u32);
            let zs = f32s(z);
            for lane in 0..n {
                if live & (1 << lane) == 0 {
                    continue;
                }
                let x = x0 + lane;
                tile.put_pixel(x, y, pixels[lane]);
                if self.depth_write {
                    tile.set_depth(x, y, zs[lane]);
                }
                written += 1;
            }
            x0 += 4;
        }
        written
    }
}

/// SSE2's lane-wise `min` and `max` of signed dwords (SSE4.1 has them).
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
fn clamp4(v: __m128i, most: i32) -> __m128i {
    // not below 0: zero where the sign is set
    let v = _mm_andnot_si128(_mm_srai_epi32(v, 31), v);
    let most = _mm_set1_epi32(most);
    let over = _mm_cmpgt_epi32(v, most);
    _mm_or_si128(_mm_and_si128(over, most), _mm_andnot_si128(over, v))
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
impl Texels<'_> {
    /// The four corners' texels (x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1) for four
    /// lanes, as `argb` gives them, with the places worked out four at a time: for a whole
    /// texture of a pitch and texel of powers of two, wrapped by masks or clamped (every lane
    /// read: wrapping and clamping keep every place inside); else None.
    #[inline]
    #[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
    fn quad4(&self, x: __m128i, y: __m128i) -> Option<[[u32; 4]; 4]> {
        let (pitch_shift, byte_shift) = self.shifts?;
        if !self.whole || matches!(self.kind, Kind::Other) {
            return None;
        }
        let one = _mm_set1_epi32(1);
        let (x1, y1) = (_mm_add_epi32(x, one), _mm_add_epi32(y, one));
        let t = self.texture;
        let (x0, x1, y0, y1) = if self.clamp {
            let (w, h) = (t.width as i32 - 1, t.height as i32 - 1);
            (clamp4(x, w), clamp4(x1, w), clamp4(y, h), clamp4(y1, h))
        } else {
            let (mx, my) = self.masks?;
            let (mx, my) = (_mm_set1_epi32(mx), _mm_set1_epi32(my));
            (
                _mm_and_si128(x, mx),
                _mm_and_si128(x1, mx),
                _mm_and_si128(y, my),
                _mm_and_si128(y1, my),
            )
        };
        let shift = |v: __m128i, by: i32| _mm_sll_epi32(v, _mm_cvtsi32_si128(by));
        let (r0, r1) = (shift(y0, pitch_shift), shift(y1, pitch_shift));
        let (c0, c1) = (shift(x0, byte_shift), shift(x1, byte_shift));
        let places = [
            _mm_add_epi32(r0, c0),
            _mm_add_epi32(r0, c1),
            _mm_add_epi32(r1, c0),
            _mm_add_epi32(r1, c1),
        ];
        let mut out = [[0u32; 4]; 4];
        for (corner, p) in places.iter().enumerate() {
            let at = i32s(*p);
            for lane in 0..4 {
                // `whole` (every place of the texture within its bytes, `of`), and the places
                // wrapped by the masks or clamped into the texture above.
                let at = at[lane] as u32 as usize;
                out[corner][lane] = match &self.kind {
                    Kind::Usual(full) => texel32(t.pixels, at) | full,
                    Kind::Table(table) => table[texel16(t.pixels, at) as usize],
                    Kind::Other => 0,
                };
            }
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A frame of `w` x `h` 32-bit pixels drawn with one fan (blending and the depth test
    /// on), in tiles of `size` (0: one), scalar or not: its pixels and depths.
    fn draw(size: usize, scalar: bool) -> (Vec<u32>, Vec<f32>) {
        let (w, h) = (100, 70);
        let mut pixels = vec![0x0010_2030u32; w * h];
        let mut depth = vec![1.0f32; w * h];
        let v = |x: f32, y: f32, z: f32, colour: u32| -> Vertex {
            [
                x.to_bits(),
                y.to_bits(),
                z.to_bits(),
                1f32.to_bits(),
                colour,
                0xFF00_0000,
                0,
                0,
            ]
        };
        let fan = [
            v(3.5, 2.0, 0.5, 0x80FF_0000),
            v(95.0, 10.0, 0.2, 0x8000_FF00),
            v(70.0, 66.0, 0.7, 0x8000_00FF),
            v(10.0, 50.0, 0.4, 0xFFFF_FFFF),
        ];
        let mut states = [0u32; 0x40];
        states[0x07] = 1;
        states[0x17] = 4;
        states[0x0E] = 1;
        states[0x1B] = 1;
        states[0x13] = 5;
        states[0x14] = 6;
        states[0x16] = 1;
        SCALAR_ONLY.store(scalar, std::sync::atomic::Ordering::SeqCst);
        {
            let mut frame = Frame::new(Pixels::Bits32(&mut pixels), w, w, h, &mut depth);
            for tile in frame.tiles(size).iter_mut() {
                tile.draw_fan(&fan, &States(&states), None);
                tile.clear_depth([0, 0, 5, 5]);
            }
        }
        SCALAR_ONLY.store(false, std::sync::atomic::Ordering::SeqCst);
        (pixels, depth)
    }

    #[test]
    fn tiles_and_lanes_draw_the_same() {
        let whole = draw(0, true);
        assert!(whole.0.iter().any(|&p| p != 0x0010_2030), "something drawn");
        // The four-lane path and the scalar one: the same numbers.
        assert!(whole == draw(0, false));
        assert!(draw(16, false) == draw(16, true));
        // Tiles: the same colours; a row's values start from its first place in the tile,
        // so a depth can differ in its last bit.
        for tiled in [draw(16, false), draw(7, true)] {
            assert!(whole.0 == tiled.0);
            assert!(
                whole
                    .1
                    .iter()
                    .zip(&tiled.1)
                    .all(|(a, b)| (a - b).abs() < 1e-6)
            );
        }
    }
}
