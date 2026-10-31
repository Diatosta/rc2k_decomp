//! The Direct3D the game's 3D code stands on, for the standalone program: textures of
//! our own, the scene's beginning and end, the depth buffer's clearing. Not ports, as
//! in `platform.rs`: each takes the place of a function of the game's Direct3D layer,
//! keeps what that function leaves for the game's own code, and does the rest itself.
//! No self-tests: there is no original to compare with.
//!
//! The textures. The game keeps two pools of records of 0x78 bytes, up to 0x140 each:
//! system textures (`unk_5B521C`, count `dword_5B5218`), into which pictures are put,
//! and video textures (`unk_5ABC18`, count `dword_5ABC14`), which are drawn with. A
//! record (`TextureRecord`): +0 its kind (1 system, 2 video), +4 width, +8 height, +0xC its pixel kind's
//! description, +0x10 levels, +0x14 a surface a level, +0x38 a texture a level, +0x5C
//! (system) the alpha from a second picture, +0x60 locked, +0x64 (video) the system
//! texture it holds, +0x68 its texture, +0x6C its handle, +0x70 a material. Here every
//! surface is an object of ours, a pointer to `VTABLE` and its number, whose `Lock`
//! hands out memory of ours: so `gfx::upload_texture_picture`, which locks through the
//! surface's own table, fills it unchanged.
//!
//! - `d3d::find_texture_formats` (0x419F5A, under Direct3D's start-up; called here
//!   when the screen mode is set): the four descriptions as the device would have
//!   found its formats, 555, 1555, 4444 and 8888, and the three kinds pointed at them
//!   the original's way.
//! - `gfx::allocate_textures` (0x4051ED), `gfx::release_all_textures` (0x4051D3).
//! - `scene::load_texture` (0x44E778): a system texture into a video texture, level
//!   for level (the original's `Blt`).
//! - `d3d::begin_scene` (0x41ABC7), `d3d::end_scene` (0x41AB8A): the layer's own
//!   states and its "in a scene" switch.
//! - `d3d::clear_viewport` (0x44E9D9): the depth buffer.
//!
//! The drawing is noted and drawn in tiles over the processor's threads when something is to
//! see it (see `flush`).

use crate::d3d::cards_store::CARDS_3D;
use crate::errors::FatalError;
use crate::framework::{GameHook, RawHook, Regs, log, rva};
use crate::gfx::reports::REPORTS;
use crate::gfx::screen_state::DRAW_SURFACE;
use crate::gfx::texture_upload::TEXTURE_TABLES;
use crate::gfx::texture_upload::{TextureFormat, TextureRecord};
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
use crate::scene::draw_store::DRAW;
use crate::session::SCREEN;
use crate::system::find_entry::find_entry;
use std::sync::Mutex;

// ---- game data ----

const MOST: u32 = 0x140;
const LEVELS: u32 = 9;
const NO_TEXTURE: u32 = 0xFFFF_FC43;
/// A `DDERR`, for a surface that is not there.
const NOT_THERE: i32 = 0x8876_0000u32 as i32 | 0x1C2;

// ---- DirectX interface ids (nothing reads them) ----

/// IDA's address of the first interface ids (four of 16 bytes and five zero dwords).
pub const GUIDS_A_START: usize = 0x5BE81C;
/// Their length.
pub const GUIDS_A_LEN: usize = 0x5BE870 - GUIDS_A_START;
/// IDA's address of the last two interface ids.
pub const GUIDS_B_START: usize = 0x5BE874;
/// Their length.
pub const GUIDS_B_LEN: usize = 0x20;

/// The ids the original passed to `QueryInterface` (DirectDraw and Direct3D); the
/// standalone's platform does not use them.
pub static GUIDS_A: [crate::session::Dword; 21] = crate::gfx::dwords([
    0xb3a6f3e0, 0x11cf2b43, 0xaa00dea2, 0x5633b900, 0x57805885, 0x11cf6eec, 0x23a84194, 0x270ec103,
    0x6aae1ec1, 0x11d0662a, 0xaa009d88, 0x6ab7bb00, 0x93281502, 0x11d08cf8, 0xa000ab89, 0x294105c9,
    0, 0, 0, 0, 0,
]);

/// The last two.
pub static GUIDS_B: [crate::session::Dword; 8] = crate::gfx::dwords([
    0xa35ff56a, 0x11d09fda, 0xc000df8f, 0x9d18d94f, 0xa35ff56b, 0x11d09fda, 0xc000df8f, 0x9d18d94f,
]);

/// Where the first ids are (`moved::MOVED`'s home for IDA 0x5BE81C).
pub fn guids_a_home() -> usize {
    (crate::pointers::game_address(GUIDS_A.as_ptr())) as usize
}

/// Where the last two are (`moved::MOVED`'s home for IDA 0x5BE874).
pub fn guids_b_home() -> usize {
    (crate::pointers::game_address(GUIDS_B.as_ptr())) as usize
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn the_interface_ids_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &GUIDS_A,
            GUIDS_A_START,
            GUIDS_A_START + GUIDS_A_LEN,
        );
        crate::gfx::first_values_are_the_games(
            &GUIDS_B,
            GUIDS_B_START,
            GUIDS_B_START + GUIDS_B_LEN,
        );
        assert_eq!(crate::moved::place(GUIDS_A_START), guids_a_home());
        assert_eq!(crate::moved::place(GUIDS_B_START), guids_b_home());
    }
}

