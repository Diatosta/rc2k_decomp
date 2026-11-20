//! A software rasteriser: what Direct3D 5 did with the
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

mod simd;
mod texels;

#[cfg(target_arch = "aarch64")]
use crate::render::rasteriser_neon::*;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use self::simd::*;
use self::texels::*;
pub(crate) use self::texels::{table_for, to_argb};
use std::cell::Cell;

/// A vertex: x, y, z, 1/w, colour, specular, u, v (floats as their bits but the two
/// colours).
pub type Vertex = [u32; 8];

/// Where it draws: the colours, rows `pitch` pixels apart, and the depths, a float a pixel.
/// Drawn into through its tiles (`tiles`), bands of whole rows, each holding its own rows
/// of both, so tiles can be drawn at once.
pub struct Frame<'a> {
    /// The colours, 32 bits (XRGB) a pixel.
    pixels: &'a mut [u32],
    pitch: usize,
    width: usize,
    height: usize,
    depth: &'a mut [f32],
}

impl<'a> Frame<'a> {
    /// The frame on `pixels` (`height` rows `pitch` pixels apart, `width` of them drawn) and
    /// `depth` (`width` x `height`). Too small a buffer for those ends the program.
    pub fn new(
        pixels: &'a mut [u32],
        pitch: usize,
        width: usize,
        height: usize,
        depth: &'a mut [f32],
    ) -> Self {
        // A size that overflows is larger than any buffer: refused like one that is too big.
        let rows = if height == 0 {
            Some(0)
        } else {
            pitch
                .checked_mul(height - 1)
                .and_then(|rows| rows.checked_add(width))
        };
        assert!(
            pitch >= width
                && rows.is_some_and(|rows| pixels.len() >= rows)
                && width
                    .checked_mul(height)
                    .is_some_and(|area| depth.len() >= area),
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
        let mut pixels: &mut [u32] = &mut self.pixels[..];
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
            let (band, rest) = std::mem::take(&mut pixels).split_at_mut(take);
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
    pixels: &'a mut [u32],
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
        Cells {
            pixels: Cell::from_mut(&mut self.pixels[..]).as_slice_of_cells(),
            depth: Cell::from_mut(&mut self.depth[..]).as_slice_of_cells(),
            pitch: self.pitch,
            width: self.width,
            height: self.height,
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

/// A tile's places while it is drawn: its colours and depths as cells (from the tile's own
/// `&mut`), every place checked to lie in its band.
struct Cells<'a> {
    pixels: &'a [Cell<u32>],
    depth: &'a [Cell<f32>],
    pitch: usize,
    width: usize,
    height: usize,
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

    /// The raw pixel at (x, y).
    #[inline(always)]
    fn pixel(&self, x: usize, y: usize) -> u32 {
        self.check(x, y, 1);
        self.pixels[(y - self.clip.top) * self.pitch + x].get()
    }

    /// The raw pixel at (x, y) written.
    #[inline(always)]
    fn put_pixel(&self, x: usize, y: usize, v: u32) {
        self.check(x, y, 1);
        self.pixels[(y - self.clip.top) * self.pitch + x].set(v);
    }

    fn read(&self, x: usize, y: usize) -> Colour {
        let mut colour = unpack(self.pixel(x, y));
        colour[3] = 1.0;
        colour
    }

    fn write(&self, x: usize, y: usize, colour: &Colour) {
        let to = |c: f32, most: f32| (c.clamp(0.0, 1.0) * most + 0.5) as i32 as u32;
        let p = to(colour[2], 255.0) << 16 | to(colour[1], 255.0) << 8 | to(colour[0], 255.0);
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
    /// The smaller levels (`mip_chain`) when the caller has them; empty draws the first level only.
    pub lower: &'a [MipLevel],
}

/// A texture's smaller level: 32-bit 0xAARRGGBB texels as little-endian bytes, rows without gaps.
pub struct MipLevel {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

/// The most smaller levels a chain holds (the game's textures reach 256 across).
const MOST_LEVELS: usize = 8;

/// The smaller levels of `texture` (halving each side down to one texel, at most `MOST_LEVELS`),
/// each texel the average of the four it covers in the level above (a box filter), per channel.
pub(super) fn mip_chain(texture: &Texture) -> Vec<MipLevel> {
    let base = Texels::of(texture, false);
    let (mut w, mut h) = (texture.width.max(1), texture.height.max(1));
    let mut above: Vec<u32> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| base.argb(x as i32, y as i32))
        .collect();
    let mut chain = Vec::new();
    while (w > 1 || h > 1) && chain.len() < MOST_LEVELS {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut level = Vec::with_capacity(nw * nh);
        for y in 0..nh {
            for x in 0..nw {
                let (x0, y0) = (x * 2, y * 2);
                let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
                let four = [
                    above[y0 * w + x0],
                    above[y0 * w + x1],
                    above[y1 * w + x0],
                    above[y1 * w + x1],
                ];
                let mut out = 0u32;
                for shift in [0, 8, 16, 24] {
                    let sum: u32 = four.iter().map(|p| p >> shift & 0xFF).sum();
                    out |= ((sum + 2) >> 2) << shift;
                }
                level.push(out);
            }
        }
        chain.push(MipLevel {
            width: nw,
            height: nh,
            pixels: level.iter().flat_map(|p| p.to_le_bytes()).collect(),
        });
        above = level;
        (w, h) = (nw, nh);
    }
    chain
}

/// How many levels down a triangle reads, from how its texels change along the screen's axes
/// (`[[du/dx, du/dy], [dv/dx, dv/dy]]`, in texels a pixel): the level whose texels are nearest
/// the footprint's narrow side, so a stretched triangle (a tall thin trunk, a road to the
/// horizon) keeps its sharpness across and only the stretched way is left to alias.
fn mip_level([[dudx, dudy], [dvdx, dvdy]]: [[f32; 2]; 2], lowest: usize) -> usize {
    // The squares of the footprint's two sides are the eigenvalues of J * J^T; the smaller one.
    let trace = dudx * dudx + dudy * dudy + dvdx * dvdx + dvdy * dvdy;
    let determinant = dudx * dvdy - dudy * dvdx;
    let spread = (trace * trace - 4.0 * determinant * determinant).max(0.0);
    let mut narrow = (trace - spread.sqrt()) * 0.5;
    let mut level = 0;
    // The level n is read from 2 * 4^n texels^2 a pixel^2 (half a level's step).
    while level < lowest && narrow >= 2.0 {
        narrow *= 0.25;
        level += 1;
    }
    level
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

impl Texture<'_> {
    /// A texel place along an axis of `size`: wrapped or clamped.
    fn place(n: i32, size: usize, clamp: bool) -> usize {
        // A texture of no width or height reads as one texel across: `clamp(0, -1)` would panic.
        let size = size.max(1);
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
    // linear when either filter asks for it (0x11 magnifying 2, 0x12 minifying 2, 5
    // or 6): which applies would be the pixel's footprint; the game sets both alike
    let linear = states.get(0x11) == 2 || matches!(states.get(0x12), 2 | 5 | 6);
    // The smaller levels, when the game asks for them (minifying filter 6) and the caller has them.
    let lower_texture;
    let mut texture = texture;
    if let Some(t) = texture
        && !t.lower.is_empty()
        && linear
        && states.get(0x12) == 6
    {
        // The texel changes along the triangle's two edges from `a`, turned into changes along
        // the screen's axes by the edges' own screen steps.
        let (w, h) = (t.width as f32, t.height as f32);
        let (du1, dv1) = ((b.u - a.u) * w, (b.v - a.v) * h);
        let (du2, dv2) = ((c.u - a.u) * w, (c.v - a.v) * h);
        let (ex1, ey1, ex2, ey2) = (b.x - a.x, b.y - a.y, c.x - a.x, c.y - a.y);
        let inverse = 1.0 / (ex1 * ey2 - ex2 * ey1);
        let level = mip_level(
            [
                [
                    (du1 * ey2 - du2 * ey1) * inverse,
                    (du2 * ex1 - du1 * ex2) * inverse,
                ],
                [
                    (dv1 * ey2 - dv2 * ey1) * inverse,
                    (dv2 * ex1 - dv1 * ex2) * inverse,
                ],
            ],
            t.lower.len(),
        );
        if level > 0 {
            let l = &t.lower[level - 1];
            lower_texture = Texture {
                pixels: &l.pixels,
                width: l.width,
                height: l.height,
                pitch: l.width * 4,
                bytes: 4,
                masks: [
                    0xFF,
                    0xFF00,
                    0xFF_0000,
                    if t.has_alpha() { 0xFF00_0000 } else { 0 },
                ],
                lower: &[],
            };
            texture = Some(&lower_texture);
        }
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
        linear,
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
            // SAFETY: the call's one requirement is `row4`'s `target_feature`, SSE2, which every
            // x86 build has (`SSE2_IN_THE_BUILD`, checked at compile time).
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
static SCALAR_ONLY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
mod tests;
