//! The Direct3D the game's 3D code stands on, for the standalone program: textures of
//! our own, the scene's beginning and end, the depth buffer's clearing. Not ports, as
//! in `platform.rs`: each takes the place of a function of the game's Direct3D layer,
//! keeps what that function leaves for the game's own code, and does the rest itself.
//! There is no original to compare with.
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

// Not FFI: `LentBytes::slice_mut` (the target's pixels) is a memory primitive.
#![allow(unsafe_code)]

use crate::d3d::owner::d3d;
use crate::errors::FatalError;
use crate::framework::log;
use crate::gfx::owner::gfx;
use crate::gfx::texture_upload::TextureFormat;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
use crate::render::{Command, STATES, Target, TextureNote};
use crate::system::find_entry::find_entry;
use rc2k_core::fatal::Context;
use std::sync::Mutex;

// ---- game data ----

const MOST: u32 = 0x140;
const LEVELS: u32 = 9;
const NO_TEXTURE: u32 = 0xFFFF_FC43;
/// A `DDERR`, for a surface that is not there.
const NOT_THERE: i32 = 0x8876_0000u32 as i32 | 0x1C2;

// ---- DirectX interface ids (nothing reads them) ----

/// IDA's address of the first interface ids (four of 16 bytes and five zero dwords).
#[cfg(test)]
pub const GUIDS_A_START: usize = 0x5BE81C;
/// Their length.
#[cfg(test)]
pub const GUIDS_A_LEN: usize = 0x5BE870 - GUIDS_A_START;
/// IDA's address of the last two interface ids.
#[cfg(test)]
pub const GUIDS_B_START: usize = 0x5BE874;
/// Their length.
#[cfg(test)]
pub const GUIDS_B_LEN: usize = 0x20;

/// The ids the original passed to `QueryInterface` (DirectDraw and Direct3D); the
/// standalone's platform does not use them.
#[cfg(test)]
pub static GUIDS_A: [u32; 21] = [
    0xb3a6f3e0, 0x11cf2b43, 0xaa00dea2, 0x5633b900, 0x57805885, 0x11cf6eec, 0x23a84194, 0x270ec103,
    0x6aae1ec1, 0x11d0662a, 0xaa009d88, 0x6ab7bb00, 0x93281502, 0x11d08cf8, 0xa000ab89, 0x294105c9,
    0, 0, 0, 0, 0,
];

/// The last two.
#[cfg(test)]
pub static GUIDS_B: [u32; 8] = [
    0xa35ff56a, 0x11d09fda, 0xc000df8f, 0x9d18d94f, 0xa35ff56b, 0x11d09fda, 0xc000df8f, 0x9d18d94f,
];

// ---- the texture formats ----

/// `d3d::find_texture_formats`, for a screen of `bits`.
pub fn describe_texture_formats(bits: u32, gfx: &crate::gfx::owner::Gfx) {
    // (bits, masks blue, green, red, alpha)
    let formats: [(u32, [u32; 4]); 4] = [
        (16, [0x001F, 0x03E0, 0x7C00, 0]),
        (16, [0x001F, 0x03E0, 0x7C00, 0x8000]),
        (16, [0x000F, 0x00F0, 0x0F00, 0xF000]),
        (32, [0xFF, 0xFF00, 0xFF_0000, 0xFF00_0000]),
    ];
    for (cell, (format_bits, masks)) in gfx.card.texture_tables.descriptions.iter().zip(formats) {
        let (layout, number) =
            crate::gfx::describe_format::describe_masks(&gfx.pixel.formats, masks);
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
        gfx.card.texture_tables.kinds[1 + n].set(kind);
    }
}

// ---- the textures' memory ----

/// A record of a pool (0 system, 1 video).
fn record_at(pool: usize, number: u32) -> &'static crate::gfx::texture_upload::RecordCell {
    &gfx().card.texture_tables.pool(pool).records[number as usize]
}

struct Level {
    /// The pixels, a region while the level lives (locked, the game's code writes them by
    /// address: `pointers`).
    pixels: crate::pointers::LentBytes,
    pitch: u32,
}

/// A surface's number: (pool, texture, level), the pool 0 for system, 1 for video.
fn surface_number(pool: usize, texture: u32, level: u32) -> usize {
    (pool * MOST as usize + texture as usize) * LEVELS as usize + level as usize
}

const SURFACES: usize = 2 * MOST as usize * LEVELS as usize;