// ---- the texture formats ----

/// `d3d::find_texture_formats`, for a screen of `bits`.
pub fn describe_texture_formats(bits: u32) {
    // (bits, masks blue, green, red, alpha)
    let formats: [(u32, [u32; 4]); 4] = [
        (16, [0x001F, 0x03E0, 0x7C00, 0]),
        (16, [0x001F, 0x03E0, 0x7C00, 0x8000]),
        (16, [0x000F, 0x00F0, 0x0F00, 0xF000]),
        (32, [0xFF, 0xFF00, 0xFF_0000, 0xFF00_0000]),
    ];
    for (cell, (format_bits, masks)) in TEXTURE_TABLES.descriptions.iter().zip(formats) {
        let (layout, number) = crate::gfx::describe_format::describe_masks(masks);
        cell.set(crate::gfx::texture_upload::TextureFormat {
            found: 1,
            bytes_per_pixel: format_bits / 8,
            format: rc2k_core::gfx::pixels::PixelFormat { layout, masks },
            number,
            // DirectDraw's pixel format: size, flags (RGB, with alpha), four-CC, bits, the
            // masks red, green, blue, alpha.
            dd: [
                0x20,
                if masks[3] == 0 { 0x40 } else { 0x41 },
                0,
                format_bits,
                masks[2],
                masks[1],
                masks[0],
                masks[3],
            ],
        });
    }
    // By handle (`texture_upload::FORMAT_TAG`).
    let at = |n: usize| crate::gfx::texture_upload::FORMAT_TAG | n as u32;
    let kinds = if bits == 32 {
        [at(3); 3]
    } else {
        [at(0), at(1), at(2)]
    };
    for (n, kind) in kinds.into_iter().enumerate() {
        TEXTURE_TABLES.kinds[1 + n].set(kind);
    }
}

// ---- the textures' memory ----

/// A record of a pool (0 system, 1 video).
fn record_at(pool: usize, number: u32) -> &'static crate::session::Shared<TextureRecord> {
    &TEXTURE_TABLES.pool(pool).records[number as usize]
}

struct Level {
    /// The pixels a region while the level lives (locked, the game's code writes them by
    /// address: `pointers`). (Declared first: fields drop in order, and the region must go before the memory does.)
    _lent: crate::pointers::Lent,
    pixels: Vec<u8>,
    pitch: u32,
}

/// A surface's number: (pool, texture, level), the pool 0 for system, 1 for video.
fn surface_number(pool: usize, texture: u32, level: u32) -> usize {
    (pool * MOST as usize + texture as usize) * LEVELS as usize + level as usize
}

const SURFACES: usize = 2 * MOST as usize * LEVELS as usize;

static LEVELS_MEMORY: Mutex<Vec<Option<Level>>> = Mutex::new(Vec::new());
/// A level's surface as the game holds it: its number and 1 (phase 11, 2026-10-26: before,
/// an object whose first dword was a method table with `Lock` at +0x64 and `Unlock` at
/// +0x80, which the game's code called).
fn object(number: usize) -> u32 {
    number as u32 + 1
}

/// DirectDraw's `Lock` of a texture's surface (`d3d::lock_texture`,
/// `gfx::upload_texture_picture`): the pixels' address at +0x24 and the pitch at +0x10 of
/// the surface description at `description`; `NOT_THERE` for a surface with no memory.
pub fn lock_surface(surface: u32, description: u32) -> i32 {
    let number = surface.wrapping_sub(1) as usize;
    // The texture is to be written: the noted drawing that reads it first (only then:
    // the mud's, the map's and the HUD's textures are locked every frame).
    let pixels = LEVELS_MEMORY
        .lock()
        .unwrap()
        .get(number)
        .and_then(|l| l.as_ref())
        .map(|l| crate::pointers::game_address(l.pixels.as_ptr()));
    if pixels.is_none_or(reads_texture) {
        flush();
    }
    let mut memory = LEVELS_MEMORY.lock().unwrap();
    match memory.get_mut(number).and_then(|l| l.as_mut()) {
        Some(level) => {
            let what = "a surface description";
            crate::pointers::write_or_end(
                description + 0x24,
                crate::pointers::game_address(level.pixels.as_mut_ptr()),
                what,
            );
            crate::pointers::write_or_end(description + 0x10, level.pitch, what);
            0
        }
        None => NOT_THERE,
    }
}

/// DirectDraw's `Unlock`: the pixels are the level's already.
pub fn unlock_surface(_surface: u32) -> i32 {
    0
}

// ---- drawing ----

const STATES: usize = 0x40;

/// For the standalone's timing line (`platform::show`): fans drawn and the microseconds
/// spent drawing them (at the flushes).
pub static FANS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static RASTER_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

// ---- the frame's drawing, in tiles ----
//
// The fans and the depth buffer's clears are noted as the game makes them and drawn when
// something is to see or change what they draw into (`flush`): the frame shown, the screen
// locked for 2D drawing, a texture locked, made, loaded or released, the scene ended, the
// screen mode changed. Then the screen is cut into tiles that the processor's threads draw
// at once, each tile its fans and clears in the game's order: each pixel sees the same
// operations in the same order as one by one. A texture is noted by where its memory is
// (that does not change before the flush) and drawn from as it is then.

