//! `scene::submit_polygon` (0x44CCC0): hands the polygon in the vertex buffer to
//! Direct3D with its texture.
//!
//! In:  nothing: the vertices are at `g_polygonVertices` (0x9605A0), their number in
//!      `g_polygonVertexCount` (0x6240F8), the texture's number in `g_polygonTexture`
//!      (0x6240D4)
//! Out: the original leaves eax, ecx, edx and esi changed; none of its fourteen call
//!      sites (all in `sub_44BA2C`) reads them, and the Rust version keeps every
//!      register
//! With `scene::draw_polygon_fan` (0x44EA36), `scene::check_polygon_vertices`
//! (0x44EA6A), `scene::set_texture` (0x44E77E) and `scene::clear_texture` (0x44E958)
//! inlined, each of which has this as its only caller on the way here.
//! The texture: 0xFFFFFC43 means none; a number of 0x400 or more counts as 0; one that
//! is not `g_currentTexture` is set first. Both go to Direct3D only if there is 3D
//! (`dword_5CDA8C` not 0; a reading).
//! The vertices: each must have x in 0 .. the screen's width, y in 0 .. its height
//! and z in 0 .. 1 (compared on the x87; one that is not a number fails), else "Error"
//! is logged, the buffer shown by `sub_443AEC` and nothing drawn. Then, with 3D, the
//! fan; a result other than 0 is logged as "drawtri" and the number.
//! Shadow mode: the Rust version's calls into the Direct3D layer are worked out
//! without being made, the original's are noted as it makes them (`d3d/recorder.rs`),
//! and the two lists compared.
//! Self-test: a redirected copy of the three functions whose log calls and
//! `sub_443AEC` only note themselves, on polygons of the test's in the game's buffer,
//! with the Direct3D layer's calls swallowed.

use crate::d3d::draw_fan_unclipped::{COUNT_RVA, VERTICES_RVA, polygon};
use crate::d3d::recorder::{Call, record};
use crate::d3d::set_texture::CURRENT_RVA;
use crate::d3d::{NO_TEXTURE, VERTEX_LEN, Vertex};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::submit_polygon", rva(0x44CCC0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TEXTURE_RVA: usize = rva(0x6240D4); // g_polygonTexture
const HAS_3D_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
const WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const CHECKED_RVA: usize = rva(0x62400C); // dword_62400C
const TEXTURES: u32 = 0x400;

const SET_TEXTURE: usize = 0x41ACF5; // d3d::set_texture
const DRAW_FAN: usize = 0x41AD9C; // d3d::draw_fan_unclipped
const SHOW_VERTICES: usize = 0x443AEC; // sub_443AEC
const LOG_L1: usize = 0x401427; // logfile::log_write_l1
const LOG_L1_NL: usize = 0x40142B; // logfile::log_write_l1_nl
const LOG_HEX8_L1: usize = 0x401622; // logfile::log_hex8_l1
const ERROR_TEXT: usize = 0x4DD9A7; // "Error"
const DRAWTRI_TEXT: usize = 0x4DD99F; // "drawtri"

// ---- Rust version ----

/// What the function asks of the code around it.
pub trait Outside {
    /// `d3d::set_texture`.
    fn set_texture(&mut self, texture: u32);
    /// `d3d::draw_fan_unclipped`: Direct3D's result.
    fn draw_fan(&mut self) -> u32;
    /// A vertex is off the screen: "Error" to the log and `sub_443AEC`.
    fn bad_vertex(&mut self);
    /// Direct3D did not draw: "drawtri" and its result to the log.
    fn not_drawn(&mut self, result: u32);
}

/// `scene::check_polygon_vertices`: every vertex on the screen and between the planes.
pub fn vertices_fit(vertices: &[Vertex], width: u32, height: u32) -> bool {
    // As the x87 compares: below 0 or not a number fails, above the limit fails.
    let fits = |bits: u32, limit: f64| {
        let value = f32::from_bits(bits) as f64;
        value >= 0.0 && value <= limit
    };
    vertices
        .iter()
        .all(|v| fits(v[0], width as f64) && fits(v[1], height as f64) && fits(v[2], 1.0))
}

pub unsafe fn submit_polygon(outside: &mut dyn Outside) {
    unsafe {
        let has_3d = game_read::<u32>(HAS_3D_RVA) != 0;
        let texture: u32 = game_read(TEXTURE_RVA);
        if texture == NO_TEXTURE {
            if has_3d {
                outside.set_texture(NO_TEXTURE);
            }
        } else {
            let texture = if texture >= TEXTURES { 0 } else { texture };
            if texture != game_read::<u32>(CURRENT_RVA) && has_3d {
                outside.set_texture(texture);
            }
        }
        let vertices = polygon();
        if !vertices.is_empty() {
            // The check clears this before it looks at a vertex.
            game_write(CHECKED_RVA, 0u32);
        }
        if !vertices_fit(&vertices, game_read(WIDTH_RVA), game_read(HEIGHT_RVA)) {
            outside.bad_vertex();
            return;
        }
        if has_3d {
            let result = outside.draw_fan();
            if result != 0 {
                outside.not_drawn(result);
            }
        }
    }
}

