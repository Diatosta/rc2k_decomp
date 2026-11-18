//! The commands as a GPU wants them: triangles in one list, and runs of them drawn alike. What a
//! state means is decided here, the same for every GPU backend; nothing here touches one.

use super::{Command, CommandList, STATES, TextureNote};

/// A vertex as the game makes it: x, y, z, 1/w, colour, specular, u, v.
pub type Vertex = crate::render::rasteriser::Vertex;

/// What makes up a pipeline. The comparisons and factors are Direct3D's numbers.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PipelineKey {
    /// Direct3D's culling: 1 none, 2 clockwise, 3 counter-clockwise on the screen.
    pub cull: u32,
    /// The depth test's comparison (always, when the test is off: writing goes on regardless,
    /// which a GPU's switched-off test would stop).
    pub depth_function: u32,
    pub depth_write: bool,
    /// The blend's source and target factors.
    pub blend: Option<(u32, u32)>,
    /// False for a depth clear: only the depth is written.
    pub colour: bool,
}

/// Which of a pipeline's states a key sets differently from the key before it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PipelineChange {
    pub cull: bool,
    pub depth_function: bool,
    pub depth_write: bool,
    pub blend: bool,
    pub colour: bool,
}

impl PipelineKey {
    /// The face a GL culling mode drops (`GL_FRONT` 0x0404 or `GL_BACK` 0x0405), `None` when
    /// culling is off.
    pub fn cull_face(&self) -> Option<u32> {
        match self.cull {
            3 => Some(0x0405),
            2 => Some(0x0404),
            _ => None,
        }
    }

    /// The GL depth comparison (`GL_NEVER` .. `GL_ALWAYS`) of the key's number.
    pub fn depth_func(&self) -> u32 {
        match self.depth_function {
            f @ 1..=7 => 0x0200 + f - 1,
            _ => 0x0207, // as the rasteriser's `compare`: anything else passes
        }
    }

    /// The states that must be set after `was` (all of them when nothing was set yet). A
    /// state whose GL value is the same stays out, even when the key's number differs.
    pub fn changes_from(&self, was: Option<&PipelineKey>) -> PipelineChange {
        match was {
            None => PipelineChange {
                cull: true,
                depth_function: true,
                depth_write: true,
                blend: true,
                colour: true,
            },
            Some(was) => PipelineChange {
                cull: self.cull_face() != was.cull_face(),
                depth_function: self.depth_func() != was.depth_func(),
                depth_write: self.depth_write != was.depth_write,
                blend: self.blend != was.blend,
                colour: self.colour != was.colour,
            },
        }
    }
}

/// The pixel shader's values (`scene.frag`'s `Pixel`).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PixelValues {
    pub fog_colour: [f32; 4],
    pub alpha_reference: f32,
    pub alpha_function: i32,
    pub mixing: i32,
    pub flags: i32,
}

impl PixelValues {
    /// The values' dwords as the shader's uniform block holds them.
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    pub fn to_words(self) -> [u32; 8] {
        let [red, green, blue, alpha] = self.fog_colour;
        [
            red.to_bits(),
            green.to_bits(),
            blue.to_bits(),
            alpha.to_bits(),
            self.alpha_reference.to_bits(),
            self.alpha_function as u32,
            self.mixing as u32,
            self.flags as u32,
        ]
    }
}

pub const TEXTURED: i32 = 1;
pub const TEXTURE_HAS_ALPHA: i32 = 2;
pub const FOG: i32 = 4;
/// The batch draws a second pass over its first in the same draw (`Batch::second`).
pub const SECOND: i32 = 8;

/// `source` with `defines` (whole `#define` lines) after its `#version` line, which must stay
/// the first.
pub fn with_defines(source: &str, defines: &str) -> String {
    match source.split_once('\n') {
        Some((version, rest)) => format!("{version}\n{defines}{rest}"),
        None => source.to_string(),
    }
}

/// What of one pass's values a pixel shader can be made for, in place of reading them for
/// every pixel: values that draw the same pixels have the same kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PassKind {
    /// The alpha test's comparison (1 ..= 7); 0 when the test discards nothing.
    pub alpha_function: u8,
    /// How the texel and the colour make the pixel: 1 the texel alone, 4 their product, 0
    /// their colours' product with the alpha of one of them. 0 without a texture.
    pub mixing: u8,
    pub textured: bool,
    /// The alpha is the texel's (only where `mixing` is 0).
    pub texel_alpha: bool,
    pub fog: bool,
}

impl PassKind {
    /// The kind of `values`; `discard_free`: the pass's alpha test is known to keep every pixel.
    pub fn of(values: &PixelValues, discard_free: bool) -> PassKind {
        let textured = values.flags & TEXTURED != 0;
        let mixing = match values.mixing {
            _ if !textured => 0,
            1 | 7 => 1,
            4 => 4,
            _ => 0,
        };
        PassKind {
            alpha_function: match values.alpha_function {
                function @ 1..=7 if !discard_free => function as u8,
                _ => 0,
            },
            mixing,
            textured,
            texel_alpha: textured && mixing == 0 && values.flags & TEXTURE_HAS_ALPHA != 0,
            fog: values.flags & FOG != 0,
        }
    }

    /// The kind as the shader's `flags` holds it.
    fn flags(&self) -> i32 {
        let bit = |on: bool, flag: i32| if on { flag } else { 0 };
        bit(self.textured, TEXTURED) | bit(self.texel_alpha, TEXTURE_HAS_ALPHA) | bit(self.fog, FOG)
    }
}

/// What of a batch the scene's pixel shader can be made for: its pass's kind and, with a
/// second pass, that one's.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ShaderKind {
    pub first: PassKind,
    pub second: Option<PassKind>,
}

impl ShaderKind {
    pub fn of(batch: &Batch) -> ShaderKind {
        ShaderKind {
            first: PassKind::of(&batch.values, batch.discard_free),
            second: batch
                .second
                .map(|second| PassKind::of(&second.values, false)),
        }
    }

    /// The `#define` lines that make the scene's shaders this kind's: the passes' numbers as
    /// constants, and which of the vertex's values a pass reads at all.
    pub fn defines(&self) -> String {
        let mut lines = format!(
            "#define MADE_FOR_KIND\n#define ALPHA_FUNCTION {}\n#define MIXING {}\n#define FLAGS {}\n",
            self.first.alpha_function,
            self.first.mixing,
            self.first.flags() | if self.second.is_some() { SECOND } else { 0 },
        );
        let mut define = |on: bool, name: &str| {
            if on {
                lines += &format!("#define {name}\n");
            }
        };
        define(self.first.alpha_function == 0, "NO_DISCARD");
        define(self.first.textured, "FIRST_TEXTURE");
        define(self.first.fog, "FIRST_FOG");
        if let Some(second) = self.second {
            define(true, "SECOND");
            define(second.textured, "SECOND_TEXTURE");
            define(second.fog, "SECOND_FOG");
            lines += &format!(
                "#define ALPHA_FUNCTION2 {}\n#define MIXING2 {}\n#define FLAGS2 {}\n",
                second.alpha_function,
                second.mixing,
                second.flags(),
            );
        }
        lines
    }

    /// The `#define` lines of the shaders made for every kind: each pass's numbers read for
    /// every pixel, every value of the vertex passed on. `discard_free`: with no alpha test.
    pub fn defines_for_any(discard_free: bool) -> String {
        let mut lines = String::from(
            "#define FIRST_TEXTURE\n#define FIRST_FOG\n#define SECOND\n#define SECOND_TEXTURE\n#define SECOND_FOG\n",
        );
        if discard_free {
            lines += "#define NO_DISCARD\n";
        }
        lines
    }
}

/// A second pass drawn over a batch's own in the same draw: the face drawn again at once
/// without depth writes, blended by its alpha (the ground's second texture). Its vertices' colour,
/// specular and texture place are in `Batches::seconds`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Second {
    pub values: PixelValues,
    pub texture: Option<u32>,
    pub linear: bool,
    pub clamp: bool,
    /// Read from the texture's smaller levels as the face shrinks (the game's mip filter).
    pub mips: bool,
}

/// A run of triangles drawn alike.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Batch {
    pub first_vertex: u32,
    pub vertices: u32,
    pub pipeline: PipelineKey,
    pub values: PixelValues,
    /// Which of `Batches::textures`.
    pub texture: Option<u32>,
    pub linear: bool,
    pub clamp: bool,
    /// Read from the texture's smaller levels as the face shrinks (the game's mip filter).
    pub mips: bool,
    /// Scissor rectangle for a depth clear: left, top, right, bottom.
    pub scissor: Option<[u32; 4]>,
    pub second: Option<Second>,
    /// The fragment shader may leave out its alpha test: it cannot discard a pixel of this
    /// batch (`fan_fast_path`). A GPU that draws such a batch with a shader with no `discard`
    /// can reject hidden pixels before running it. Only set by `of_layered` (the handheld's).
    pub discard_free: bool,
}