/// A texture's first level as noted: its memory and how it is laid out.
#[derive(Clone, Copy)]
struct TextureNote {
    /// The memory's address (a number: the tiles' threads read it).
    pixels: usize,
    len: usize,
    width: usize,
    height: usize,
    pitch: usize,
    bytes: usize,
    masks: [u32; 4],
}

enum Noted {
    Fan {
        vertices: Vec<crate::rasteriser::Vertex>,
        states: Box<[u32; STATES]>,
        texture: Option<TextureNote>,
        /// The vertices' bounds on the screen: (left, top, right, bottom).
        bounds: (f32, f32, f32, f32),
    },
    /// The depth buffer back to 1.0 in a rectangle (left, top, right, bottom).
    Clear([usize; 4]),
}

/// What the noted drawing draws into.
#[derive(Clone, Copy, PartialEq)]
struct Into {
    pixels: usize,
    pitch: usize,
    width: usize,
    height: usize,
    bits: u32,
}

struct Pending {
    into: Option<Into>,
    noted: Vec<Noted>,
    /// The memory of each texture the noted fans read.
    textures: Vec<u32>,
}

static PENDING: Mutex<Pending> = Mutex::new(Pending {
    into: None,
    noted: Vec::new(),
    textures: Vec::new(),
});

/// Whether a noted fan reads the texture whose memory is at `pixels`.
fn reads_texture(pixels: u32) -> bool {
    PENDING.lock().unwrap().textures.contains(&pixels)
}

/// The flushes that drew something and the pixels written (for `platform::report_timing`).
/// The parallel drawing's wall time and the threads' time inside the tiles (summed), in
/// microseconds: how well the tiles keep the threads busy.
pub static PARALLEL_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static TILE_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static FLUSHES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static PIXELS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Below this many noted fans a flush draws on its own thread, the screen one tile.
const FEW: usize = 48;
// Safety: the pointers in it are the game's memory and ours, used only by the game's
// thread and, during a flush, by the tiles' threads it waits for.
unsafe impl Send for Pending {}

const TILE: usize = 64;

/// The screen as the game has it now.
fn screen() -> Option<Into> {
    let pixels: u32 = DRAW_SURFACE.present_pixels.get();
    (pixels != 0).then(|| Into {
        pixels: pixels as usize,
        pitch: DRAW_SURFACE.present_pitch.get() as usize,
        width: SCREEN.width.get() as usize,
        height: SCREEN.height.get() as usize,
        bits: SCREEN.bits.get(),
    })
}

/// Notes `what` for the screen as it is now (the noted so far drawn first when that is
/// another).
fn note(what: Noted) {
    let Some(into) = screen() else {
        return;
    };
    let mut pending = PENDING.lock().unwrap();
    if pending.into.is_some_and(|i| i != into) {
        draw_noted(&mut pending);
    }
    pending.into = Some(into);
    if let Noted::Fan {
        texture: Some(t), ..
    } = &what
        && !pending.textures.contains(&(t.pixels as u32))
    {
        pending.textures.push(t.pixels as u32);
    }
    pending.noted.push(what);
}

/// Draws everything noted, the screen in tiles over the threads.
pub fn flush() {
    let mut pending = PENDING.lock().unwrap();
    draw_noted(&mut pending);
}

/// The small frame draw_noted draws into, on systems that have one (`handheld::Small`).
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
#[derive(Clone, Copy)]
struct Small {
    pixels: *mut u8,
    width: usize,
    height: usize,
    depth: *mut f32,
}

/// The noted things' screen places for the handheld's small frame.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
fn scale_noted(noted: &mut [Noted]) {
    const BY: f32 = crate::handheld::SMALLER as f32;
    let by = crate::handheld::SMALLER;
    for what in noted {
        match what {
            Noted::Fan {
                vertices, bounds, ..
            } => {
                for v in vertices.iter_mut() {
                    v[0] = (f32::from_bits(v[0]) / BY).to_bits();
                    v[1] = (f32::from_bits(v[1]) / BY).to_bits();
                }
                *bounds = (bounds.0 / BY, bounds.1 / BY, bounds.2 / BY, bounds.3 / BY);
            }
            Noted::Clear([left, top, right, bottom]) => {
                *left /= by;
                *top /= by;
                *right = right.div_ceil(by);
                *bottom = bottom.div_ceil(by);
            }
        }
    }
}