static LEVELS_MEMORY: Mutex<Vec<Option<Level>>> = Mutex::new(Vec::new());
/// Bumped (under `LEVELS_MEMORY`'s lock) when a level is made or let go: a fan's level as
/// noted (`Pending.level`) is kept until then.
static LEVELS_CHANGED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn levels_changed() {
    LEVELS_CHANGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}
/// A level's surface as the game holds it: its number and 1 (phase 11, 2026-10-26: before,
/// an object whose first dword was a method table with `Lock` at +0x64 and `Unlock` at
/// +0x80, which the game's code called).
fn object(number: usize) -> u32 {
    number as u32 + 1
}

/// DirectDraw's `Lock` of a texture's surface (`d3d::lock_texture`,
/// `gfx::upload_texture_picture`): the pixels' address at +0x24 and the pitch at +0x10 of
/// the surface description at `description`; `NOT_THERE` for a surface with no memory.
pub fn lock_surface(surface: u32, description: &crate::gfx::upload::SurfaceDesc) -> i32 {
    let number = surface.wrapping_sub(1) as usize;
    // The texture is to be written: the noted drawing that reads it first (only then:
    // the mud's, the map's and the HUD's textures are locked every frame).
    let pixels = LEVELS_MEMORY
        .lock()
        .unwrap()
        .get(number)
        .and_then(|l| l.as_ref())
        .map(|l| l.pixels.ptr());
    if pixels.is_none_or(reads_texture) {
        flush();
    }
    if let Some(pixels) = pixels {
        crate::render::texture_written(pixels);
    }
    let mut memory = LEVELS_MEMORY.lock().unwrap();
    match memory.get_mut(number).and_then(|l| l.as_mut()) {
        Some(level) => {
            description.pixels.set(level.pixels.addr());
            description.pitch.set(level.pitch);
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

// ---- the frame's drawing, in tiles ----
//
// The fans and the depth buffer's clears are noted as the game makes them and drawn when
// something is to see or change what they draw into (`flush`): the frame shown, the screen
// locked for 2D drawing, a texture locked, made, loaded or released, the scene ended, the
// screen mode changed. Then the screen is cut into tiles that the processor's threads draw
// at once, each tile its fans and clears in the game's order: each pixel sees the same
// operations in the same order as one by one. A texture is noted by where its memory is
// (that does not change before the flush) and drawn from as it is then.

struct Pending {
    into: Option<Target>,
    noted: Vec<Command>,
    /// The memory of each texture the noted fans read.
    textures: Vec<crate::pointers::GamePtr>,
    /// The states as the last fan had them, until one changes (`forget_states`).
    states: Option<std::sync::Arc<[u32; STATES]>>,
    /// The last fan's texture level, the lock of the levels spared while it holds.
    level: Option<NotedLevel>,
}

/// A surface's level as a fan read it.
#[derive(Clone, Copy)]
struct NotedLevel {
    surface: usize,
    /// `LEVELS_CHANGED` when it was read.
    changes: u64,
    /// Its pixels' place (bounded to them) and length and its pitch; `None` for no level.
    level: Option<(crate::pointers::GamePtr, usize, u32)>,
}

/// The noted drawing's way to the backend: nothing noted is left behind.
fn draw_noted(pending: &mut Pending) {
    if crate::workers::index() != 0 {
        // The parts' drawing is put together on the game's thread: a flush here would draw
        // this part ahead of the earlier parts.
        crate::framework::log("platform: a flush on a drawing thread (its drawing out of order)");
    }
    // Room for as many as there has been: growing to them shows in a profile.
    let room = pending.noted.capacity();
    let commands = std::mem::replace(&mut pending.noted, Vec::with_capacity(room));
    pending.textures.clear();
    let Some(target) = pending.into.take() else {
        return;
    };
    if commands.is_empty() {
        return;
    }
    crate::render::dump::dump_if_asked(&commands, &target);
    crate::render::dump::hash_if_asked(&commands);
    crate::render::draw(&target, commands);
}

thread_local! {
    /// The drawing noted since the last flush. Only the game's thread notes and flushes
    /// (fans, mode changes, the frame shown), so it is that thread's own: no lock per fan.
    static PENDING: std::cell::RefCell<Pending> = const {
        std::cell::RefCell::new(Pending {
            into: None,
            noted: Vec::new(),
            textures: Vec::new(),
            states: None,
            level: None,
        })
    };
}

/// Whether a noted fan reads the texture whose memory is at `pixels`.
fn reads_texture(pixels: crate::pointers::GamePtr) -> bool {
    PENDING.with_borrow(|p| p.textures.contains(&pixels))
}

/// The screen as the game has it now.
pub fn screen() -> Option<Target> {
    let drawing = &gfx().drawing;
    let pixels: u32 = drawing.draw_surface.present_pixels.get();
    (pixels != 0).then(|| Target {
        pixels: pixels as usize,
        pitch: drawing.draw_surface.present_pitch.get() as usize,
        width: crate::system::owner::system().session.screen.width.get() as usize,
        height: crate::system::owner::system().session.screen.height.get() as usize,
        bits: crate::system::owner::system().session.screen.bits.get(),
    })
}

/// Notes `what` for the screen as it is now (the noted so far drawn first when that is
/// another).
fn note(what: impl FnOnce(&mut Pending) -> Command) {
    let Some(into) = screen() else {
        return;
    };
    PENDING.with_borrow_mut(|pending| {
        if pending.into.is_some_and(|i| i != into) {
            draw_noted(pending);
        }
        pending.into = Some(into);
        let what = what(pending);
        if let Command::Fan {
            texture: Some(t), ..
        } = &what
            // Most fans run on in the last one's texture: that is looked at first.
            && pending.textures.last() != Some(&t.pixels)
            && !pending.textures.contains(&t.pixels)
        {
            pending.textures.push(t.pixels);
        }
        pending.noted.push(what);
    });
}

/// The commands noted since the last flush (a part's mark, `workers`).
pub fn noted_len() -> usize {
    PENDING.with_borrow(|p| p.noted.len())
}

/// The commands noted from `mark` on, taken out, the states and level as the last fan had them
/// forgotten (`workers`: each part notes as from nothing).
pub fn take_noted_from(mark: usize) -> Vec<Command> {
    PENDING.with_borrow_mut(|p| {
        p.states = None;
        p.level = None;
        p.noted.split_off(mark.min(p.noted.len()))
    })
}

/// `commands` noted after those so far (`workers`), their textures with them.
pub fn put_noted(commands: Vec<Command>) {
    PENDING.with_borrow_mut(|p| {
        for c in &commands {
            if let Command::Fan {
                texture: Some(t), ..
            } = c
                && !p.textures.contains(&t.pixels)
            {
                p.textures.push(t.pixels);
            }
        }
        p.states = None;
        p.level = None;
        p.noted.extend(commands);
    });
}

/// Draws everything noted, the screen in tiles over the threads.
pub fn flush() {
    PENDING.with_borrow_mut(draw_noted);
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
    // Timed only where something shows it: a look at the clock for every fan shows in a profile.
    if cfg!(debug_ui) {
        let noting = std::time::Instant::now();
        note_fan(vertices);
        NOTE_MICROS.fetch_add(
            noting.elapsed().as_micros() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
    } else {
        note_fan(vertices);
    }
}

/// A render state has changed: the next fan's are read again.
pub fn forget_states() {
    PENDING.with_borrow_mut(|p| p.states = None);
}

fn note_fan(vertices: &[crate::rasteriser::Vertex]) {
    if vertices.len() < 3 {
        return;
    }
    let current: u32 = crate::d3d::render_cache::texture();
    let record = (current < MOST)
        .then(|| record_at(1, current).get())
        .filter(|r| r.format != 0);
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
    note(|pending| Command::Fan {
        texture: record.and_then(|record| {
            let level = level_of(pending, surface_number(1, current, 0))?;
            let format = format_at(record.format);
            Some(TextureNote {
                pixels: level.0,
                len: level.1,
                width: record.width as usize,
                height: record.height as usize,
                pitch: level.2 as usize,
                bytes: format.bytes_per_pixel as usize,
                masks: format.format.masks,
            })
        }),
        vertices: vertices.into(),
        states: std::sync::Arc::clone(
            pending
                .states
                .get_or_insert_with(|| std::sync::Arc::new(crate::d3d::render_cache::all_kept())),
        ),
        bounds,
    });
}

/// Surface `surface`'s level (its pixels' address and length, its pitch), as the last fan
/// with it read it while no level has been made or let go since.
fn level_of(
    pending: &mut Pending,
    surface: usize,
) -> Option<(crate::pointers::GamePtr, usize, u32)> {
    let changes = LEVELS_CHANGED.load(std::sync::atomic::Ordering::Relaxed);
    if let Some(noted) = pending.level
        && noted.surface == surface
        && noted.changes == changes
    {
        return noted.level;
    }
    let memory = LEVELS_MEMORY.lock().unwrap();
    let level = memory
        .get(surface)
        .and_then(|l| l.as_ref())
        .map(|l| (l.pixels.ptr(), l.pixels.len(), l.pitch));
    // Read under the lock: a level made or let go meanwhile has bumped it.
    let changes = LEVELS_CHANGED.load(std::sync::atomic::Ordering::Relaxed);
    pending.level = Some(NotedLevel {
        surface,
        changes,
        level,
    });
    level
}

/// The texture format at `format`: the description a handle names, else (a kind read past
/// the table) a number read through the game's memory.
fn format_at(format: u32) -> TextureFormat {
    match gfx().card.texture_tables.description(format) {
        Some(d) => d.get(),
        None => crate::pointers::read_or_end::<TextureFormat>(format, "a texture format"),
    }
}

/// The screen into system texture `number`'s first level, as DirectDraw's `Blt` without
/// rectangles does it: stretched to the texture's size, each texel the screen's pixel at
/// its place scaled (nearest), in the texture's format (its alpha full). For
/// `d3d::copy_screen_to_texture` (the mirror's picture).
pub fn copy_screen_to_system_texture(number: u32) {
    flush();
    if let Some(screen) = screen() {
        crate::render::bring_back(&screen);
    }
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
    let format = format_at(record.format);
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
    let rows = crate::gfx::rows::Rows::new(
        screen.pixels as u32,
        screen.pitch as u32,
        screen.height as u32,
    );
    for y in 0..h {
        let sy = y * screen.height / h;
        let what = "the screen's row";
        let row = rows
            .read(
                sy as u32,
                0,
                screen.width * if screen.bits == 16 { 2 } else { 4 },
                what,
            )
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
            // SAFETY: this thread draws; the texture's level is not locked while it is copied to.
            if let Some(to) = unsafe { level.pixels.slice_mut() }.get_mut(at..at + bytes) {
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

/// 0x41ADF5 (`d3d::draw_indexed`), the platform's draw of the race's batches: the same
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

fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}

/// A size by its number in `dword_5ABA98`.
fn size_of(code: u32) -> Option<u32> {
    let entry = find_entry(
        code,
        crate::pointers::GamePtr::to(&gfx().card.texture_tables.sizes),
        5,
    )
    .ok()??;
    Some(
        entry
            .offset(4)
            .read("a texture size")
            .unwrap_or_else(|e| crate::fatal::raise::end("a texture size", e)),
    )
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
    let pool_vars = gfx().card.texture_tables.pool(pool);
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
        let bytes = format_at(description).bytes_per_pixel;
        let mut memory = LEVELS_MEMORY.lock().unwrap();
        levels_changed();
        if memory.is_empty() {
            memory.resize_with(SURFACES, || None);
        }
        for level in 0..LEVELS {
            let surface = surface_number(pool, number, level);
            if level < levels {
                let (w, h) = ((width >> level).max(1), (height >> level).max(1));
                memory[surface] = Some(Level {
                    pixels: crate::pointers::LentBytes::from_vec(vec![0; (w * h * bytes) as usize]),
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
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), FatalError> {
    let card = &gfx.card;
    let width = size_of(sizes & 0xFF).ok_or(fatal(0x80A1_8641))?;
    let height = if sizes & 0x1000 != 0 {
        size_of((sizes >> 8) & 0xFF).ok_or(fatal(0x80A1_8642))?
    } else {
        width
    };
    let description: u32 = card.texture_tables.kind_entry(kind as usize & 0xFF);
    card.texture_tables.alpha_kind.set((kind == 3) as u32);
    let levels = if d3d().cards.mips.get() == 1 { mips } else { 0 }.wrapping_add(1);
    if levels > LEVELS {
        return Err(fatal(0x80A1_8643));
    }
    card.texture_tables
        .first_video
        .set(card.texture_tables.video.count.get());
    card.texture_tables
        .first_system
        .set(card.texture_tables.system.count.get());
    for _ in 0..video {
        make_texture(1, description, width, height, levels)?;
    }
    let alpha: u32 = card.texture_tables.alpha_kind.get();
    for _ in 0..system {
        let number = make_texture(0, description, width, height, levels)?;
        card.texture_tables.system.records[number as usize].update(|r| r.keeps_alpha = alpha);
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
    let gfx = crate::gfx::owner::gfx();
    match allocate_textures_now(video, system, sizes, kind, mips, gfx) {
        Ok(first) => first,
        Err(e) => {
            // The registers the report shows: the original's at the call.
            let r = Context {
                eax: video,
                ebx: system,
                ecx: sizes,
                edx: kind,
                ebp: mips,
                ..Context::default()
            };
            crate::fatal::raise::refused_in("gfx::allocate_textures", e, &r);
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
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(u32, u32), FatalError> {
    flush();
    let reports: u32 = gfx.reports.on.get();
    gfx.reports.on.set(0);
    let done = allocate_textures(video, system, sizes, kind, mips, gfx);
    gfx.reports.on.set(reports);
    done?;
    Ok((
        gfx.card.texture_tables.first_video.get(),
        gfx.card.texture_tables.first_system.get(),
    ))
}

// ---- gfx::release_all_textures (0x4051D3) ----

/// 0x41A3B2 for every texture of a pool, and the count 0.
fn release_pool(pool: usize, gfx: &crate::gfx::owner::Gfx) {
    let pool_vars = gfx.card.texture_tables.pool(pool);
    let count: u32 = pool_vars.count.get();
    let mut memory = LEVELS_MEMORY.lock().unwrap();
    levels_changed();
    for number in 0..count.min(MOST) {
        for level in 0..LEVELS {
            if let Some(slot) = memory.get_mut(surface_number(pool, number, level)) {
                if let Some(gone) = slot {
                    crate::render::texture_gone(gone.pixels.ptr());
                }
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
    let gfx = crate::gfx::owner::gfx();
    release_all_textures(gfx);
}

/// `gfx::release_all_textures`: the drawing noted flushed, both pools emptied (also what
/// `gfx::set_screen_mode` does first, 0x404C55).
pub fn release_all_textures(gfx: &crate::gfx::owner::Gfx) {
    flush();
    let what = "the release text";
    let text = crate::files::cstring_at(VIEWER_PANEL.text_ptr(texts::SVM_RELEASE_TEXTURES), what)
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e));
    crate::logfile::write_line::log_text(&text, true, 2);
    release_pool(1, gfx);
    release_pool(0, gfx);
}

// ---- scene::load_texture (0x44E778) ----

/// `scene::load_texture` for Rust callers: video texture `video` holds system texture
/// `system`. Every register kept.
pub fn load_texture(system: u32, video: u32) {
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
            Some(level) => level.pixels.to_vec(),
            None => continue,
        };
        if let Some(Some(target)) = memory.get_mut(to) {
            crate::render::texture_written(target.pixels.ptr());
            let n = pixels.len().min(target.pixels.len());
            // SAFETY: this thread draws; the target is not locked while it is loaded.
            let to = unsafe { target.pixels.slice_mut() };
            to[..n].copy_from_slice(&pixels[..n]);
        }
    }
}

// ---- d3d::begin_scene (0x41ABC7), d3d::end_scene (0x41AB8A) ----

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
    let d3d = d3d();
    d3d.cards.in_scene.set(1u32);
    for (value, state) in BEGIN_STATES
        .into_iter()
        .chain([(d3d.cards.fog_state.get(), 0x1A)])
    {
        crate::d3d::set_render_state::set_render_state(value, state);
    }
    // 0x41ACDE: no texture is the current one, then none is set.
    crate::d3d::render_cache::set_texture(0xFFFF_FC44u32);
    crate::d3d::set_texture::set_texture(NO_TEXTURE);
}

/// `d3d::end_scene` for Rust callers: true when a scene was open and is ended (the
/// original then leaves eax 0 and the zero flag set; the carry always clear).
pub fn end_layer() -> bool {
    let d3d = d3d();
    flush();
    {
        if d3d.cards.in_scene.get() == 1 {
            d3d.cards.in_scene.set(0u32);
            return true;
        }
    }
    false
}

// ---- d3d::clear_viewport (0x44E9D9) ----

/// `d3d::clear_viewport` for Rust callers: the depth buffer cleared over the rectangle
/// `DRAW.cleared_rectangle` holds (the original leaves eax 0).
pub fn clear_viewport(own: &[crate::session::Dword; 4]) {
    let [left, top, right, bottom] = crate::session::read_bits(own).map(|v| v as i32);
    let c = |v: i32| v.max(0) as usize;
    note(|_| Command::Clear([c(left), c(top), c(right), c(bottom)]));
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
    }
}