#[derive(Default)]
pub struct Batches {
    pub vertices: Vec<Vertex>,
    pub batches: Vec<Batch>,
    pub textures: Vec<TextureNote>,
    /// With second passes joined (`of`), each vertex's second pass's colour, specular and
    /// texture place (vertices of batches without one: zeros).
    pub seconds: Vec<[u32; 4]>,
    /// Layered (`of_layered`): each vertex's first and second pass's texture, by number in
    /// `textures` (`NO_TEXTURE` for none).
    pub vertex_textures: Vec<[u16; 2]>,
    /// Layered: whether two textures can be read in one draw.
    layered: Option<fn(&TextureNote, &TextureNote) -> bool>,
    /// Fans are given their `fan_fast_path` (`of_layered` with `fast_path`).
    fast_path: bool,
    /// The target's width and height (`apart`).
    size: (f64, f64),
    /// The pixels each batch can touch, while they are being gathered.
    areas: Vec<Area>,
    /// The last fan, while it can still take a second pass.
    last_fan: Option<LastFan>,
    /// Each of `textures`' number, by its id; the last one numbered (`texture_number`).
    texture_numbers: super::TextureMap<u32>,
    last_texture: Option<(super::TextureId, u32)>,
}

/// A pass without a texture, in `Batches::vertex_textures`.
pub const NO_TEXTURE: u16 = u16::MAX;

impl Batches {
    /// Everything taken out, the vectors' room kept.
    fn clear(&mut self) {
        let Batches {
            vertices,
            batches,
            textures,
            seconds,
            vertex_textures,
            layered,
            fast_path,
            size,
            areas,
            last_fan,
            texture_numbers,
            last_texture,
        } = self;
        vertices.clear();
        batches.clear();
        textures.clear();
        seconds.clear();
        vertex_textures.clear();
        *layered = None;
        *fast_path = false;
        *size = (0.0, 0.0);
        areas.clear();
        *last_fan = None;
        texture_numbers.clear();
        *last_texture = None;
    }
}

/// Batches drawn and handed back (`give_back_batches`), their room kept for the next flushes:
/// the vectors grown to a frame's fans are most of the batching's time.
static SPARE_BATCHES: std::sync::Mutex<Vec<Batches>> = std::sync::Mutex::new(Vec::new());

/// Batches kept at most: the ones in flight between the batching thread and the GL's, and the
/// one being drawn.
const SPARE_BATCHES_KEPT: usize = 4;

/// Empty batches: drawn ones handed back, else new.
fn spare_batches() -> Batches {
    SPARE_BATCHES.lock().unwrap().pop().unwrap_or_default()
}

/// Batches drawn: emptied and kept for the next flushes.
pub fn give_back_batches(mut batches: Batches) {
    batches.clear();
    let mut spare = SPARE_BATCHES.lock().unwrap();
    if spare.len() < SPARE_BATCHES_KEPT {
        spare.push(batches);
    }
}

/// A fan that can take a second pass.
struct LastFan {
    /// Its triangles' first vertex and count: the last batch's tail.
    first_vertex: u32,
    vertices: u32,
    /// Its corners' places (x, y, z, 1/w).
    places: smallvec::SmallVec<[[u32; 4]; 6]>,
    batch: Batch,
    area: Area,
}