fn draw_noted(pending: &mut Pending) {
    use crate::rasteriser::{Frame, States, Texture};
    use rayon::prelude::*;
    #[cfg_attr(
        not(all(target_os = "linux", target_arch = "aarch64")),
        allow(unused_mut) // only the handheld changes them
    )]
    let mut noted = std::mem::take(&mut pending.noted);
    pending.textures.clear();
    let Some(into) = pending.into.take() else {
        return;
    };
    if noted.is_empty() {
        return;
    }
    FLUSHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    dump_if_asked(&noted, &into);
    let started = std::time::Instant::now();
    let fans = noted
        .iter()
        .filter(|n| matches!(n, Noted::Fan { .. }))
        .count() as u64;
    // The handheld draws a race's 3D smaller (`handheld`): the noted things made to fit a small
    // frame, which takes the screen's picture so far and gives it back enlarged.
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let small = (into.bits == 32 && crate::handheld::draws_small()).then(|| {
        crate::pointers::check_or_end(
            into.pixels as u32,
            into.pitch * into.height,
            "the screen's pixels",
        );
        scale_noted(&mut noted);
        crate::handheld::small_frame(
            crate::pointers::host_at(into.pixels),
            into.pitch,
            into.width,
            into.height,
        )
    });
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    let small: Option<Small> = None;
    with_depth(|depth| {
        if small.is_none() && (depth.width != into.width || depth.height != into.height) {
            return;
        }
        // The screen's pixels are lent (a region the size of its rows), checked here; the
        // depths are Rust's own, borrowed for the frame.
        crate::pointers::check_or_end(
            into.pixels as u32,
            into.pitch * into.height,
            "the screen's pixels",
        );
        // SAFETY: the pixels checked above, `height` rows of `pitch` bytes; the depths
        // `width` x `height` floats (the sizes compared above), borrowed mutably here; the
        // game's code does not run while the frame is drawn (under `PENDING`'s lock).
        let mut frame = unsafe {
            match small {
                Some(s) => Frame::new(s.pixels, s.width * 4, s.width, s.height, 32, s.depth),
                None => Frame::new(
                    crate::pointers::host_at(into.pixels),
                    into.pitch,
                    into.width,
                    into.height,
                    into.bits,
                    depth.values.as_mut_ptr(),
                ),
            }
        };
        let mut tiles = frame.tiles(if noted.len() < FEW { 0 } else { TILE });
        let noted = &noted;
        let draw_tile = move |tile: &mut crate::rasteriser::Tile| {
            let tile_started = std::time::Instant::now();
            let clip = *tile.clip();
            let mut written = 0u64;
            for what in noted {
                match what {
                    Noted::Fan {
                        vertices,
                        states,
                        texture,
                        bounds: (left, top, right, bottom),
                    } => {
                        // Not this tile's (the bounds a pixel wider for the edges' rule).
                        if *right < clip.left as f32 - 1.0
                            || *left > clip.right as f32 + 1.0
                            || *bottom < clip.top as f32 - 1.0
                            || *top > clip.bottom as f32 + 1.0
                        {
                            continue;
                        }
                        let texture = texture.map(|t| Texture {
                            // SAFETY: a noted texture is not written while it is drawn
                            // from: locking it flushes the noted fans first (`lock`).
                            pixels: unsafe {
                                crate::pointers::GamePtr::new(t.pixels as u32)
                                    .slice(t.len, "a texture's pixels")
                            },
                            width: t.width,
                            height: t.height,
                            pitch: t.pitch,
                            bytes: t.bytes,
                            masks: t.masks,
                        });
                        written += tile.draw_fan(vertices, &States(&states[..]), texture.as_ref());
                    }
                    Noted::Clear(rect) => tile.clear_depth(*rect),
                }
            }
            PIXELS.fetch_add(written, std::sync::atomic::Ordering::Relaxed);
            TILE_MICROS.fetch_add(
                tile_started.elapsed().as_micros() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
        };
        let parallel_started = std::time::Instant::now();
        if tiles.len() == 1 {
            draw_tile(&mut tiles[0]);
        } else {
            tiles.par_iter_mut().for_each(draw_tile);
        }
        PARALLEL_MICROS.fetch_add(
            parallel_started.elapsed().as_micros() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
    });
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    if let Some(s) = small {
        crate::handheld::enlarge(
            s,
            crate::pointers::host_at(into.pixels),
            into.pitch,
            into.width,
            into.height,
        );
    }
    use std::sync::atomic::Ordering::Relaxed;
    FANS.fetch_add(fans, Relaxed);
    RASTER_MICROS.fetch_add(started.elapsed().as_micros() as u64, Relaxed);
}

/// With `RC2K_DUMP=n` in the environment: the n-th flush of 1000 fans or more written to
/// `rc2k_frame_dump.bin`, for replaying it outside the program (the benchmark): "RC2KDUMP",
/// the screen's width, height, bits; the textures (each once: width, height, pitch, bytes,
/// four masks, its bytes' length and bytes); then each noted thing in order: 1 a fan (its
/// vertices' count and the vertices, 8 dwords each; the 0x40 states; its texture's number
/// or -1), 2 a depth clear (left, top, right, bottom). All little-endian dwords.
fn dump_if_asked(noted: &[Noted], into: &Into) {
    use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
    static SEEN: AtomicU32 = AtomicU32::new(0);
    let Some(wanted) = std::env::var("RC2K_DUMP")
        .ok()
        .and_then(|n| n.parse::<u32>().ok())
    else {
        return;
    };
    let fans = noted
        .iter()
        .filter(|n| matches!(n, Noted::Fan { .. }))
        .count();
    if fans < 1000 || SEEN.fetch_add(1, Relaxed) + 1 != wanted {
        return;
    }
    let mut out: Vec<u8> = b"RC2KDUMP".to_vec();
    let put = |out: &mut Vec<u8>, v: u32| out.extend(v.to_le_bytes());
    for v in [into.width as u32, into.height as u32, into.bits] {
        put(&mut out, v);
    }
    let mut textures: Vec<usize> = Vec::new();
    for n in noted {
        if let Noted::Fan {
            texture: Some(t), ..
        } = n
            && !textures.contains(&t.pixels)
        {
            textures.push(t.pixels);
        }
    }
    put(&mut out, textures.len() as u32);
    for &at in &textures {
        let t = noted
            .iter()
            .find_map(|n| match n {
                Noted::Fan {
                    texture: Some(t), ..
                } if t.pixels == at => Some(t),
                _ => None,
            })
            .unwrap();
        for v in [t.width, t.height, t.pitch, t.bytes] {
            put(&mut out, v as u32);
        }
        for m in t.masks {
            put(&mut out, m);
        }
        put(&mut out, t.len as u32);
        let what = "a noted texture's pixels";
        out.extend(
            crate::pointers::GamePtr::new(t.pixels as u32)
                .read_bytes(t.len, what)
                .unwrap_or_else(|e| crate::fatal::raise::end(what, e)),
        );
    }
    put(&mut out, noted.len() as u32);
    for n in noted {
        match n {
            Noted::Fan {
                vertices,
                states,
                texture,
                ..
            } => {
                put(&mut out, 1);
                put(&mut out, vertices.len() as u32);
                for v in vertices {
                    for &d in v {
                        put(&mut out, d);
                    }
                }
                for &st in states.iter() {
                    put(&mut out, st);
                }
                let number = texture
                    .as_ref()
                    .and_then(|t| textures.iter().position(|&p| p == t.pixels))
                    .map_or(u32::MAX, |n| n as u32);
                put(&mut out, number);
            }
            Noted::Clear(r) => {
                put(&mut out, 2);
                for &v in r {
                    put(&mut out, v as u32);
                }
            }
        }
    }
    let written = std::fs::write("rc2k_frame_dump.bin", &out);
    crate::framework::log(&format!(
        "platform_3d: flush {wanted} of {fans} fans dumped to rc2k_frame_dump.bin ({} bytes, {} textures): {written:?}",
        out.len(),
        textures.len()
    ));
}

/// A fan into the screen, with the states and the current texture
/// (`g_currentTexture`, a video texture's number) as the game set them: noted, drawn at
/// the next flush.
/// A triangle strip (`D3DPT_TRIANGLESTRIP`): triangle n of vertices n, n + 1, n + 2, every
/// second one's first two swapped so that all keep the strip's winding (as Direct3D does).
pub fn draw_strip(vertices: &[crate::rasteriser::Vertex]) {
    for n in 0..vertices.len().saturating_sub(2) {
        let (a, b, c) = (vertices[n], vertices[n + 1], vertices[n + 2]);
        if n % 2 == 0 {
            draw_fan(&[a, b, c]);
        } else {
            draw_fan(&[b, a, c]);
        }
    }
}

/// The time noting fans (`draw_fan`), in microseconds.
pub static NOTE_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn draw_fan(vertices: &[crate::rasteriser::Vertex]) {
    let noting = std::time::Instant::now();
    note_fan(vertices);
    NOTE_MICROS.fetch_add(
        noting.elapsed().as_micros() as u64,
        std::sync::atomic::Ordering::Relaxed,
    );
}

fn note_fan(vertices: &[crate::rasteriser::Vertex]) {
    if vertices.len() < 3 {
        return;
    }
    let states: [u32; STATES] = std::array::from_fn(|n| CARDS_3D.kept_states[n].get());
    let current: u32 = CARDS_3D.current_texture.get();
    let memory = LEVELS_MEMORY.lock().unwrap();
    let texture = (current < MOST)
        .then(|| {
            let record = record_at(1, current).get();
            let level = memory.get(surface_number(1, current, 0))?.as_ref()?;
            if record.format == 0 {
                return None;
            }
            let format =
                crate::pointers::read_or_end::<TextureFormat>(record.format, "a texture format");
            Some(TextureNote {
                pixels: (crate::pointers::game_address(level.pixels.as_ptr())) as usize,
                len: level.pixels.len(),
                width: record.width as usize,
                height: record.height as usize,
                pitch: level.pitch as usize,
                bytes: format.bytes_per_pixel as usize,
                masks: format.format.masks,
            })
        })
        .flatten();
    drop(memory);
    let f = |v: &crate::rasteriser::Vertex, n: usize| f32::from_bits(v[n]);
    let mut bounds = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for v in vertices {
        let (x, y) = (f(v, 0), f(v, 1));
        bounds = (
            bounds.0.min(x),
            bounds.1.min(y),
            bounds.2.max(x),
            bounds.3.max(y),
        );
    }
    if !(bounds.0.is_finite()
        && bounds.1.is_finite()
        && bounds.2.is_finite()
        && bounds.3.is_finite())
    {
        bounds = (f32::MIN, f32::MIN, f32::MAX, f32::MAX);
    }
    note(Noted::Fan {
        vertices: vertices.to_vec(),
        states: Box::new(states),
        texture,
        bounds,
    });
}

/// The screen into system texture `number`'s first level, as DirectDraw's `Blt` without
/// rectangles does it: stretched to the texture's size, each texel the screen's pixel at
/// its place scaled (nearest), in the texture's format (its alpha full). For
/// `d3d::copy_screen_to_texture` (the mirror's picture).
pub fn copy_screen_to_system_texture(number: u32) {
    flush();
    if number >= MOST {
        return;
    }
    let Some(screen) = screen() else {
        return;
    };
    let record = record_at(0, number).get();
    if record.format == 0 || record.width == 0 || record.height == 0 {
        return;
    }
    let format = crate::pointers::read_or_end::<TextureFormat>(record.format, "a texture format");
    let bytes = format.bytes_per_pixel as usize;
    let masks = format.format.masks;
    let mut memory = LEVELS_MEMORY.lock().unwrap();
    let Some(level) = memory
        .get_mut(surface_number(0, number, 0))
        .and_then(|l| l.as_mut())
    else {
        return;
    };
    let (w, h) = (record.width as usize, record.height as usize);
    // A component (0 .. 255) into a mask's place.
    let put = |c: u32, mask: u32| -> u32 {
        if mask == 0 {
            return 0;
        }
        let shift = mask.trailing_zeros();
        let most = mask >> shift;
        ((c * most + 127) / 255) << shift
    };
    for y in 0..h {
        let sy = y * screen.height / h;
        let what = "the screen's row";
        let row = crate::pointers::GamePtr::new((screen.pixels + sy * screen.pitch) as u32)
            .read_bytes(screen.width * if screen.bits == 16 { 2 } else { 4 }, what)
            .unwrap_or_else(|e| crate::fatal::raise::end(what, e));
        for x in 0..w {
            let sx = x * screen.width / w;
            let (red, green, blue) = if screen.bits == 16 {
                let p = u16::from_le_bytes([row[sx * 2], row[sx * 2 + 1]]) as u32;
                let (r5, g6, b5) = (p >> 11 & 0x1F, p >> 5 & 0x3F, p & 0x1F);
                (r5 << 3 | r5 >> 2, g6 << 2 | g6 >> 4, b5 << 3 | b5 >> 2)
            } else {
                let p = u32::from_le_bytes(row[sx * 4..sx * 4 + 4].try_into().unwrap());
                (p >> 16 & 0xFF, p >> 8 & 0xFF, p & 0xFF)
            };
            let texel = put(blue, masks[0]) | put(green, masks[1]) | put(red, masks[2]) | masks[3];
            let at = y * level.pitch as usize + x * bytes;
            if let Some(to) = level.pixels.get_mut(at..at + bytes) {
                to.copy_from_slice(&texel.to_le_bytes()[..bytes.min(4)]);
            }
        }
    }
}

/// An indexed triangle list (Direct3D's `DrawIndexedPrimitive` with type 4, as the scene's
/// batches and the HUD hand it): each three indices (16 bits) one triangle of the
/// vertices, drawn as a fan of three; `vertex(n)` the vertex `n` (below `vertex_count`),
/// `index(k)` the `k`-th index. An index past the vertices ends the list (Direct3D would
/// have refused the call).
pub fn draw_triangles(
    vertex_count: u32,
    index_count: u32,
    vertex: impl Fn(u16) -> crate::d3d::Vertex,
    index: impl Fn(usize) -> u16,
) {
    for t in 0..(index_count / 3) as usize {
        let corners = [index(3 * t), index(3 * t + 1), index(3 * t + 2)];
        if corners.iter().any(|&n| u32::from(n) >= vertex_count) {
            return;
        }
        draw_fan(&corners.map(&vertex));
    }
}

/// `draw_triangles` on a batch's own buffers. A count past them ends the game (the
/// original read on into what follows).
pub fn draw_indexed_bytes<const V: usize, const I: usize>(
    vertices: &crate::session::Bytes<V>,
    vertex_count: u32,
    indices: &crate::session::Bytes<I>,
    index_count: u32,
) {
    use crate::d3d::VERTEX_LEN;
    let past = |what: &str| -> ! {
        crate::fatal::raise::end(what, crate::errors::BadInput("past its batch's buffer"))
    };
    draw_triangles(
        vertex_count,
        index_count,
        |n| {
            let at = n as usize * VERTEX_LEN;
            if at + VERTEX_LEN > V {
                past("a vertex")
            }
            std::array::from_fn(|k| vertices.get_u32(at + 4 * k))
        },
        |k| {
            if 2 * k + 2 > I {
                past("an index")
            }
            u16::from_le_bytes([indices.get(2 * k), indices.get(2 * k + 1)])
        },
    );
}

/// `sub_41ADF5` (`d3d::draw_indexed`), the platform's draw of the race's batches: the same
/// call as `draw_triangles` (DrawIndexedPrimitive, type 4, flags 0x1C; the original takes
/// edx the index count, ecx the indices, eax the vertex count, ebx the vertices). Returns
/// Direct3D's result: 0, drawn. Its edx leftover is read by no caller (`lq.py -s`: the
/// three sites read eax only, as `race::flush_draw_batch` and the two batch flushes do).
pub fn draw_indexed<const V: usize, const I: usize>(
    vertices: &crate::session::Bytes<V>,
    vertex_count: u32,
    indices: &crate::session::Bytes<I>,
    index_count: u32,
) -> u32 {
    draw_indexed_bytes(vertices, vertex_count, indices, index_count);
    0
}

// ---- gfx::allocate_textures (0x4051ED) ----

static ALLOCATE: RawHook = RawHook::new("gfx::allocate_textures", rva(0x4051ED));

fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}

