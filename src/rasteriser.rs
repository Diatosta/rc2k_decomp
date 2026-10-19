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
        (argb & 0xFF) as f32 / 255.0,
        (argb >> 8 & 0xFF) as f32 / 255.0,
        (argb >> 16 & 0xFF) as f32 / 255.0,
        (argb >> 24) as f32 / 255.0,
    ]
}

/// A part of a pixel by its mask, from 0 to 1; 1 for a mask of nothing.
fn part(pixel: u32, mask: u32) -> f32 {
    if mask == 0 {
        return 1.0;
    }
    let most = mask >> mask.trailing_zeros();
    ((pixel & mask) >> mask.trailing_zeros()) as f32 / most as f32
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

/// Draws a triangle fan (vertex 0 with each two neighbours).
///
/// # Safety
/// `target.pixels` must hold `target.height` rows of `target.pitch` bytes.
pub unsafe fn draw_fan(
    vertices: &[Vertex],
    target: &Target,
    states: &States,
    texture: Option<&Texture>,
    depth: &mut [f32],
) {
    if depth.len() < target.width * target.height {
        return;
    }
    let clip = Clip {
        left: 0,
        top: 0,
        right: target.width,
        bottom: target.height,
    };
    unsafe { draw_fan_in(vertices, target, states, texture, depth.as_mut_ptr(), &clip) };
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
    let points: Vec<Point> = vertices.iter().map(point).collect();
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
        clamp: states.get(0x03) == 3,
        // linear when either filter asks for it (0x11 magnifying 2, 0x12 minifying 2, 5
        // or 6): which applies would be the pixel's footprint; the game sets both alike
        linear: states.get(0x11) == 2 || matches!(states.get(0x12), 2 | 5 | 6),
        texture: texture.map(Texels::of),
        target,
        depth,
    };
    let mut written = 0;
    for y in top..bottom {
        written += unsafe { triangle.row(y) };
    }
    written
}

/// 0 .. 255 as 0 .. 1.
const UNIT: [f32; 256] = {
    let mut t = [0.0f32; 256];
    let mut n = 0;
    while n < 256 {
        t[n] = n as f32 / 255.0;
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

/// A texture read as 0xAARRGGBB: a 16-bit one through its format's table, a 32-bit one of
/// the usual masks as it is (the alpha made full where there is none), any other pixel by
/// pixel.
struct Texels<'a> {
    texture: &'a Texture<'a>,
    table: Option<Table>,
    /// 0xFF000000 for a 32-bit texture without alpha, else 0.
    full_alpha: u32,
    usual: bool,
}

impl<'a> Texels<'a> {
    fn of(texture: &'a Texture<'a>) -> Self {
        let m = texture.masks;
        let usual = texture.bytes == 4
            && m[0] == 0xFF
            && m[1] == 0xFF00
            && m[2] == 0xFF_0000
            && (m[3] == 0 || m[3] == 0xFF00_0000);
        Self {
            texture,
            table: (texture.bytes == 2).then(|| table_for(&m)),
            full_alpha: if m[3] == 0 { 0xFF00_0000 } else { 0 },
            usual,
        }
    }

    /// Texel (x, y), wrapped or clamped, as 0xAARRGGBB.
    fn argb(&self, x: i32, y: i32, clamp: bool) -> u32 {
        let raw = self.texture.texel(x, y, clamp);
        match &self.table {
            Some(t) => t[raw as usize & 0xFFFF],
            None if self.usual => raw | self.full_alpha,
            None => to_argb(raw, &self.texture.masks),
        }
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
    fn nearest(&self, u: f32, v: f32, clamp: bool) -> Colour {
        let t = self.texture;
        let x = (u * t.width as f32).floor() as i32;
        let y = (v * t.height as f32).floor() as i32;
        Self::colour(self.argb(x, y, clamp))
    }

    /// The colour at (u, v) filtered from the four texels around it (Direct3D's linear
    /// filter: texel centres at half places), with 8-bit weights, two channels at a time.
    fn bilinear(&self, u: f32, v: f32, clamp: bool) -> Colour {
        let t = self.texture;
        let tx = u * t.width as f32 - 0.5;
        let ty = v * t.height as f32 - 0.5;
        let (fx, fy) = (tx.floor(), ty.floor());
        let wx = ((tx - fx) * 256.0) as u32;
        let wy = ((ty - fy) * 256.0) as u32;
        let (x, y) = (fx as i32, fy as i32);
        let lerp = |a: u32, b: u32, w: u32| -> u32 {
            let rb = ((a & 0x00FF_00FF) * (256 - w) + (b & 0x00FF_00FF) * w) >> 8 & 0x00FF_00FF;
            let ag =
                ((a >> 8 & 0x00FF_00FF) * (256 - w) + (b >> 8 & 0x00FF_00FF) * w) & 0xFF00_FF00;
            rb | ag
        };
        let top = lerp(self.argb(x, y, clamp), self.argb(x + 1, y, clamp), wx);
        let bottom = lerp(
            self.argb(x, y + 1, clamp),
            self.argb(x + 1, y + 1, clamp),
            wx,
        );
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
    clamp: bool,
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

    /// Returns the pixels written.
    unsafe fn row(&self, y: usize) -> u64 {
        let [a, b, c] = self.points;
        let mut written = 0;
        let target = self.target;
        let py = y as f32;
        let (from, to) = self.span(py);
        for x in from..to {
            let px = x as f32;
            let mut weights = [0.0f32; 3];
            let mut inside = true;
            for (n, ((p, q), &bias)) in self.edges.iter().zip(&self.biases).enumerate() {
                let e = edge(p, q, px, py);
                if e < 0.0 || (e == 0.0 && !bias) {
                    inside = false;
                    break;
                }
                weights[n] = e * self.inverse_area;
            }
            if !inside {
                continue;
            }
            let [wa, wb, wc] = weights;
            let mix = |fa: f32, fb: f32, fc: f32| wa * fa + wb * fb + wc * fc;
            let z = mix(a.z, b.z, c.z);
            let place = y * target.width + x;
            let depth = unsafe { self.depth.add(place) };
            if self.depth_test && !compare(self.depth_function, z, unsafe { *depth }) {
                continue;
            }
            let colour: Colour = [0, 1, 2, 3].map(|n| mix(a.colour[n], b.colour[n], c.colour[n]));
            let mut out = colour;
            if let Some(texels) = &self.texture {
                let texture = texels.texture;
                let w = mix(a.w, b.w, c.w);
                let (u, v) = if w != 0.0 {
                    let over = 1.0 / w;
                    (
                        mix(self.uw[0], self.uw[1], self.uw[2]) * over,
                        mix(self.vw[0], self.vw[1], self.vw[2]) * over,
                    )
                } else {
                    (mix(a.u, b.u, c.u), mix(a.v, b.v, c.v))
                };
                let texel = if self.linear {
                    texels.bilinear(u, v, self.clamp)
                } else {
                    texels.nearest(u, v, self.clamp)
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
                let kept = mix(a.specular[3], b.specular[3], c.specular[3]).clamp(0.0, 1.0);
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