/// The game's Direct3D layer and log.
pub struct Game<'a>(pub &'a Regs);

impl Game<'_> {
    fn call(&self, address: usize, eax: u32) -> u32 {
        let mut r = Regs { eax, ..*self.0 };
        unsafe { call_game(&mut r, game_addr(rva(address))) };
        r.eax
    }
}

impl Outside for Game<'_> {
    fn set_texture(&mut self, texture: u32) {
        self.call(SET_TEXTURE, texture);
    }
    fn draw_fan(&mut self) -> u32 {
        self.call(DRAW_FAN, 0)
    }
    fn bad_vertex(&mut self) {
        self.call(LOG_L1_NL, game_addr(rva(ERROR_TEXT)) as u32);
        self.call(SHOW_VERTICES, 0);
    }
    fn not_drawn(&mut self, result: u32) {
        self.call(LOG_L1, game_addr(rva(DRAWTRI_TEXT)) as u32);
        self.call(LOG_HEX8_L1, result);
    }
}

/// Works out what would be asked without asking it. Direct3D is taken to draw, and
/// the texture asked for becomes the current one, as the layer keeps it.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Foreseen {
    pub calls: Vec<Call>,
    pub bad_vertex: bool,
}

impl Outside for Foreseen {
    fn set_texture(&mut self, texture: u32) {
        self.calls.push(Call::Texture(texture));
        unsafe { game_write(CURRENT_RVA, texture) };
    }
    fn draw_fan(&mut self) -> u32 {
        self.calls.push(Call::Fan(polygon()));
        0
    }
    fn bad_vertex(&mut self) {
        self.bad_vertex = true;
    }
    fn not_drawn(&mut self, _: u32) {}
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        let entry = *r;
        unsafe { submit_polygon(&mut Game(&entry)) };
        return;
    }
    let mut mine = Foreseen::default();
    let texture: u32 = unsafe { game_read(TEXTURE_RVA) };
    unsafe {
        let current: u32 = game_read(CURRENT_RVA);
        submit_polygon(&mut mine);
        game_write(CURRENT_RVA, current);
    }
    let (_, theirs) = record(false, || unsafe { call_original(r) });
    if theirs != mine.calls {
        HOOK.mismatch(|| {
            format!(
                "texture {texture:#x}, {} vertices: original {theirs:x?}; rust {:x?}",
                polygon().len(),
                mine.calls
            )
        });
    }
}

// ---- self-test ----

/// `submit_polygon`, `draw_polygon_fan` and `check_polygon_vertices`: address, length,
/// first bytes, the offsets of their calls.
const SUBMIT: (usize, usize, [u8; 8], &[usize]) = (
    0x44CCC0,
    0x2F,
    [0xA1, 0xD4, 0x40, 0x62, 0x00, 0x3D, 0x43, 0xFC],
    &[0x1D, 0x24, 0x29],
);
const FAN: (usize, usize, [u8; 8], &[usize]) = (
    0x44EA36,
    0x2F,
    [0xE8, 0x2F, 0x00, 0x00, 0x00, 0x72, 0x27, 0x83],
    &[0x00, 0x10, 0x21, 0x28],
);
const CHECK: (usize, usize, [u8; 8], &[usize]) = (
    0x44EA6A,
    0x9A,
    [0x8B, 0x0D, 0xF8, 0x40, 0x62, 0x00, 0x83, 0xF9],
    &[0x8D, 0x93],
);

/// 1 after the copy logged "Error" or called `sub_443AEC`.
static BAD_VERTEX: AtomicU32 = AtomicU32::new(0);