/// A size by its number in `dword_5ABA98`.
fn size_of(code: u32) -> Option<u32> {
    let entry = find_entry(code, TEXTURE_TABLES.sizes_addr(), 5).ok()??;
    Some(crate::pointers::read_or_end(entry + 4, "a texture size"))
}

/// One texture more in a pool: its record made and its levels' memory.
fn make_texture(
    pool: usize,
    description: u32,
    width: u32,
    height: u32,
    levels: u32,
) -> Result<u32, FatalError> {
    let full = if pool == 0 { 0x80D1_8501 } else { 0x80D1_8601 };
    let pool_vars = TEXTURE_TABLES.pool(pool);
    let number: u32 = pool_vars.count.get();
    if number >= MOST {
        return Err(fatal(full));
    }
    pool_vars.count.set(number + 1);
    pool_vars.records[number as usize].update(|record| {
        record.kind = if pool == 0 { 1 } else { 2 };
        record.width = width;
        record.height = height;
        record.format = description;
        record.levels = levels;
        let bytes = crate::pointers::read_or_end::<TextureFormat>(description, "a texture format")
            .bytes_per_pixel;
        let mut memory = LEVELS_MEMORY.lock().unwrap();
        if memory.is_empty() {
            memory.resize_with(SURFACES, || None);
        }
        for level in 0..LEVELS {
            let surface = surface_number(pool, number, level);
            if level < levels {
                let (w, h) = ((width >> level).max(1), (height >> level).max(1));
                let pixels = vec![0; (w * h * bytes) as usize];
                memory[surface] = Some(Level {
                    _lent: crate::pointers::Lent::new(&pixels),
                    pixels,
                    pitch: w * bytes,
                });
                record.surfaces[level as usize] = object(surface);
            } else {
                memory[surface] = None;
                record.surfaces[level as usize] = 0;
            }
        }
        if pool == 1 {
            record.holds = u32::MAX;
            record.texture = 1;
            record.handle = number + 1;
        }
    });
    Ok(number)
}

