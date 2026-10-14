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
//! - the texture (state 1 by `d3d::set_texture`), sampled at the nearest texel with
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
    fn texel(&self, u: f32, v: f32, clamp: bool) -> Colour {
        let at = |t: f32, size: usize| -> usize {
            let n = (t * size as f32).floor() as i64;
            if clamp {
                n.clamp(0, size as i64 - 1) as usize
            } else {
                n.rem_euclid(size as i64) as usize
            }
        };
        let (x, y) = (at(u, self.width), at(v, self.height));
        let place = y * self.pitch + x * self.bytes;
        let pixel = match self.bytes {
            2 => self
                .pixels
                .get(place..place + 2)
                .map_or(0, |b| u16::from_le_bytes([b[0], b[1]]) as u32),
            _ => self
                .pixels
                .get(place..place + 4)
                .map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        };
        [0, 1, 2, 3].map(|n| part(pixel, self.masks[n]))
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
    let to = |c: f32, most: f32| (c.clamp(0.0, 1.0) * most + 0.5) as u32;
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
    if vertices.len() < 3 || depth.len() < target.width * target.height {
        return;
    }
    let points: Vec<Point> = vertices.iter().map(point).collect();
    for n in 1..points.len() - 1 {
        unsafe {
            draw_triangle(
                [&points[0], &points[n], &points[n + 1]],
                target,
                states,
                texture,
                depth,
            )
        };
    }
}

unsafe fn draw_triangle(
    [a, b, c]: [&Point; 3],
    target: &Target,
    states: &States,
    texture: Option<&Texture>,
    depth: &mut [f32],
) {
    // Positive: clockwise on the screen (y goes down).
    let area = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
    if area == 0.0 || !area.is_finite() {
        return;
    }
    let cull = match states.get(0x16) {
        0 => 3,
        cull => cull,
    };
    if (cull == 3 && area < 0.0) || (cull == 2 && area > 0.0) {
        return;
    }
    // Wound the same way whatever the input, for the edges' signs.
    let (a, b, c, area) = if area < 0.0 {
        (a, c, b, -area)
    } else {
        (a, b, c, area)
    };
    let left = a.x.min(b.x).min(c.x).floor().max(0.0) as usize;
    let right = (a.x.max(b.x).max(c.x).ceil() as i64).clamp(0, target.width as i64) as usize;
    let top = a.y.min(b.y).min(c.y).floor().max(0.0) as usize;
    let bottom = (a.y.max(b.y).max(c.y).ceil() as i64).clamp(0, target.height as i64) as usize;

    let depth_test = states.get(0x07) != 0;
    let depth_function = states.get(0x17);
    let depth_write = states.get(0x0E) != 0;
    let alpha_test = states.get(0x0F) != 0;
    let alpha_function = states.get(0x19);
    let alpha_reference = (states.get(0x18) & 0xFF) as f32 / 255.0;
    let blending = states.get(0x1B) != 0;
    let (source_blend, target_blend) = (states.get(0x13), states.get(0x14));
    let fog = states.get(0x1C) != 0;
    let fog_colour = unpack(states.get(0x22));
    let mixing = states.get(0x15);
    let clamp = states.get(0x03) == 3;

    // Inside where all three edge functions are at or above 0, as the area is.
    let edges = [(b, c), (c, a), (a, b)];
    let biases: Vec<bool> = edges.iter().map(|(p, q)| top_left(p, q)).collect();
    for y in top..bottom {
        for x in left..right {
            let (px, py) = (x as f32, y as f32);
            let mut weights = [0.0f32; 3];
            let mut inside = true;
            for (n, ((p, q), &bias)) in edges.iter().zip(&biases).enumerate() {
                let e = edge(p, q, px, py);
                if e < 0.0 || (e == 0.0 && !bias) {
                    inside = false;
                    break;
                }
                weights[n] = e / area;
            }
            if !inside {
                continue;
            }
            let [wa, wb, wc] = weights;
            let mix = |fa: f32, fb: f32, fc: f32| wa * fa + wb * fb + wc * fc;
            let z = mix(a.z, b.z, c.z);
            let place = y * target.width + x;
            if depth_test && !compare(depth_function, z, depth[place]) {
                continue;
            }
            let colour: Colour = [0, 1, 2, 3].map(|n| mix(a.colour[n], b.colour[n], c.colour[n]));
            let mut out = colour;
            if let Some(texture) = texture {
                let w = mix(a.w, b.w, c.w);
                let (u, v) = if w != 0.0 {
                    (
                        mix(a.u * a.w, b.u * b.w, c.u * c.w) / w,
                        mix(a.v * a.w, b.v * b.w, c.v * c.w) / w,
                    )
                } else {
                    (mix(a.u, b.u, c.u), mix(a.v, b.v, c.v))
                };
                let texel = texture.texel(u, v, clamp);
                out = match mixing {
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
            if alpha_test && !compare(alpha_function, out[3], alpha_reference) {
                continue;
            }
            if fog {
                let kept = mix(a.specular[3], b.specular[3], c.specular[3]).clamp(0.0, 1.0);
                for n in 0..3 {
                    out[n] = out[n] * kept + fog_colour[n] * (1.0 - kept);
                }
            }
            if blending {
                let under = unsafe { read_target(target, x, y) };
                let (s, d) = (
                    factor(source_blend, &out, &under),
                    factor(target_blend, &out, &under),
                );
                out = [0, 1, 2, 3].map(|n| out[n] * s[n] + under[n] * d[n]);
            }
            unsafe { write_target(target, x, y, &out) };
            if depth_write {
                depth[place] = z;
            }
        }
    }
}