fn handler(_: u32, _: &mut Regs) {
    BAD_VERTEX.store(1, Ordering::SeqCst);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The copies: the check's log and `sub_443AEC` only note themselves; the fan's
    // "drawtri" lines are never reached, the layer's calls being swallowed.
    let copies = unsafe {
        let noting = |callee: usize| {
            [LOG_L1, LOG_L1_NL, LOG_HEX8_L1, SHOW_VERTICES]
                .iter()
                .any(|&known| rva(known) == callee)
                .then(|| stand_in(0))
        };
        let check =
            RedirectedCopy::new(game_addr(rva(CHECK.0)), CHECK.1, &CHECK.2, CHECK.3, &noting);
        check.and_then(|check| {
            let fan = RedirectedCopy::new(game_addr(rva(FAN.0)), FAN.1, &FAN.2, FAN.3, &|callee| {
                if callee == rva(CHECK.0) {
                    Some(check.addr())
                } else {
                    noting(callee)
                }
            });
            fan.and_then(|fan| {
                RedirectedCopy::new(
                    game_addr(rva(SUBMIT.0)),
                    SUBMIT.1,
                    &SUBMIT.2,
                    SUBMIT.3,
                    &|callee| (callee == rva(FAN.0)).then(|| fan.addr()),
                )
                .map(|submit| (check, fan, submit))
            })
        })
    };
    let (_check, _fan, submit) = match copies {
        Ok(copies) => copies,
        Err(why) => return t.skip(why),
    };
    let (width, height): (u32, u32) = unsafe { (game_read(WIDTH_RVA), game_read(HEIGHT_RVA)) };
    if width == 0 || height == 0 {
        return t.skip("there is no screen mode");
    }
    let regions = [
        (game_addr(VERTICES_RVA) as u32, 8 * VERTEX_LEN),
        at(COUNT_RVA),
        at(TEXTURE_RVA),
        at(HAS_3D_RVA),
        at(CURRENT_RVA),
        at(CHECKED_RVA),
    ];
    let saved = capture(&regions);
    let f = |value: f32| value.to_bits();
    let (w, h) = (width as f32, height as f32);
    let vertex = |x: f32, y: f32, z: f32| -> Vertex {
        [f(x), f(y), f(z), f(1.0), 0xFF80_4020, 0, f(0.25), f(0.75)]
    };
    let good = vec![
        vertex(0.0, 0.0, 0.0),
        vertex(w, 0.0, 1.0),
        vertex(w, h, 0.5),
        vertex(10.5, h - 0.5, 0.999),
    ];
    let with = |n: usize, bad: Vertex| {
        let mut polygon = good.clone();
        polygon[n] = bad;
        polygon
    };
    let polygons: Vec<(&str, Vec<Vertex>)> = vec![
        ("on the screen", good.clone()),
        ("a triangle", good[..3].to_vec()),
        ("no vertices", Vec::new()),
        ("x below 0", with(0, vertex(-0.001, 5.0, 0.5))),
        ("x past the width", with(2, vertex(w + 0.5, 5.0, 0.5))),
        ("y below 0", with(1, vertex(5.0, -1.0, 0.5))),
        ("y past the height", with(3, vertex(5.0, h + 1.0, 0.5))),
        ("z below 0", with(0, vertex(5.0, 5.0, -0.1))),
        ("z past 1", with(3, vertex(5.0, 5.0, 1.001))),
        ("x not a number", with(1, vertex(f32::NAN, 5.0, 0.5))),
        ("z not a number", with(2, vertex(5.0, 5.0, f32::NAN))),
        ("x infinite", with(0, vertex(f32::INFINITY, 5.0, 0.5))),
        ("minus zero", with(0, vertex(-0.0, -0.0, -0.0))),
    ];
    // (the polygon's texture, the current one)
    let textures = [
        (NO_TEXTURE, 5u32),
        (NO_TEXTURE, NO_TEXTURE),
        (5, 5),
        (6, 5),
        (0x3FF, 0),
        (0x400, 0),
        (0x400, 7),
        (0xFFFF_FFFF, 3),
        (0, NO_TEXTURE),
    ];
    for (what, polygon) in &polygons {
        for (texture, current) in textures {
            for has_3d in [1u32, 0, 0xFFFF_FFFF] {
                let run = |rust: bool| unsafe {
                    restore(&regions, &vec![0xAA; saved.len()]);
                    for (n, v) in polygon.iter().enumerate() {
                        ((game_addr(VERTICES_RVA) + n * VERTEX_LEN) as *mut Vertex)
                            .write_unaligned(*v);
                    }
                    game_write(COUNT_RVA, polygon.len() as u32);
                    game_write(TEXTURE_RVA, texture);
                    game_write(HAS_3D_RVA, has_3d);
                    game_write(CURRENT_RVA, current);
                    BAD_VERTEX.store(0, Ordering::SeqCst);
                    set_handler(handler);
                    if rust {
                        let mut mine = Foreseen::default();
                        submit_polygon(&mut mine);
                        (mine.calls, mine.bad_vertex, capture(&regions))
                    } else {
                        let mut ro = Regs::SENTINEL;
                        let (_, calls) = record(true, || call_game(&mut ro, submit.addr()));
                        (
                            calls,
                            BAD_VERTEX.load(Ordering::SeqCst) == 1,
                            capture(&regions),
                        )
                    }
                };
                let (theirs, mine) = (run(false), run(true));
                t.check(theirs == mine, || {
                    format!(
                        "{what}, texture {texture:#x} over {current:#x}, 3D {has_3d:#x}: original {:x?} bad vertex {}; rust {:x?} bad vertex {}",
                        theirs.0, theirs.1, mine.0, mine.1
                    )
                });
            }
        }
    }
    restore(&regions, &saved);
    t.finish()
}