fn allocate_textures(
    video: u32,
    system: u32,
    sizes: u32,
    kind: u32,
    mips: u32,
) -> Result<(), FatalError> {
    let width = size_of(sizes & 0xFF).ok_or(fatal(0x80A1_8641))?;
    let height = if sizes & 0x1000 != 0 {
        size_of((sizes >> 8) & 0xFF).ok_or(fatal(0x80A1_8642))?
    } else {
        width
    };
    let description: u32 = TEXTURE_TABLES.kind_entry(kind as usize & 0xFF);
    TEXTURE_TABLES.alpha_kind.set((kind == 3) as u32);
    let levels = if CARDS_3D.mips.get() == 1 { mips } else { 0 }.wrapping_add(1);
    if levels > LEVELS {
        return Err(fatal(0x80A1_8643));
    }
    TEXTURE_TABLES
        .first_video
        .set(TEXTURE_TABLES.video.count.get());
    TEXTURE_TABLES
        .first_system
        .set(TEXTURE_TABLES.system.count.get());
    for _ in 0..video {
        make_texture(1, description, width, height, levels)?;
    }
    let alpha: u32 = TEXTURE_TABLES.alpha_kind.get();
    for _ in 0..system {
        let number = make_texture(0, description, width, height, levels)?;
        TEXTURE_TABLES.system.records[number as usize].update(|r| r.keeps_alpha = alpha);
    }
    Ok(())
}