/// Where a batch can touch the target, in the game's coordinates: a rectangle around its
/// triangles, both ends in. It is kept as the continuous extent, not rounded to whole game
/// pixels: a target finer than the game's grid (a window larger than the game's screen) has
/// pixel centres between the game's whole places, so two triangles with no whole place in
/// common can still share target pixels.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Area {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Area {
    const EVERYWHERE: Area = Area {
        left: f32::NEG_INFINITY,
        top: f32::NEG_INFINITY,
        right: f32::INFINITY,
        bottom: f32::INFINITY,
    };

    /// Around `vertices`: a triangle touches the target's pixels whose centres it holds, so
    /// nothing outside its least and most x and y (the margin on each side covers the GPU's
    /// rounding of the corners to its sub-pixel grid, a sixteenth of a pixel at the coarsest
    /// Vulkan allows, in pixels of a target smaller than the game's screen too).
    fn around(vertices: &[Vertex]) -> Area {
        const MARGIN: f32 = 0.25;
        let (mut left, mut top) = (f32::INFINITY, f32::INFINITY);
        let (mut right, mut bottom) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        for v in vertices {
            let (x, y) = (f32::from_bits(v[0]), f32::from_bits(v[1]));
            if !x.is_finite() || !y.is_finite() {
                return Area::EVERYWHERE;
            }
            (left, right) = (left.min(x), right.max(x));
            (top, bottom) = (top.min(y), bottom.max(y));
        }
        Area {
            left: left - MARGIN,
            top: top - MARGIN,
            right: right + MARGIN,
            bottom: bottom + MARGIN,
        }
    }

    fn with(self, other: Area) -> Area {
        Area {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    fn meets(self, other: Area) -> bool {
        self.left <= other.right
            && other.left <= self.right
            && self.top <= other.bottom
            && other.top <= self.bottom
    }
}

/// A triangle's corners on the target as the GPU places them (`scene.vert`: x and y as given,
/// times the clip space's w, which the GPU divides out again; w is 1 over the fourth dword when
/// that is above 0, else 1), when that is a plain place: x, y and w finite, w small enough that
/// the clip space's place is finite too.
fn corners(t: &[Vertex; 3]) -> Option<[(f64, f64); 3]> {
    let mut out = [(0.0, 0.0); 3];
    for (corner, v) in out.iter_mut().zip(t) {
        let (x, y, w) = (
            f32::from_bits(v[0]),
            f32::from_bits(v[1]),
            f32::from_bits(v[3]),
        );
        if !x.is_finite() || !y.is_finite() || !w.is_finite() || (w > 0.0 && w < 1e-30) {
            return None;
        }
        *corner = (f64::from(x), f64::from(y));
    }
    Some(out)
}

/// Whether triangles `t` and `u` are drawn on no pixel in common, `size` the target's.
///
/// Either a line through one's edge has the other wholly on its far side, by more than the GPU's
/// rounding of the corners can bring it back: `Area::around`'s margin, widened for a corner far
/// past the edge's ends, which a tilt of the line moves the most. Or the two have an edge with
/// the same corners, bit for bit in all the GPU's place reads (x, y, w), inside the target (no
/// clipping makes new corners on it), and the other's third corner is on its far side by more
/// than the margin: the GPU places those corners alike, so the edge stays one line between them,
/// and the pixels on it go to one triangle only (the rasterising rule for a shared edge). A
/// triangle with no area, or a place that is no plain place (`corners`), may share pixels.
fn apart(t: &[Vertex; 3], u: &[Vertex; 3], size: (f64, f64)) -> bool {
    const MARGIN: f64 = 0.125;
    let (Some(tp), Some(up)) = (corners(t), corners(u)) else {
        return false;
    };
    let same = |a: &Vertex, b: &Vertex| a[0] == b[0] && a[1] == b[1] && a[3] == b[3];
    // Inside the clip space (the shaders put x at x + 0.5 of the target's width): no clipping.
    let inside =
        |p: (f64, f64)| (0.0..=size.0 - 1.0).contains(&p.0) && (0.0..=size.1 - 1.0).contains(&p.1);
    // An edge of `own` (its corners `n` and `n + 1`), with its third corner, against `other`.
    let cuts = |own: &[(f64, f64); 3],
                own_v: &[Vertex; 3],
                other: &[(f64, f64); 3],
                other_v: &[Vertex; 3]| {
        (0..3).any(|n| {
            let (p, q, r) = (own[n], own[(n + 1) % 3], own[(n + 2) % 3]);
            let (dx, dy) = (q.0 - p.0, q.1 - p.1);
            let length = (dx * dx + dy * dy).sqrt();
            if length == 0.0 {
                return false;
            }
            // The distance from the edge's line, positive on the third corner's side.
            let side = |s: (f64, f64)| (dx * (s.1 - p.1) - dy * (s.0 - p.0)) / length;
            let own_side = side(r);
            if own_side.abs() <= MARGIN {
                return false;
            }
            let beyond = |s: (f64, f64)| {
                let distance = |a: (f64, f64)| ((s.0 - a.0).powi(2) + (s.1 - a.1).powi(2)).sqrt();
                let reach = distance(p).max(distance(q));
                side(s) * own_side.signum() < -MARGIN * (1.0 + 2.0 * reach / length)
            };
            let (pv, qv) = (&own_v[n], &own_v[(n + 1) % 3]);
            let shared = (0..3).find(|&m| {
                let (a, b) = (&other_v[m], &other_v[(m + 1) % 3]);
                (same(a, pv) && same(b, qv)) || (same(a, qv) && same(b, pv))
            });
            match shared {
                Some(m) if inside(p) && inside(q) => beyond(other[(m + 2) % 3]),
                _ => other.iter().all(|&s| beyond(s)),
            }
        })
    };
    cuts(&tp, t, &up, u) || cuts(&up, u, &tp, t)
}

/// Whether two batches are drawn alike, wherever their triangles lie.
fn alike(a: &Batch, b: &Batch) -> bool {
    Batch {
        first_vertex: 0,
        vertices: 0,
        ..*a
    } == Batch {
        first_vertex: 0,
        vertices: 0,
        ..*b
    }
}

const ALWAYS: u32 = 8;

// As the rasteriser reads them.
fn linear_of(states: &[u32; STATES]) -> bool {
    states[0x11] == 2 || matches!(states[0x12], 2 | 5 | 6)
}

/// The minifying filter is the mipmapping one (the software rasteriser's `mip_chains` asks alike).
fn mips_of(states: &[u32; STATES]) -> bool {
    states[0x12] == 6
}

fn clamp_of(states: &[u32; STATES]) -> bool {
    states[0x03] == 3
}

/// Whether the pixel shader's alpha test (`passes`) keeps a pixel of alpha `value`.
fn alpha_passes(values: &PixelValues, value: f32) -> bool {
    let reference = values.alpha_reference;
    match values.alpha_function {
        0 => true,
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

/// What a fan's draw can leave out without changing a pixel (`fan_fast_path`).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct FastPath {
    /// The alpha test cannot discard a pixel of the fan.
    pub discard_free: bool,
    /// The blend leaves each pixel as the shader made it, so it can be switched off.
    pub blend_off: bool,
}

/// Whether the alpha test keeps every pixel of a fan: always when the test is off or its
/// comparison is not one the shader knows, and, when `alpha_full` (every pixel's alpha is 1,
/// up to the few millionths the interpolation may be off), for the comparisons that pass
/// there whatever the reference is up to 254/255.
fn alpha_test_keeps_all(values: &PixelValues, alpha_full: bool) -> bool {
    match values.alpha_function {
        0 | 8.. => true,
        5..=7 => alpha_full && values.alpha_reference * 255.0 <= 254.5,
        _ => false,
    }
}

/// Which work a fan's draw can leave out, the picture the same. `texture_clear`: the fan has
/// no texture, or a texture with no pixel under a full alpha. With that and every corner's
/// colour at a full alpha, each pixel's alpha is 1 (the fog and the alpha test change none).
/// - Blend factors source alpha / inverse source alpha keep the source's colour where its alpha
///   is 1 (`1 * s + 0 * d`), and one / zero do for any alpha: the blend can be off.
/// - An alpha test that passes (or is off) cannot discard.
///
/// A fan that takes a second pass later (`join_second`) keeps what it has here: that pass is
/// blended in the shader and discards nothing.
pub fn fan_fast_path(states: &[u32; STATES], vertices: &[Vertex], texture_clear: bool) -> FastPath {
    let alpha_full = texture_clear && vertices.iter().all(|v| v[4] >> 24 == 0xFF);
    let values = values_of(states, None);
    FastPath {
        discard_free: alpha_test_keeps_all(&values, alpha_full),
        blend_off: match pipeline_of(states).blend {
            Some((2, 1)) => true,
            Some((5, 6)) => alpha_full,
            _ => false,
        },
    }
}

fn pipeline_of(states: &[u32; STATES]) -> PipelineKey {
    PipelineKey {
        cull: match states[0x16] {
            0 => 3, // Direct3D's default
            cull => cull,
        },
        depth_function: if states[0x07] != 0 {
            states[0x17]
        } else {
            ALWAYS
        },
        depth_write: states[0x0E] != 0,
        blend: (states[0x1B] != 0).then_some((states[0x13], states[0x14])),
        colour: true,
    }
}

fn values_of(states: &[u32; STATES], texture: Option<&TextureNote>) -> PixelValues {
    let fog = states[0x22];
    let unit = |shift: u32| (fog >> shift & 0xFF) as f32 / 255.0;
    let mut flags = 0;
    if let Some(t) = texture {
        flags |= TEXTURED;
        if t.masks[3] != 0 {
            flags |= TEXTURE_HAS_ALPHA;
        }
    }
    if states[0x1C] != 0 {
        flags |= FOG;
    }
    PixelValues {
        fog_colour: [unit(16), unit(8), unit(0), unit(24)],
        alpha_reference: (states[0x18] & 0xFF) as f32 / 255.0,
        alpha_function: if states[0x0F] != 0 {
            states[0x19] as i32
        } else {
            0
        },
        mixing: states[0x15] as i32,
        flags,
    }
}

impl Batches {
    /// `commands` in the order they came. With `opaque` (whether a texture's every pixel has a
    /// full alpha), a fan drawn again at once over the same places without depth writes and
    /// blended joins the fan before as its second pass (`Batch::second`): one draw for both,
    /// the same picture but for the blend's rounding done in the shader (a step at most).
    pub fn of(
        commands: &CommandList,
        width: u32,
        height: u32,
        opaque: Option<&dyn Fn(&TextureNote) -> bool>,
    ) -> Batches {
        Batches::of_in(Batches::default(), commands, width, height, opaque)
    }

    /// `of`, for a GPU that holds its textures as layers of a few texture arrays and reads
    /// which one each vertex's passes take from `vertex_textures`: batches that differ only in
    /// their textures are drawn as one when `same_array` says each pass's textures can be
    /// read in one draw. The order and the gathering's rule are `of`'s, so the picture is.
    /// With `fast_path`, each fan's batch is told what it can leave out (`fan_fast_path`).
    pub fn of_layered(
        commands: &CommandList,
        width: u32,
        height: u32,
        opaque: &dyn Fn(&TextureNote) -> bool,
        same_array: fn(&TextureNote, &TextureNote) -> bool,
        fast_path: bool,
    ) -> Batches {
        let start = Batches {
            layered: Some(same_array),
            fast_path,
            ..spare_batches()
        };
        Batches::of_in(start, commands, width, height, Some(opaque))
    }

    fn of_in(
        mut out: Batches,
        noted: &CommandList,
        width: u32,
        height: u32,
        opaque: Option<&dyn Fn(&TextureNote) -> bool>,
    ) -> Batches {
        out.size = (f64::from(width), f64::from(height));
        for command in &noted.commands {
            match command {
                Command::Fan {
                    vertices,
                    states,
                    texture,
                    ..
                } => {
                    let vertices = noted.fan_vertices(*vertices);
                    if vertices.len() < 3 {
                        continue;
                    }
                    let Some(opaque) = opaque else {
                        out.push_fan(vertices, states, texture.as_ref(), None);
                        continue;
                    };
                    if out.join_second(vertices, states, texture.as_ref()) {
                        continue;
                    }
                    out.push_fan(vertices, states, texture.as_ref(), Some(opaque));
                    out.seconds.resize(out.vertices.len(), [0; 4]);
                    out.note_first(vertices, texture.as_ref(), opaque);
                }
                Command::Clear(rect) => {
                    out.last_fan = None;
                    out.push_clear(*rect, width, height);
                    if opaque.is_some() {
                        out.seconds.resize(out.vertices.len(), [0; 4]);
                    }
                }
            }
        }
        out.gather();
        out
    }

    /// Whether two batches can be drawn as one: alike, or (layered) alike but for textures
    /// read from the same arrays.
    fn joinable(&self, a: &Batch, b: &Batch) -> bool {
        let Some(same_array) = self.layered else {
            return alike(a, b);
        };
        let fit = |x: Option<u32>, y: Option<u32>| match (x, y) {
            (None, None) => true,
            (Some(x), Some(y)) => {
                x == y || same_array(&self.textures[x as usize], &self.textures[y as usize])
            }
            _ => false,
        };
        let seconds_fit = match (&a.second, &b.second) {
            (Some(x), Some(y)) => fit(x.texture, y.texture),
            (None, None) => true,
            _ => false,
        };
        // Only whether each pass is textured stays in the comparison.
        let plain = |x: &Batch| Batch {
            first_vertex: 0,
            vertices: 0,
            texture: x.texture.map(|_| 0),
            second: x.second.map(|s| Second {
                texture: s.texture.map(|_| 0),
                ..s
            }),
            ..*x
        };
        fit(a.texture, b.texture) && seconds_fit && plain(a) == plain(b)
    }

    /// Each batch moved back into the last run drawn like it, when no batch between them can
    /// touch its pixels. Batches that touch no pixel in common give the same picture in either
    /// order, whatever their depth tests and blending, so the picture is the one the order the
    /// commands came in gives; a stage's frame then takes about half the draws (the ground's two
    /// passes per face), the handheld's driver's cost.
    fn gather(&mut self) {
        /// How many runs back a batch looks for its own (32 found nearly all of a frame's).
        const LOOK_BACK: usize = 32;
        struct Run {
            batch: Batch,
            members: smallvec::SmallVec<[usize; 4]>,
            /// All the members' areas.
            area: Area,
        }
        // Layered, each triangle's area (`may_share_pixels`).
        let triangle_areas: Vec<Area> = if self.layered.is_some() {
            let (triangles, _) = self.vertices.as_chunks::<3>();
            triangles.iter().map(|t| Area::around(t)).collect()
        } else {
            Vec::new()
        };
        let mut runs: Vec<Run> = Vec::with_capacity(self.batches.len());
        for (n, (batch, &area)) in self.batches.iter().zip(&self.areas).enumerate() {
            let mut joined = None;
            for (r, run) in runs.iter().enumerate().rev().take(LOOK_BACK) {
                if self.joinable(&run.batch, batch) {
                    joined = Some(r);
                    break;
                }
                if run.area.meets(area)
                    && run.members.iter().any(|&m| {
                        self.areas[m].meets(area) && self.may_share_pixels(m, n, &triangle_areas)
                    })
                {
                    break;
                }
            }
            match joined {
                Some(r) => {
                    runs[r].members.push(n);
                    runs[r].area = runs[r].area.with(area);
                }
                None => runs.push(Run {
                    batch: *batch,
                    members: smallvec::smallvec![n],
                    area,
                }),
            }
        }
        self.areas.clear();
        if runs.len() == self.batches.len() {
            return;
        }
        let mut vertices = Vec::with_capacity(self.vertices.len());
        let mut seconds = Vec::with_capacity(self.seconds.len());
        let mut vertex_textures = Vec::with_capacity(self.vertex_textures.len());
        let mut batches = Vec::with_capacity(runs.len());
        for run in runs {
            let first_vertex = vertices.len() as u32;
            for m in run.members {
                let b = &self.batches[m];
                let range = b.first_vertex as usize..(b.first_vertex + b.vertices) as usize;
                vertices.extend_from_slice(&self.vertices[range.clone()]);
                if !self.seconds.is_empty() {
                    seconds.extend_from_slice(&self.seconds[range.clone()]);
                }
                if !self.vertex_textures.is_empty() {
                    vertex_textures.extend_from_slice(&self.vertex_textures[range]);
                }
            }
            batches.push(Batch {
                first_vertex,
                vertices: vertices.len() as u32 - first_vertex,
                ..run.batch
            });
        }
        self.vertices = vertices;
        self.seconds = seconds;
        self.vertex_textures = vertex_textures;
        self.batches = batches;
    }

    /// Whether batches `a` and `b` (numbers in `batches`, before gathering), whose areas meet,
    /// may touch a pixel in common. Layered, triangles are looked at pair by pair (`apart`), each
    /// with its area (`triangle_areas`, by its first vertex's number over 3): a car's faces, each
    /// drawn twice (its texture, then its shine added), have areas that meet their neighbours'
    /// along every edge they share, though no pixel is drawn by both. A depth clear (a scissor
    /// over a triangle bigger than the target) touches everything its area meets.
    fn may_share_pixels(&self, a: usize, b: usize, triangle_areas: &[Area]) -> bool {
        if self.layered.is_none() {
            return true;
        }
        let (area_a, area_b) = (self.areas[a], self.areas[b]);
        let (a, b) = (&self.batches[a], &self.batches[b]);
        if a.scissor.is_some() || b.scissor.is_some() {
            return true;
        }
        // Each batch's triangles that reach into the other's area, by number.
        let near = |x: &Batch, other: Area| {
            let first = x.first_vertex as usize / 3;
            (first..first + x.vertices as usize / 3)
                .filter(move |&n| triangle_areas[n].meets(other))
        };
        let theirs: smallvec::SmallVec<[usize; 16]> = near(b, area_a).collect();
        let triangle = |n: usize| {
            let (chunks, _) = self.vertices[3 * n..3 * n + 3].as_chunks::<3>();
            &chunks[0]
        };
        near(a, area_b).any(|t| {
            theirs.iter().any(|&u| {
                triangle_areas[t].meets(triangle_areas[u])
                    && !apart(triangle(t), triangle(u), self.size)
            })
        })
    }

    /// Notes the fan just pushed as one that can take a second pass, when its own pass leaves
    /// exactly its colour: depth written, an opaque texture, every corner's alpha full, its
    /// alpha test passed at a full alpha, and a blend (if any) that keeps a full alpha's source.
    fn note_first(
        &mut self,
        vertices: &[Vertex],
        texture: Option<&TextureNote>,
        opaque: &dyn Fn(&TextureNote) -> bool,
    ) {
        self.last_fan = None;
        let Some(batch) = self.batches.last() else {
            return;
        };
        let count = (vertices.len() as u32 - 2) * 3;
        let keeps_source = matches!(batch.pipeline.blend, None | Some((5, 6)) | Some((2, 1)));
        if batch.pipeline.depth_write
            && batch.pipeline.colour
            && keeps_source
            && batch.scissor.is_none()
            && texture.is_some_and(opaque)
            && vertices.iter().all(|v| v[4] >> 24 == 0xFF)
            && alpha_passes(&batch.values, 1.0)
        {
            self.last_fan = Some(LastFan {
                first_vertex: self.vertices.len() as u32 - count,
                vertices: count,
                places: vertices.iter().map(|v| [v[0], v[1], v[2], v[3]]).collect(),
                batch: Batch {
                    first_vertex: 0,
                    vertices: 0,
                    ..*batch
                },
                area: Area::around(vertices),
            });
        }
    }

    /// Joins the fan as the last fan's second pass, when it is one: the same places, depth
    /// tested alike without writing, blended by its alpha. False when it is not.
    fn join_second(
        &mut self,
        vertices: &[Vertex],
        states: &[u32; STATES],
        texture: Option<&TextureNote>,
    ) -> bool {
        let Some(first) = self.last_fan.take() else {
            return false;
        };
        let pipeline = pipeline_of(states);
        let same_places = first.places.len() == vertices.len()
            && first
                .places
                .iter()
                .zip(vertices)
                .all(|(p, v)| p[..] == v[..4]);
        if !same_places
            || pipeline.depth_write
            || pipeline.blend != Some((5, 6))
            || pipeline.depth_function != first.batch.pipeline.depth_function
            || pipeline.cull != first.batch.pipeline.cull
        {
            return false;
        }
        let second = Second {
            values: values_of(states, texture),
            texture: texture.map(|t| self.texture_number(t)),
            linear: linear_of(states),
            clamp: clamp_of(states),
            mips: mips_of(states),
        };
        // The first fan's triangles leave the last batch, which they end, for a batch of
        // their own with the second pass.
        if let Some(last) = self.batches.last_mut() {
            last.vertices -= first.vertices;
            if last.vertices == 0 {
                self.batches.pop();
                self.areas.pop();
            }
        }
        let start = first.first_vertex as usize;
        for (n, corner) in (1..vertices.len() - 1)
            .flat_map(|n| [vertices[0], vertices[n], vertices[n + 1]])
            .enumerate()
        {
            self.seconds[start + n] = [corner[4], corner[5], corner[6], corner[7]];
        }
        if self.layered.is_some() {
            let number = second.texture.map_or(NO_TEXTURE, |n| n as u16);
            for v in &mut self.vertex_textures[start..start + first.vertices as usize] {
                v[1] = number;
            }
        }
        // Layered, `first.batch` (the batch the fan had joined) may name another texture of
        // the same arrays: the vertices' own numbers say which each reads.
        let mut values = first.batch.values;
        values.flags |= SECOND;
        self.push(
            Batch {
                first_vertex: first.first_vertex,
                vertices: first.vertices,
                pipeline: PipelineKey {
                    blend: None,
                    ..first.batch.pipeline
                },
                values,
                second: Some(second),
                ..first.batch
            },
            first.area,
        );
        true
    }

    /// `texture`'s number in `textures`, which it joins when it is new: looked up by its id,
    /// the last fan's first (most fans run on in the last one's texture).
    fn texture_number(&mut self, texture: &TextureNote) -> u32 {
        if let Some((pixels, number)) = self.last_texture
            && pixels == texture.pixels
        {
            return number;
        }
        let next = self.textures.len() as u32;
        let number = *self.texture_numbers.entry(texture.pixels).or_insert(next);
        if number == next {
            self.textures.push(*texture);
        }
        self.last_texture = Some((texture.pixels, number));
        number
    }

    fn push_fan(
        &mut self,
        vertices: &[Vertex],
        states: &[u32; STATES],
        texture: Option<&TextureNote>,
        opaque: Option<&dyn Fn(&TextureNote) -> bool>,
    ) {
        if vertices.len() < 3 {
            return;
        }
        let first_vertex = self.vertices.len() as u32;
        let area = Area::around(vertices);
        for n in 1..vertices.len() - 1 {
            self.vertices
                .extend([vertices[0], vertices[n], vertices[n + 1]]);
        }
        let number = texture.map(|t| self.texture_number(t));
        if self.layered.is_some() {
            let layer = number.map_or(NO_TEXTURE, |n| n as u16);
            self.vertex_textures
                .resize(self.vertices.len(), [layer, NO_TEXTURE]);
        }
        let mut pipeline = pipeline_of(states);
        let mut discard_free = false;
        if self.fast_path {
            // A texture whose opacity is not known counts as one with alpha.
            let clear = texture.is_none_or(|t| opaque.is_some_and(|opaque| opaque(t)));
            let fast = fan_fast_path(states, vertices, clear);
            discard_free = fast.discard_free;
            if fast.blend_off {
                pipeline.blend = None;
            }
        }
        let batch = Batch {
            first_vertex,
            vertices: (vertices.len() as u32 - 2) * 3,
            pipeline,
            values: values_of(states, texture),
            texture: number,
            linear: linear_of(states),
            clamp: clamp_of(states),
            mips: mips_of(states),
            scissor: None,
            second: None,
            discard_free,
        };
        self.push(batch, area);
    }

    fn push_clear(&mut self, [left, top, right, bottom]: [usize; 4], width: u32, height: u32) {
        let (right, bottom) = ((right as u32).min(width), (bottom as u32).min(height));
        let (left, top) = (left as u32, top as u32);
        if left >= right || top >= bottom {
            return;
        }
        let first_vertex = self.vertices.len() as u32;
        // A triangle over the whole target, at the farthest depth, cut by the scissor.
        let corner = |x: f32, y: f32| -> Vertex {
            [
                x.to_bits(),
                y.to_bits(),
                1f32.to_bits(),
                1f32.to_bits(),
                0,
                0,
                0,
                0,
            ]
        };
        let (w, h) = (width as f32, height as f32);
        // Wound clockwise on the screen, though nothing culls it.
        self.vertices.extend([
            corner(-1.0, -1.0),
            corner(2.0 * w, -1.0),
            corner(-1.0, 2.0 * h),
        ]);
        if self.layered.is_some() {
            self.vertex_textures
                .resize(self.vertices.len(), [NO_TEXTURE; 2]);
        }
        self.push(
            Batch {
                first_vertex,
                vertices: 3,
                pipeline: PipelineKey {
                    cull: 1,
                    depth_function: ALWAYS,
                    depth_write: true,
                    blend: None,
                    colour: false,
                },
                values: PixelValues {
                    fog_colour: [0.0; 4],
                    alpha_reference: 0.0,
                    alpha_function: 0,
                    mixing: 0,
                    flags: 0,
                },
                texture: None,
                linear: false,
                clamp: false,
                mips: false,
                scissor: Some([left, top, right, bottom]),
                second: None,
                // Nothing a shader decides is drawn; the program in use stays.
                discard_free: false,
            },
            // The triangle is the whole target's and more; the scissor is not counted on.
            Area::EVERYWHERE,
        );
    }

    /// `batch`, touching `area`, joined to the last one when they are drawn alike.
    fn push(&mut self, batch: Batch, area: Area) {
        let joins = self.batches.last().is_some_and(|last| {
            last.first_vertex + last.vertices == batch.first_vertex && self.joinable(last, &batch)
        });
        if let (true, Some(last), Some(last_area)) =
            (joins, self.batches.last_mut(), self.areas.last_mut())
        {
            last.vertices += batch.vertices;
            *last_area = last_area.with(area);
            return;
        }
        self.batches.push(batch);
        self.areas.push(area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::render::{TestCommand, list};

    fn fan(corners: usize, states: [u32; STATES]) -> TestCommand {
        TestCommand::Fan {
            vertices: vec![[0; 8]; corners],
            states: std::sync::Arc::new(states),
            texture: None,
            bounds: (0.0, 0.0, 0.0, 0.0),
        }
    }

    #[test]
    fn fans_become_triangles_and_alike_runs_join() {
        let plain = [0; STATES];
        let mut blended = plain;
        blended[0x1B] = 1;
        let b = Batches::of(
            &list([fan(4, plain), fan(3, plain), fan(3, blended)]),
            640,
            480,
            None,
        );
        assert_eq!(b.vertices.len(), 12);
        assert_eq!(b.batches.len(), 2);
        assert_eq!(b.batches[0].vertices, 9);
        assert_eq!(b.batches[1].first_vertex, 9);
    }

    #[test]
    fn a_switched_off_depth_test_still_writes() {
        let mut states = [0; STATES];
        states[0x0E] = 1;
        states[0x17] = 2;
        let p = pipeline_of(&states);
        assert_eq!((p.depth_function, p.depth_write), (ALWAYS, true));
        states[0x07] = 1;
        assert_eq!(pipeline_of(&states).depth_function, 2);
    }

    #[test]
    fn a_clear_is_cut_to_the_target_and_only_touches_depth() {
        let b = Batches::of(
            &list([TestCommand::Clear([10, 20, 9999, 30])]),
            640,
            480,
            None,
        );
        assert_eq!(b.batches[0].scissor, Some([10, 20, 640, 30]));
        assert!(!b.batches[0].pipeline.colour);
        assert!(
            Batches::of(&list([TestCommand::Clear([5, 5, 5, 9])]), 640, 480, None)
                .batches
                .is_empty()
        );
    }

    /// A triangle over x `left ..= right`, y 0 ..= 10, marked by its first vertex's colour.
    fn triangle(left: f32, right: f32, mark: u32, states: [u32; STATES]) -> TestCommand {
        let corner = |x: f32, y: f32| [x.to_bits(), y.to_bits(), 0, 0, mark, 0, 0, 0];
        TestCommand::Fan {
            vertices: vec![corner(left, 0.0), corner(right, 0.0), corner(left, 10.0)],
            states: std::sync::Arc::new(states),
            texture: None,
            bounds: (0.0, 0.0, 0.0, 0.0),
        }
    }

    fn marks(b: &Batches) -> Vec<Vec<u32>> {
        let batch = |x: &Batch| {
            let first = x.first_vertex as usize;
            b.vertices[first..first + x.vertices as usize]
                .iter()
                .step_by(3)
                .map(|v| v[4])
                .collect()
        };
        b.batches.iter().map(batch).collect()
    }

    #[test]
    fn alike_batches_gather_past_others_that_touch_none_of_their_pixels() {
        let plain = [0; STATES];
        let mut blended = plain;
        blended[0x1B] = 1;
        let b = Batches::of(
            &list([
                triangle(0.0, 10.0, 1, plain),
                triangle(20.0, 30.0, 2, blended),
                triangle(40.0, 50.0, 3, plain),
            ]),
            640,
            480,
            None,
        );
        assert_eq!(marks(&b), [vec![1, 3], vec![2]]);
        assert_eq!(b.vertices.len(), 9);
    }

    #[test]
    fn triangles_sharing_only_places_between_whole_ones_keep_their_order() {
        let plain = [0; STATES];
        let mut blended = plain;
        blended[0x1B] = 1;
        // The blended triangle and the last one both cover x from 10.3 to 10.6, where no whole
        // place lies: a target finer than the game's grid has pixel centres there.
        let b = Batches::of(
            &list([
                triangle(0.0, 10.0, 1, plain),
                triangle(10.3, 30.0, 2, blended),
                triangle(9.0, 10.6, 3, plain),
            ]),
            640,
            480,
            None,
        );
        assert_eq!(marks(&b), [vec![1], vec![2], vec![3]]);
    }

    #[test]
    fn a_batch_between_that_touches_the_same_pixels_keeps_the_order() {
        let plain = [0; STATES];
        let mut blended = plain;
        blended[0x1B] = 1;
        // Pixel column 20 is in all three: the order there is the picture.
        let b = Batches::of(
            &list([
                triangle(0.0, 20.0, 1, plain),
                triangle(20.0, 30.0, 2, blended),
                triangle(20.0, 50.0, 3, plain),
            ]),
            640,
            480,
            None,
        );
        assert_eq!(marks(&b), [vec![1], vec![2], vec![3]]);
        // A column apart, with a margin under a pixel, is apart.
        let b = Batches::of(
            &list([
                triangle(0.0, 19.0, 1, plain),
                triangle(20.0, 30.0, 2, blended),
                triangle(31.0, 50.0, 3, plain),
            ]),
            640,
            480,
            None,
        );
        assert_eq!(marks(&b), [vec![1, 3], vec![2]]);
    }

    #[test]
    fn nothing_gathers_past_a_clear() {
        let plain = [0; STATES];
        let b = Batches::of(
            &list([
                triangle(0.0, 10.0, 1, plain),
                TestCommand::Clear([100, 100, 200, 200]),
                triangle(40.0, 50.0, 3, plain),
            ]),
            640,
            480,
            None,
        );
        assert_eq!(b.batches.len(), 3);
        assert_eq!(b.batches[1].scissor, Some([100, 100, 200, 200]));
    }

    #[test]
    fn a_place_that_is_no_number_touches_everything() {
        let plain = [0; STATES];
        let mut blended = plain;
        blended[0x1B] = 1;
        let b = Batches::of(
            &list([
                triangle(0.0, 10.0, 1, plain),
                triangle(f32::NAN, 30.0, 2, blended),
                triangle(40.0, 50.0, 3, plain),
            ]),
            640,
            480,
            None,
        );
        assert_eq!(b.batches.len(), 3);
    }

    /// A textured fan of `corners` places (the same each time), its colour `colour`, its
    /// texture at `texture` (an identity only).
    fn textured(corners: usize, colour: u32, texture: u32, states: [u32; STATES]) -> TestCommand {
        let corner = |n: usize| {
            let (x, y) = (n as f32 * 10.0, (n % 2) as f32 * 10.0);
            [x.to_bits(), y.to_bits(), 0, 0, colour, 0x7F00_0000, 3, 4]
        };
        TestCommand::Fan {
            vertices: (0..corners).map(corner).collect(),
            states: std::sync::Arc::new(states),
            texture: Some(TextureNote {
                pixels: crate::render::TextureId {
                    surface: texture,
                    serial: 0,
                },
                len: 4,
                width: 1,
                height: 1,
                pitch: 4,
                bytes: 4,
                masks: [0xFF_0000, 0xFF00, 0xFF, 0xFF00_0000],
            }),
            bounds: (0.0, 0.0, 0.0, 0.0),
        }
    }

    /// The ground's passes: the first writes depth, the second does not and blends.
    fn passes() -> ([u32; STATES], [u32; STATES]) {
        let mut first = [0; STATES];
        first[0x07] = 1;
        first[0x17] = 4;
        first[0x0E] = 1;
        first[0x1B] = 1;
        first[0x13] = 5;
        first[0x14] = 6;
        let mut second = first;
        second[0x0E] = 0;
        (first, second)
    }

    const OPAQUE: u32 = 0x1000;
    const SEE_THROUGH: u32 = 0x2000;

    fn joined(commands: impl IntoIterator<Item = TestCommand>) -> Batches {
        Batches::of(
            &list(commands),
            640,
            480,
            Some(&|t: &TextureNote| t.pixels.surface == OPAQUE),
        )
    }

    #[test]
    fn a_second_pass_over_the_same_places_joins_the_first() {
        let (first, second) = passes();
        let b = joined([
            textured(4, 0xFF00_0000, OPAQUE, first),
            textured(4, 0x8000_0000, SEE_THROUGH, second),
        ]);
        assert_eq!(b.batches.len(), 1);
        let batch = &b.batches[0];
        assert_eq!(batch.vertices, 6);
        assert_eq!(batch.pipeline.blend, None);
        assert!(batch.pipeline.depth_write);
        assert_ne!(batch.values.flags & SECOND, 0);
        let s = batch.second.expect("a second pass");
        assert_eq!(s.texture, Some(1));
        assert_eq!(b.seconds.len(), 6);
        assert!(
            b.seconds
                .iter()
                .all(|v| *v == [0x8000_0000, 0x7F00_0000, 3, 4])
        );
    }

    #[test]
    fn a_second_pass_joins_only_over_an_opaque_full_alpha_first() {
        let (first, second) = passes();
        // A texture with see-through pixels, corners not fully opaque, other places: apart.
        for commands in [
            [
                textured(4, 0xFF00_0000, SEE_THROUGH, first),
                textured(4, 0x8000_0000, OPAQUE, second),
            ],
            [
                textured(4, 0xFE00_0000, OPAQUE, first),
                textured(4, 0x8000_0000, OPAQUE, second),
            ],
            [
                textured(4, 0xFF00_0000, OPAQUE, first),
                textured(5, 0x8000_0000, OPAQUE, second),
            ],
            [
                textured(4, 0xFF00_0000, OPAQUE, first),
                textured(4, 0x8000_0000, OPAQUE, first),
            ],
        ] {
            let b = joined(commands);
            assert!(b.batches.iter().all(|b| b.second.is_none()));
            assert_eq!(b.seconds.len(), b.vertices.len());
        }
    }

    /// `textured`'s, with a texture of `size` x `size`.
    fn sized(texture: u32, size: usize, x: f32, states: [u32; STATES]) -> TestCommand {
        let mut command = textured(3, 0xFF00_0000, texture, states);
        if let TestCommand::Fan {
            vertices,
            texture: Some(note),
            ..
        } = &mut command
        {
            (note.width, note.height) = (size, size);
            for v in vertices.iter_mut() {
                v[0] = (f32::from_bits(v[0]) + x).to_bits();
            }
        }
        command
    }

    #[test]
    fn layered_batches_join_across_textures_of_the_same_arrays() {
        let plain = [0; STATES];
        let same = |a: &TextureNote, b: &TextureNote| {
            let main = |t: &TextureNote| matches!(t.width, 128 | 256);
            a.width == b.width || (main(a) && main(b))
        };
        let b = Batches::of_layered(
            &list([
                sized(0x10, 128, 0.0, plain),
                sized(0x20, 256, 0.0, plain),
                sized(0x30, 64, 0.0, plain),
                sized(0x40, 128, 100.0, plain),
            ]),
            640,
            480,
            &|_| false,
            same,
            false,
        );
        // The 64 x 64 one apart (it overlaps the first two: the last one, apart from it,
        // gathers back into theirs).
        assert_eq!(b.batches.len(), 2);
        assert_eq!(b.batches[0].vertices, 9);
        let numbers: Vec<u16> = b.vertex_textures.iter().map(|v| v[0]).collect();
        assert_eq!(numbers, [0, 0, 0, 1, 1, 1, 3, 3, 3, 2, 2, 2]);
        assert!(b.vertex_textures.iter().all(|v| v[1] == NO_TEXTURE));
        // Unlayered, one texture a draw.
        let commands = list([sized(0x10, 128, 0.0, plain), sized(0x20, 256, 0.0, plain)]);
        assert_eq!(Batches::of(&commands, 640, 480, None).batches.len(), 2);
        assert!(
            Batches::of(&commands, 640, 480, None)
                .vertex_textures
                .is_empty()
        );
    }

    #[test]
    fn the_first_pass_leaves_the_batch_it_had_joined() {
        let (first, second) = passes();
        let b = joined([
            textured(3, 0xFF00_0000, OPAQUE, first),
            textured(4, 0xFF00_0000, OPAQUE, first),
            textured(4, 0x8000_0000, SEE_THROUGH, second),
        ]);
        // The first fan alone, then the second fan with its pass: their places meet, so in
        // that order.
        assert_eq!(b.batches.len(), 2);
        assert_eq!((b.batches[0].vertices, b.batches[0].second), (3, None));
        assert_eq!(b.batches[1].vertices, 6);
        assert!(b.batches[1].second.is_some());
        assert_eq!(b.seconds[..3], [[0; 4]; 3]);
    }

    /// A vertex at (`x`, `y`) with 1/w `w`.
    fn at(x: f32, y: f32, w: f32) -> Vertex {
        [
            x.to_bits(),
            y.to_bits(),
            0,
            w.to_bits(),
            0xFF00_0000,
            0,
            0,
            0,
        ]
    }

    const TARGET: (f64, f64) = (640.0, 480.0);

    #[test]
    fn triangles_on_either_side_of_a_shared_edge_are_apart() {
        let (p, q) = (at(10.0, 10.0, 1.0), at(50.0, 30.0, 1.0));
        let left = [p, q, at(12.0, 40.0, 1.0)];
        let right = [q, p, at(45.0, 5.0, 1.0)];
        assert!(apart(&left, &right, TARGET) && apart(&right, &left, TARGET));
        // The other's third corner on the same side: they overlap.
        assert!(!apart(&left, &[q, p, at(20.0, 35.0, 1.0)], TARGET));
        // The edge's corners the same in x and y but not in w: the GPU may place them apart.
        let right_w = [at(50.0, 30.0, 0.5), at(10.0, 10.0, 1.0), at(45.0, 5.0, 1.0)];
        assert!(!apart(&left, &right_w, TARGET));
        // The shared edge past the target: clipping may make new corners on it.
        assert!(!apart(&left, &right, (40.0, 480.0)));
        // A third corner within the margin of the shared edge's line.
        assert!(!apart(&left, &[q, p, at(30.0, 19.95, 1.0)], TARGET));
    }

    #[test]
    fn triangles_apart_by_more_than_the_margin_are_apart() {
        let a = [at(0.0, 0.0, 1.0), at(10.0, 0.0, 1.0), at(0.0, 10.0, 1.0)];
        let gap = |d: f32| {
            [
                at(10.0 + d, 0.0, 1.0),
                at(10.0 + d, 10.0, 1.0),
                at(d, 10.0, 1.0),
            ]
        };
        assert!(apart(&a, &gap(1.0), TARGET));
        // A corner on the diagonal (no corners the same), or nearly: they may meet.
        let touching = [at(5.0, 5.0, 1.0), at(15.0, 5.0, 1.0), at(5.0, 15.0, 1.0)];
        assert!(!apart(&a, &touching, TARGET));
        assert!(!apart(&a, &gap(0.1), TARGET));
        // The diagonal's corners the same: an edge shared.
        assert!(apart(&a, &gap(0.0), TARGET));
        // No plain place, or no area: they may meet.
        let far = [
            at(f32::NAN, 0.0, 1.0),
            at(100.0, 0.0, 1.0),
            at(100.0, 10.0, 1.0),
        ];
        assert!(!apart(&a, &far, TARGET));
        let tiny_w = [
            at(100.0, 0.0, 1e-31),
            at(110.0, 0.0, 1.0),
            at(100.0, 10.0, 1.0),
        ];
        assert!(!apart(&a, &tiny_w, TARGET));
        let flat = [
            at(100.0, 0.0, 1.0),
            at(110.0, 0.0, 1.0),
            at(120.0, 0.0, 1.0),
        ];
        assert!(!apart(&flat, &flat, TARGET));
    }

    /// A face's two passes, as a car's: its texture with depth written, then its shine added
    /// without (blended source alpha, one), each pass a batch of its own kind.
    #[test]
    fn layered_faces_drawn_twice_gather_past_neighbours_sharing_an_edge() {
        let mut first = [0; STATES];
        (first[0x07], first[0x17], first[0x0E]) = (1, 4, 1);
        let mut shine = first;
        (shine[0x0E], shine[0x1B], shine[0x13], shine[0x14]) = (0, 1, 5, 2);
        let face = |corners: [Vertex; 3], states: [u32; STATES]| TestCommand::Fan {
            vertices: corners.to_vec(),
            states: std::sync::Arc::new(states),
            texture: None,
            bounds: (0.0, 0.0, 0.0, 0.0),
        };
        let (p, q) = (at(10.0, 10.0, 1.0), at(50.0, 30.0, 1.0));
        let one = [p, q, at(12.0, 40.0, 1.0)];
        let two = [q, p, at(45.0, 5.0, 1.0)];
        let commands = list([
            face(one, first),
            face(one, shine),
            face(two, first),
            face(two, shine),
        ]);
        let same = |_: &TextureNote, _: &TextureNote| true;
        let layered = Batches::of_layered(&commands, 640, 480, &|_| false, same, false);
        assert_eq!(layered.batches.len(), 2);
        assert_eq!(layered.batches[0].pipeline.blend, None);
        assert_eq!(layered.batches[1].pipeline.blend, Some((5, 2)));
        // In each pass's batch, the faces in their order.
        assert_eq!(layered.vertices[..3], one);
        assert_eq!(layered.vertices[3..6], two);
        // Unlayered (the PC's GPU), the areas alone are looked at: they meet.
        assert_eq!(Batches::of(&commands, 640, 480, None).batches.len(), 4);
        // A second face over the first's places is not apart from it: the order stays.
        let over = [p, q, at(20.0, 35.0, 1.0)];
        let commands = list([
            face(one, first),
            face(one, shine),
            face(over, first),
            face(over, shine),
        ]);
        let layered = Batches::of_layered(&commands, 640, 480, &|_| false, same, false);
        assert_eq!(layered.batches.len(), 4);
    }

    /// States with blending of `source` / `target` on, an alpha test `function` against `reference`
    /// (a byte; the test on when `function` is not 0).
    fn fast_states(blend: Option<(u32, u32)>, function: u32, reference: u32) -> [u32; STATES] {
        let mut states = [0; STATES];
        if let Some((source, target)) = blend {
            states[0x1B] = 1;
            states[0x13] = source;
            states[0x14] = target;
        }
        if function != 0 {
            states[0x0F] = 1;
            states[0x19] = function;
            states[0x18] = reference;
        }
        states
    }

    fn corners(alpha: u32, count: usize) -> Vec<Vertex> {
        vec![[0, 0, 0, 0, alpha << 24 | 0x00FF_FFFF, 0, 0, 0]; count]
    }

    #[test]
    fn the_blend_is_off_where_it_keeps_the_source() {
        let full = corners(0xFF, 3);
        let half = corners(0x80, 3);
        let fast = |blend, vertices: &[Vertex], clear| {
            fan_fast_path(&fast_states(blend, 0, 0), vertices, clear).blend_off
        };
        // Source alpha / inverse source alpha: only where the alpha is 1 everywhere.
        assert!(fast(Some((5, 6)), &full, true));
        assert!(!fast(Some((5, 6)), &half, true));
        assert!(!fast(Some((5, 6)), &full, false));
        // One / zero: whatever the alpha.
        assert!(fast(Some((2, 1)), &half, false));
        // Other factors and no blend: not touched.
        assert!(!fast(Some((5, 2)), &full, true));
        assert!(!fast(Some((2, 2)), &full, true));
        assert!(!fast(None, &full, true));
        // One corner under a full alpha spoils the fan.
        let mut mixed = corners(0xFF, 4);
        mixed[2][4] = 0xFE00_0000;
        assert!(!fast(Some((5, 6)), &mixed, true));
    }

    #[test]
    fn the_alpha_test_is_left_out_where_it_cannot_fail() {
        let full = corners(0xFF, 3);
        let half = corners(0x80, 3);
        let free = |function, reference, vertices: &[Vertex], clear| {
            fan_fast_path(&fast_states(None, function, reference), vertices, clear).discard_free
        };
        // Off: for any alpha.
        assert!(free(0, 0, &half, false));
        // Greater, not equal, greater or equal against a reference under 255: where alpha is 1.
        for function in [5, 6, 7] {
            assert!(free(function, 0, &full, true));
            assert!(free(function, 0x80, &full, true));
            assert!(free(function, 254, &full, true));
            assert!(!free(function, 255, &full, true));
            assert!(!free(function, 0x80, &half, true));
            assert!(!free(function, 0x80, &full, false));
        }
        // Never, less, equal, less or equal: can fail at alpha 1.
        for function in [1, 2, 3, 4] {
            assert!(!free(function, 0, &full, true));
        }
        // Always, and a number the shader does not know: pass.
        assert!(free(8, 0, &half, false));
        assert!(free(9, 255, &half, false));
    }

    #[test]
    fn fans_of_the_fast_path_say_so_in_their_batches_and_merge_by_it() {
        let states = fast_states(Some((5, 6)), 7, 0x10);
        let corners = |colour: u32, x: f32| TestCommand::Fan {
            vertices: (0..3)
                .map(|n| {
                    let (px, py) = (x + n as f32 * 4.0, (n % 2) as f32 * 4.0);
                    [px.to_bits(), py.to_bits(), 0, 0, colour, 0, 0, 0]
                })
                .collect(),
            states: std::sync::Arc::new(states),
            texture: None,
            bounds: (0.0, 0.0, 0.0, 0.0),
        };
        let same = |_: &TextureNote, _: &TextureNote| true;
        let commands = list([
            corners(0xFF00_0000, 0.0),
            corners(0xFF00_0000, 100.0),
            corners(0x8000_0000, 200.0),
            corners(0xFF00_0000, 300.0),
        ]);
        let with = Batches::of_layered(&commands, 640, 480, &|_| false, same, true);
        // The full alpha fans leave out the alpha test and the blend and are one batch past the
        // fan between (apart from them); the half alpha one keeps both.
        let kinds: Vec<_> = with
            .batches
            .iter()
            .map(|b| (b.discard_free, b.pipeline.blend, b.vertices))
            .collect();
        assert_eq!(kinds, [(true, None, 9), (false, Some((5, 6)), 3)]);
        // Off, nothing differs from the game's states.
        let without = Batches::of_layered(&commands, 640, 480, &|_| false, same, false);
        assert!(without.batches.iter().all(|b| !b.discard_free));
        assert!(
            without
                .batches
                .iter()
                .all(|b| b.pipeline.blend == Some((5, 6)))
        );
        // The same triangles in the same places either way.
        let places = |b: &Batches| {
            let mut v: Vec<_> = b.vertices.iter().map(|v| v[0]).collect();
            v.sort_unstable();
            v
        };
        assert_eq!(places(&with), places(&without));
    }
}

#[cfg(test)]
mod shader_kind_tests {
    use super::*;

    /// The scene's pixel shader for one pass, with `function`, `mixing` and `flags` as it reads
    /// them (uniforms, or a kind's constants): the pixel's colour before the fog, whether the
    /// fog is mixed in, or `None` for a pixel the alpha test discards.
    fn shaded(
        (function, mixing, flags): (i32, i32, i32),
        reference: f32,
        colour: [f32; 4],
        texel: [f32; 4],
    ) -> Option<([f32; 4], bool)> {
        let mut c = colour;
        if flags & 1 != 0 {
            c = if mixing == 1 || mixing == 7 {
                texel
            } else if mixing == 4 {
                [0, 1, 2, 3].map(|n| texel[n] * colour[n])
            } else {
                let alpha = if flags & 2 != 0 { texel[3] } else { colour[3] };
                [
                    texel[0] * colour[0],
                    texel[1] * colour[1],
                    texel[2] * colour[2],
                    alpha,
                ]
            };
        }
        let passes = match function {
            1 => false,
            2 => c[3] < reference,
            3 => c[3] == reference,
            4 => c[3] <= reference,
            5 => c[3] > reference,
            6 => c[3] != reference,
            7 => c[3] >= reference,
            _ => true,
        };
        (function == 0 || passes).then_some((c, flags & 4 != 0))
    }

    fn values(alpha_function: i32, mixing: i32, flags: i32) -> PixelValues {
        PixelValues {
            fog_colour: [0.0; 4],
            alpha_reference: 0.5,
            alpha_function,
            mixing,
            flags,
        }
    }

    #[test]
    fn a_pass_made_for_its_kind_shades_as_one_that_reads_its_values() {
        let samples = [
            ([1.0, 0.5, 0.25, 1.0], [0.5, 0.5, 0.5, 0.0]),
            ([0.2, 0.4, 0.6, 0.5], [1.0, 0.75, 0.5, 0.5]),
            ([0.9, 0.1, 0.3, 0.25], [0.1, 0.2, 0.3, 1.0]),
            ([0.0, 1.0, 0.0, 0.75], [0.3, 0.3, 0.3, 0.25]),
        ];
        let mut kinds = std::collections::HashSet::new();
        for alpha_function in 0..10 {
            for mixing in 0..9 {
                for flags in 0..16 {
                    let v = values(alpha_function, mixing, flags);
                    let kind = PassKind::of(&v, false);
                    kinds.insert(kind);
                    let read = (alpha_function, mixing, flags);
                    let made = (
                        i32::from(kind.alpha_function),
                        i32::from(kind.mixing),
                        kind.flags(),
                    );
                    for (colour, texel) in samples {
                        assert_eq!(
                            shaded(read, v.alpha_reference, colour, texel),
                            shaded(made, v.alpha_reference, colour, texel),
                            "{read:?} as {kind:?}"
                        );
                    }
                    // What the kind says a pass has is what the shader would use of it.
                    assert_eq!(kind.textured, flags & TEXTURED != 0);
                    assert_eq!(kind.fog, flags & FOG != 0);
                }
            }
        }
        // Far fewer programs than values: 8 tests, by fog or not, by 4 ways with a texture and one without.
        assert_eq!(kinds.len(), 8 * 2 * 5);
    }

    #[test]
    fn a_test_that_keeps_every_pixel_is_no_test_in_the_kind() {
        let kind = PassKind::of(&values(5, 4, TEXTURED), true);
        assert_eq!(kind.alpha_function, 0);
        assert_eq!(
            PassKind::of(&values(5, 4, TEXTURED), false).alpha_function,
            5
        );
        // Comparisons the shader does not know keep every pixel too.
        assert_eq!(
            PassKind::of(&values(8, 4, TEXTURED), false).alpha_function,
            0
        );
        assert_eq!(
            PassKind::of(&values(0, 4, TEXTURED), false).alpha_function,
            0
        );
    }

    #[test]
    fn a_batchs_kind_is_its_passes_kinds() {
        let mut batch = Batch {
            first_vertex: 0,
            vertices: 3,
            pipeline: PipelineKey {
                cull: 1,
                depth_function: 4,
                depth_write: true,
                blend: None,
                colour: true,
            },
            values: values(5, 4, TEXTURED | FOG),
            texture: Some(0),
            linear: true,
            clamp: false,
            mips: false,
            scissor: None,
            second: None,
            discard_free: true,
        };
        let first = PassKind {
            alpha_function: 0,
            mixing: 4,
            textured: true,
            texel_alpha: false,
            fog: true,
        };
        assert_eq!(
            ShaderKind::of(&batch),
            ShaderKind {
                first,
                second: None
            }
        );
        // A second pass's test is kept whatever the first's is: only the first's is known to
        // keep every pixel.
        batch.second = Some(Second {
            values: values(5, 2, TEXTURED | TEXTURE_HAS_ALPHA),
            texture: Some(1),
            linear: true,
            clamp: false,
            mips: false,
        });
        let second = PassKind {
            alpha_function: 5,
            mixing: 0,
            textured: true,
            texel_alpha: true,
            fog: false,
        };
        assert_eq!(
            ShaderKind::of(&batch),
            ShaderKind {
                first,
                second: Some(second)
            }
        );
    }

    #[test]
    fn a_kinds_defines_name_its_numbers_and_only_the_values_it_reads() {
        let first = PassKind::of(&values(5, 4, TEXTURED | TEXTURE_HAS_ALPHA | FOG), false);
        let alone = ShaderKind {
            first,
            second: None,
        };
        assert_eq!(
            alone.defines(),
            "#define MADE_FOR_KIND\n#define ALPHA_FUNCTION 5\n#define MIXING 4\n#define FLAGS 5\n\
             #define FIRST_TEXTURE\n#define FIRST_FOG\n"
        );
        let with_second = ShaderKind {
            first: PassKind::of(&values(0, 2, 0), false),
            second: Some(PassKind::of(
                &values(7, 2, TEXTURED | TEXTURE_HAS_ALPHA),
                false,
            )),
        };
        assert_eq!(
            with_second.defines(),
            "#define MADE_FOR_KIND\n#define ALPHA_FUNCTION 0\n#define MIXING 0\n#define FLAGS 8\n\
             #define NO_DISCARD\n#define SECOND\n#define SECOND_TEXTURE\n\
             #define ALPHA_FUNCTION2 7\n#define MIXING2 0\n#define FLAGS2 3\n"
        );
        assert_eq!(
            ShaderKind::defines_for_any(true),
            "#define FIRST_TEXTURE\n#define FIRST_FOG\n#define SECOND\n#define SECOND_TEXTURE\n\
             #define SECOND_FOG\n#define NO_DISCARD\n"
        );
    }

    #[test]
    fn defines_go_after_the_version_line() {
        assert_eq!(
            with_defines(
                "#version 300 es\nvoid main() {}\n",
                "#define A\n#define B 1\n"
            ),
            "#version 300 es\n#define A\n#define B 1\nvoid main() {}\n"
        );
        assert_eq!(with_defines("no line end", "#define A\n"), "no line end");
    }
}

#[cfg(test)]
mod pipeline_change_tests {
    use super::*;

    fn key() -> PipelineKey {
        PipelineKey {
            cull: 1,
            depth_function: 4,
            depth_write: true,
            blend: None,
            colour: true,
        }
    }

    #[test]
    fn first_key_sets_everything() {
        let c = key().changes_from(None);
        assert!(c.cull && c.depth_function && c.depth_write && c.blend && c.colour);
    }

    #[test]
    fn same_key_sets_nothing() {
        let c = key().changes_from(Some(&key()));
        assert!(!(c.cull || c.depth_function || c.depth_write || c.blend || c.colour));
    }

    #[test]
    fn only_the_differing_state_is_set() {
        let mut k = key();
        k.blend = Some((5, 6));
        let c = k.changes_from(Some(&key()));
        assert_eq!(
            c,
            PipelineChange {
                cull: false,
                depth_function: false,
                depth_write: false,
                blend: true,
                colour: false
            }
        );
        k.depth_write = false;
        k.colour = false;
        let c = k.changes_from(Some(&key()));
        assert!(c.depth_write && c.colour && c.blend && !c.cull && !c.depth_function);
    }

    #[test]
    fn numbers_with_one_gl_value_do_not_count() {
        let mut k = key();
        k.cull = 7; // off, like 1
        k.depth_function = 9; // always, like 8
        let mut was = key();
        was.depth_function = 8;
        let c = k.changes_from(Some(&was));
        assert!(!c.cull && !c.depth_function);
        k.cull = 2;
        assert!(k.changes_from(Some(&was)).cull);
        assert_eq!(k.cull_face(), Some(0x0404));
        assert_eq!(key().depth_func(), 0x0203);
    }
}
