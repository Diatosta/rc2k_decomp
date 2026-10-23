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

/// A vertex: x, y, z, 1/w, colour, specular, u, v (floats as their bits but the two
/// colours).
pub type Vertex = [u32; 8];

/// Where it draws: rows `pitch` bytes apart, 16 (5-6-5) or 32 bits (XRGB) a pixel.
pub struct Target {
    pub pixels: *mut u8,
    pub pitch: usize,
    pub width: usize,
    pub height: usize,
    pub bits: u32,
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

unsafe fn read_target(target: &Target, x: usize, y: usize) -> Colour {
    unsafe {
        let row = target.pixels.add(y * target.pitch);
        if target.bits == 16 {
            let p = (row.add(x * 2) as *const u16).read_unaligned() as u32;
            [part(p, 0x001F), part(p, 0x07E0), part(p, 0xF800), 1.0]
        } else {
            let mut colour = unpack((row.add(x * 4) as *const u32).read_unaligned());
            colour[3] = 1.0;
            colour
        }
    }
}

unsafe fn write_target(target: &Target, x: usize, y: usize, colour: &Colour) {
    let to = |c: f32, most: f32| (c.clamp(0.0, 1.0) * most + 0.5) as i32 as u32;
    unsafe {
        let row = target.pixels.add(y * target.pitch);
        if target.bits == 16 {
            let p = to(colour[2], 31.0) << 11 | to(colour[1], 63.0) << 5 | to(colour[0], 31.0);
            (row.add(x * 2) as *mut u16).write_unaligned(p as u16);
        } else {
            let p = to(colour[2], 255.0) << 16 | to(colour[1], 255.0) << 8 | to(colour[0], 255.0);
            (row.add(x * 4) as *mut u32).write_unaligned(p);
        }
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

/// `draw_fan` within `clip` only (a tile of the frame: tiles drawn at once give the same
/// pixels as the whole, each pixel's triangles still in their order).
///
/// # Safety
/// `target.pixels` must hold `target.height` rows of `target.pitch` bytes, `depth` a float
/// a place of the target; no other thread may draw into `clip` meanwhile.
/// Returns the pixels written.
pub unsafe fn draw_fan_in(
    vertices: &[Vertex],
    target: &Target,
    states: &States,
    texture: Option<&Texture>,
    depth: *mut f32,
    clip: &Clip,
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
        written += unsafe {
            draw_triangle(
                [&points[0], &points[n], &points[n + 1]],
                target,
                states,
                texture,
                depth,
                clip,
            )
        };
    }
    written
}

unsafe fn draw_triangle(
    [a, b, c]: [&Point; 3],
    target: &Target,
    states: &States,
    texture: Option<&Texture>,
    depth: *mut f32,
    clip: &Clip,
) -> u64 {
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
    let right = ((a.x.max(b.x).max(c.x).ceil() as i32).clamp(0, target.width as i32) as usize)
        .min(clip.right);
    let top = (a.y.min(b.y).min(c.y).floor().max(0.0) as usize).max(clip.top);
    let bottom = ((a.y.max(b.y).max(c.y).ceil() as i32).clamp(0, target.height as i32) as usize)
        .min(clip.bottom);
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
        target,
        depth,
    };
    let mut written = 0;
    for y in top..bottom {
        written += unsafe { triangle.row(y) };
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
fn to_argb(pixel: u32, masks: &[u32; 4]) -> u32 {
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

fn table_for(masks: &[u32; 4]) -> Table {
    let mut tables = TABLES.lock().unwrap_or_else(|e| e.into_inner());
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
            // Safety: `whole` says the bytes hold every place (checked in `of`).
            unsafe {
                let at = t.pixels.as_ptr().add(place);
                if t.bytes == 2 {
                    (at as *const u16).read_unaligned() as u32
                } else {
                    (at as *const u32).read_unaligned()
                }
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
        let base = t.pixels.as_ptr();
        match (&self.kind, fast) {
            (Kind::Usual(full), true) => {
                for lane in 0..4 {
                    if live >> lane & 1 != 0 {
                        // Safety: `whole`, and the place wrapped or clamped into the texture.
                        let raw = unsafe {
                            (base.add(place(xs[lane], ys[lane])) as *const u32).read_unaligned()
                        };
                        out[lane] = raw | full;
                    }
                }
            }
            (Kind::Table(table), true) => {
                for lane in 0..4 {
                    if live >> lane & 1 != 0 {
                        let raw = unsafe {
                            (base.add(place(xs[lane], ys[lane])) as *const u16).read_unaligned()
                        };
                        out[lane] = table[raw as usize];
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
    target: &'a Target,
    depth: *mut f32,
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
    unsafe fn row(&self, y: usize) -> u64 {
        let [a, b, c] = self.points;
        let mut written = 0;
        let target = self.target;
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
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
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
            return unsafe { self.row4(y, first, last, &lines) };
        }
        for x in first..=last {
            let k = (x - first) as f32;
            let at = |(v0, d): (f32, f32)| v0 + d * k;
            let z = at(z);
            let place = y * target.width + x;
            let depth = unsafe { self.depth.add(place) };
            if self.depth_test && !compare(self.depth_function, z, unsafe { *depth }) {
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
                let under = unsafe { read_target(target, x, y) };
                let (s, d) = (
                    factor(self.source_blend, &out, &under),
                    factor(self.target_blend, &out, &under),
                );
                out = [0, 1, 2, 3].map(|n| out[n] * s[n] + under[n] * d[n]);
            }
            unsafe { write_target(target, x, y, &out) };
            written += 1;
            if self.depth_write {
                unsafe { *depth = z };
            }
        }
        written
    }
}

/// Draw every row one place at a time (the four-at-a-time path off): for comparing the
/// two, which give the same pixels.
pub static SCALAR_ONLY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A row's values: each (at its first place, its change a pixel).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
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

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// `compare` on four lanes: their mask (all ones where it holds).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn compare4(function: u32, value: __m128, reference: __m128) -> __m128 {
    unsafe {
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
}

/// `floor` on four lanes, as the scalar one: truncated, one less where that is above.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn floor4(x: __m128) -> __m128i {
    unsafe {
        let i = _mm_cvttps_epi32(x);
        let above = _mm_castps_si128(_mm_cmpgt_ps(_mm_cvtepi32_ps(i), x));
        // above is -1 where the truncation is above: add it
        _mm_add_epi32(i, above)
    }
}

/// Four lanes' 0xAARRGGBB as four channels (blue, green, red, alpha) of 0 .. 1, each
/// n x (1 / 255) as `UNIT` has it.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn channels4(argb: __m128i) -> [__m128; 4] {
    unsafe {
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
}

/// The scalar `lerp` of `Texels::bilinear` on four lanes: two channels at a time in 16-bit
/// halves (each product at most 255 x 256, so the same numbers).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn lerp4(a: __m128i, b: __m128i, w: __m128i) -> __m128i {
    unsafe {
        let mask = _mm_set1_epi32(0x00FF_00FF);
        // w in both halves of each lane, and 256 - w
        let w2 = _mm_or_si128(w, _mm_slli_epi32(w, 16));
        let rest = _mm_sub_epi16(_mm_set1_epi16(256), w2);
        let blend = |x: __m128i, y: __m128i| {
            _mm_add_epi16(_mm_mullo_epi16(x, rest), _mm_mullo_epi16(y, w2))
        };
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
}

/// `factor` on four lanes: each channel's factor (blue, green, red, alpha).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn factor4(blend: u32, source: &[__m128; 4], target: &[__m128; 4]) -> [__m128; 4] {
    unsafe {
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
}

/// `read_target` on four places from x (lanes past `n` read as 0): the channels as 0 .. 1,
/// alpha 1, each by the same multiplication as the scalar one.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn read_target4(target: &Target, x: usize, y: usize, n: usize) -> [__m128; 4] {
    unsafe {
        let row = target.pixels.add(y * target.pitch);
        let mut p = [0u32; 4];
        for (lane, p) in p.iter_mut().enumerate().take(n) {
            *p = if target.bits == 16 {
                (row.add((x + lane) * 2) as *const u16).read_unaligned() as u32
            } else {
                (row.add((x + lane) * 4) as *const u32).read_unaligned()
            };
        }
        let p = _mm_loadu_si128(p.as_ptr() as *const __m128i);
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
        if target.bits == 16 {
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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl Triangle<'_> {
    /// `row`'s places `first ..= last` four at a time with SSE2: the same
    /// numbers as the scalar loop, but the texels' reads, which stay one by one.
    unsafe fn row4(&self, y: usize, first: usize, last: usize, l: &Lines) -> u64 {
        unsafe {
            let target = self.target;
            let mut written = 0u64;
            let lanes = _mm_set_ps(3.0, 2.0, 1.0, 0.0);
            let mut x0 = first;
            while x0 <= last {
                let n = (last + 1 - x0).min(4);
                let k = _mm_add_ps(_mm_set1_ps((x0 - first) as f32), lanes);
                let at = |(v0, d): (f32, f32)| {
                    _mm_add_ps(_mm_set1_ps(v0), _mm_mul_ps(_mm_set1_ps(d), k))
                };
                let mut live: i32 = (1 << n) - 1;
                let z = at(l.z);
                let depth = self.depth.add(y * target.width + x0);
                let mut old = [1.0f32; 4];
                std::ptr::copy_nonoverlapping(depth, old.as_mut_ptr(), n);
                if self.depth_test {
                    live &= _mm_movemask_ps(compare4(
                        self.depth_function,
                        z,
                        _mm_loadu_ps(old.as_ptr()),
                    ));
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
                        let wx = _mm_cvttps_epi32(_mm_mul_ps(
                            _mm_sub_ps(tx, _mm_cvtepi32_ps(ix)),
                            scale,
                        ));
                        let wy = _mm_cvttps_epi32(_mm_mul_ps(
                            _mm_sub_ps(ty, _mm_cvtepi32_ps(iy)),
                            scale,
                        ));
                        let (mut xs, mut ys) = ([0i32; 4], [0i32; 4]);
                        _mm_storeu_si128(xs.as_mut_ptr() as *mut __m128i, ix);
                        _mm_storeu_si128(ys.as_mut_ptr() as *mut __m128i, iy);
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
                        let load = |t: &[u32; 4]| _mm_loadu_si128(t.as_ptr() as *const __m128i);
                        let top = lerp4(load(&t00), load(&t10), wx);
                        let bottom = lerp4(load(&t01), load(&t11), wx);
                        lerp4(top, bottom, wy)
                    } else {
                        let (ix, iy) = (floor4(_mm_mul_ps(u, tw)), floor4(_mm_mul_ps(v, th)));
                        let (mut xs, mut ys) = ([0i32; 4], [0i32; 4]);
                        _mm_storeu_si128(xs.as_mut_ptr() as *mut __m128i, ix);
                        _mm_storeu_si128(ys.as_mut_ptr() as *mut __m128i, iy);
                        let t = texels.fetch4(&xs, &ys, live);
                        _mm_loadu_si128(t.as_ptr() as *const __m128i)
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
                    let kept =
                        _mm_min_ps(_mm_max_ps(at(l.fog), _mm_setzero_ps()), _mm_set1_ps(1.0));
                    let rest = _mm_sub_ps(_mm_set1_ps(1.0), kept);
                    for (o, &f) in out.iter_mut().zip(&self.fog_colour).take(3) {
                        *o = _mm_add_ps(_mm_mul_ps(*o, kept), _mm_mul_ps(_mm_set1_ps(f), rest));
                    }
                }
                if self.blending {
                    let under = read_target4(target, x0, y, n);
                    let (sf, df) = (
                        factor4(self.source_blend, &out, &under),
                        factor4(self.target_blend, &out, &under),
                    );
                    out = [0, 1, 2, 3].map(|c| {
                        _mm_add_ps(_mm_mul_ps(out[c], sf[c]), _mm_mul_ps(under[c], df[c]))
                    });
                }
                // write_target's numbers: (clamped * most + 0.5) truncated
                let to = |c: __m128, most: f32| {
                    let c = _mm_min_ps(_mm_max_ps(c, _mm_setzero_ps()), _mm_set1_ps(1.0));
                    _mm_cvttps_epi32(_mm_add_ps(
                        _mm_mul_ps(c, _mm_set1_ps(most)),
                        _mm_set1_ps(0.5),
                    ))
                };
                let row = target.pixels.add(y * target.pitch);
                let mut pixels = [0u32; 4];
                let packed = if target.bits == 16 {
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
                _mm_storeu_si128(pixels.as_mut_ptr() as *mut __m128i, packed);
                let mut zs = [0.0f32; 4];
                _mm_storeu_ps(zs.as_mut_ptr(), z);
                for lane in 0..n {
                    if live & (1 << lane) == 0 {
                        continue;
                    }
                    let x = x0 + lane;
                    if target.bits == 16 {
                        (row.add(x * 2) as *mut u16).write_unaligned(pixels[lane] as u16);
                    } else {
                        (row.add(x * 4) as *mut u32).write_unaligned(pixels[lane]);
                    }
                    if self.depth_write {
                        *depth.add(lane) = zs[lane];
                    }
                    written += 1;
                }
                x0 += 4;
            }
            written
        }
    }
}

/// SSE2's lane-wise `min` and `max` of signed dwords (SSE4.1 has them).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn clamp4(v: __m128i, most: i32) -> __m128i {
    unsafe {
        // not below 0: zero where the sign is set
        let v = _mm_andnot_si128(_mm_srai_epi32(v, 31), v);
        let most = _mm_set1_epi32(most);
        let over = _mm_cmpgt_epi32(v, most);
        _mm_or_si128(_mm_and_si128(over, most), _mm_andnot_si128(over, v))
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl Texels<'_> {
    /// The four corners' texels (x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1) for four
    /// lanes, as `argb` gives them, with the places worked out four at a time: for a whole
    /// texture of a pitch and texel of powers of two, wrapped by masks or clamped (every lane
    /// read: wrapping and clamping keep every place inside); else None.
    #[inline(always)]
    unsafe fn quad4(&self, x: __m128i, y: __m128i) -> Option<[[u32; 4]; 4]> {
        unsafe {
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
            let base = t.pixels.as_ptr();
            let mut out = [[0u32; 4]; 4];
            for (corner, p) in places.iter().enumerate() {
                let mut at = [0u32; 4];
                _mm_storeu_si128(at.as_mut_ptr() as *mut __m128i, *p);
                for lane in 0..4 {
                    let at = base.add(at[lane] as usize);
                    out[corner][lane] = match &self.kind {
                        Kind::Usual(full) => (at as *const u32).read_unaligned() | full,
                        Kind::Table(table) => {
                            *table.get_unchecked((at as *const u16).read_unaligned() as usize)
                        }
                        Kind::Other => 0,
                    };
                }
            }
            Some(out)
        }
    }
}