/// `gfx::allocate_textures` for Rust callers: the first new video and system textures'
/// numbers; a failure raises its fatal error (a fatal error that returns leaves the two
/// counts as they were handed in, as the original's registers did).
pub fn allocate_textures_for(
    video: u32,
    system: u32,
    sizes: u32,
    kind: u32,
    mips: u32,
) -> (u32, u32) {
    match allocate_textures_now(video, system, sizes, kind, mips) {
        Ok(first) => first,
        Err(e) => {
            // The registers the report shows: the original's at the call.
            let r = Regs {
                eax: video,
                ebx: system,
                ecx: sizes,
                edx: kind,
                ebp: mips,
                ..Regs::default()
            };
            crate::fatal::raise::refused_in(ALLOCATE.name(), e, &r);
            (video, system)
        }
    }
}

/// `gfx::allocate_textures`: `video` textures more in the video pool and `system` in the
/// system one, of the sizes (`sizes`: the width's number in the low byte, with 0x1000 the
/// height's in the next), pixel kind (`kind`) and, with mipmaps on, `mips` levels more;
/// the first new video and system textures' numbers.
pub fn allocate_textures_now(
    video: u32,
    system: u32,
    sizes: u32,
    kind: u32,
    mips: u32,
) -> Result<(u32, u32), FatalError> {
    ALLOCATE.count_call();
    flush();
    let reports: u32 = REPORTS.on.get();
    REPORTS.on.set(0);
    let done = allocate_textures(video, system, sizes, kind, mips);
    REPORTS.on.set(reports);
    done?;
    Ok((
        TEXTURE_TABLES.first_video.get(),
        TEXTURE_TABLES.first_system.get(),
    ))
}

// ---- gfx::release_all_textures (0x4051D3) ----

static RELEASE: RawHook = RawHook::new("gfx::release_all_textures", rva(0x4051D3));

/// `sub_41A3B2` for every texture of a pool, and the count 0.
fn release_pool(pool: usize) {
    let pool_vars = TEXTURE_TABLES.pool(pool);
    let count: u32 = pool_vars.count.get();
    let mut memory = LEVELS_MEMORY.lock().unwrap();
    for number in 0..count.min(MOST) {
        for level in 0..LEVELS {
            if let Some(slot) = memory.get_mut(surface_number(pool, number, level)) {
                *slot = None;
            }
        }
        pool_vars.records[number as usize].update(|record| {
            record.surfaces = [0; 9];
            record.textures = [0; 9];
            record.texture = 0;
            record.material = 0;
            record.kind = 0;
            record.levels = 0;
            record.holds = u32::MAX;
        });
    }
    pool_vars.count.set(0);
}

/// `gfx::release_all_textures` for Rust callers, as a call by its address.
pub fn release_textures() {
    RELEASE.count_call();
    release_all_textures();
}

/// `gfx::release_all_textures`: the drawing noted flushed, both pools emptied (also what
/// `gfx::set_screen_mode` does first, 0x404C55).
pub fn release_all_textures() {
    flush();
    let what = "the release text";
    let text = crate::files::cstring(
        crate::pointers::GamePtr::new(VIEWER_PANEL.text_addr(texts::SVM_RELEASE_TEXTURES)),
        what,
    )
    .unwrap_or_else(|e| crate::fatal::raise::end(what, e));
    crate::logfile::write_line::log_text(&text, true, 2);
    release_pool(1);
    release_pool(0);
}

// ---- scene::load_texture (0x44E778) ----

static LOAD: RawHook = RawHook::new("scene::load_texture", rva(0x44E778));

/// `scene::load_texture` for Rust callers: video texture `video` holds system texture
/// `system`. Every register kept.
pub fn load_texture(system: u32, video: u32) {
    LOAD.count_call();
    hold_texture("scene::load_texture", system, video);
}

/// Video texture `video` made to hold system texture `system` (`scene::load_texture`,
/// `d3d::copy_to_device_texture`): its record's +0x64 the system one, its levels the
/// system one's where both have the level.
pub fn hold_texture(caller: &str, system: u32, video: u32) {
    flush();
    if system >= MOST || video >= MOST {
        log(&format!(
            "platform: {caller}: texture {system:#x} into {video:#x} is past the records"
        ));
        return;
    }
    record_at(1, video).update(|held| held.holds = system);
    let levels = record_at(0, system).get().levels.min(LEVELS);
    let mut memory = LEVELS_MEMORY.lock().unwrap();
    for level in 0..levels {
        let from = surface_number(0, system, level);
        let to = surface_number(1, video, level);
        let pixels = match memory.get(from).and_then(|l| l.as_ref()) {
            Some(level) => level.pixels.clone(),
            None => continue,
        };
        if let Some(Some(target)) = memory.get_mut(to) {
            let n = pixels.len().min(target.pixels.len());
            target.pixels[..n].copy_from_slice(&pixels[..n]);
        }
    }
}

// ---- d3d::begin_scene (0x41ABC7), d3d::end_scene (0x41AB8A) ----

static BEGIN: RawHook = RawHook::new("d3d::begin_scene", rva(0x41ABC7));

/// The layer's own states at a scene's beginning: (value, state).
const BEGIN_STATES: [(u32, u32); 12] = [
    (1, 0x1F),
    (1, 0x07),
    (1, 0x0E),
    (2, 0x09),
    (4, 0x17),
    (1, 0x0F),
    (0, 0x18),
    (5, 0x19),
    (1, 0x04),
    (2, 0x11),
    (2, 0x12),
    (0, 0x23),
];

/// `d3d::begin_scene` for Rust callers: the layer's states at a scene's beginning, no
/// texture set. It cannot fail (the original leaves eax 0, the carry clear, zero set).
pub fn begin_layer() {
    BEGIN.count_call();
    CARDS_3D.in_scene.set(1u32);
    for (value, state) in BEGIN_STATES
        .into_iter()
        .chain([(CARDS_3D.fog_state.get(), 0x1A)])
    {
        crate::d3d::set_render_state::set_render_state(value, state);
    }
    // `sub_41ACDE`: no texture is the current one, then none is set.
    CARDS_3D.current_texture.set(0xFFFF_FC44u32);
    crate::d3d::set_texture::set_texture(NO_TEXTURE);
}

static END: RawHook = RawHook::new("d3d::end_scene", rva(0x41AB8A));

/// `d3d::end_scene` for Rust callers: true when a scene was open and is ended (the
/// original then leaves eax 0 and the zero flag set; the carry always clear).
pub fn end_layer() -> bool {
    END.count_call();
    flush();
    {
        if CARDS_3D.in_scene.get() == 1 {
            CARDS_3D.in_scene.set(0u32);
            return true;
        }
    }
    false
}

// ---- d3d::clear_viewport (0x44E9D9) ----

static CLEAR: RawHook = RawHook::new("d3d::clear_viewport", rva(0x44E9D9));

/// The depth buffer, a float a pixel of the screen: 1.0 is the farthest.
pub struct Depth {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

pub static DEPTH: Mutex<Option<Depth>> = Mutex::new(None);

/// The depth buffer at the screen's size, made or made again.
pub fn with_depth<R>(f: impl FnOnce(&mut Depth) -> R) -> R {
    let (width, height) = (SCREEN.width.get() as usize, SCREEN.height.get() as usize);
    let mut depth = DEPTH.lock().unwrap();
    if depth
        .as_ref()
        .is_none_or(|d| d.width != width || d.height != height)
    {
        *depth = Some(Depth {
            width,
            height,
            values: vec![1.0; width * height],
        });
    }
    f(depth.as_mut().unwrap())
}

/// `d3d::clear_viewport` for Rust callers: the depth buffer cleared over the rectangle
/// `DRAW.cleared_rectangle` holds (the original leaves eax 0).
pub fn clear_viewport() {
    CLEAR.count_call();
    let [left, top, right, bottom] =
        crate::session::read_bits(&DRAW.cleared_rectangle).map(|v| v as i32);
    let c = |v: i32| v.max(0) as usize;
    note(Noted::Clear([c(left), c(top), c(right), c(bottom)]));
}

pub static HOOKS: &[&dyn GameHook] = &[&ALLOCATE, &RELEASE, &LOAD, &BEGIN, &END, &CLEAR];
