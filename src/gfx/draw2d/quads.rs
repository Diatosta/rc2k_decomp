//! Glyphs and sprites noted as rectangles of a texture atlas, for a GPU to draw as quads, in
//! place of pixels written to the screen's canvas.
//!
//! A glyph drawn on the screen while `wanted()` is decoded once (`rc2k_core::gfx::atlas`), given a
//! place in the atlas (keyed by the glyph table's `table_generation`, so a table that is rebuilt
//! is decoded again), and noted as the rectangle its clip leaves: where it goes on the screen and
//! where its texels are. A palette glyph's colours come from a palette state kept as a row of the
//! same atlas, so a palette that changes between frames (the pulsing font) is one more row.
//! The GPU takes the noted rectangles and the atlas texels still to send (`take`) at the next
//! flush and draws the rectangles under that flush's 3D, over what the canvas holds.
//!
//! The canvas path stays whole beside it. Drawing that cannot be a quad (a glyph the atlas does
//! not hold, a full atlas, a rectangle, a picture) goes to the canvas, which the GPU puts under
//! the quads; so that order stays exact, a canvas write first draws the quads noted so far onto
//! the canvas (`before_canvas_write`, hooked into the screen's pixels' writes): the canvas then
//! holds everything drawn up to that write, in order, and the quads everything after it.

use crate::gfx::blit_glyph::Scratch;
use crate::gfx::glyph_store::{self, Glyphs};
use crate::gfx::surface::{Clip, Surface};
use rc2k_core::gfx::atlas::{Decoded, UNWRITTEN_PIXEL, decode, placed};
use rc2k_core::gfx::blit::plan;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed};
use std::sync::{Arc, Mutex};

/// The atlas texture's width and height in texels.
pub const ATLAS_SIDE: u32 = 2048;
/// Palettes are kept as rows of 256 texels, eight to an atlas row, in the atlas' first rows.
pub const PALETTE_ATLAS_ROWS: u32 = 32;
const PALETTES_PER_ROW: u32 = 8;
const PALETTE_TEXELS: u32 = 256;
const MAX_PALETTES: usize = (PALETTE_ATLAS_ROWS * PALETTES_PER_ROW) as usize;
/// A palette's bytes: 256 dwords.
const PALETTE_BYTES: usize = 1024;

/// What the environment asks for.
struct Config {
    /// Quads are drawn (not with `RC2K_2D_QUADS=0`, nor while `RC2K_2D_CHECK` compares the
    /// canvas with a replay of the commands).
    quads: bool,
    /// `RC2K_QUAD_CHECK`: each frame is also drawn the canvas way and the pictures compared.
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    check: bool,
    /// `RC2K_QUAD_CHECK=2d`: both drawings leave out the 3D, so that only the 2D is compared
    /// (the GPU's drawing of the 3D differs from itself by a few pixels a frame).
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    check_2d_only: bool,
}

fn config() -> &'static Config {
    static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();
    CONFIG.get_or_init(|| {
        let set = |name| std::env::var_os(name).is_some();
        let asked = std::env::var("RC2K_2D_QUADS").map_or(true, |v| v != "0");
        Config {
            // macOS's Metal draws the canvas way until its backend draws quads.
            quads: cfg!(not(target_os = "macos")) && asked && !set("RC2K_2D_CHECK"),
            #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
            check: set("RC2K_QUAD_CHECK"),
            #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
            check_2d_only: std::env::var("RC2K_QUAD_CHECK").is_ok_and(|v| v == "2d"),
        }
    })
}

/// Whether `RC2K_QUAD_CHECK` is set: the GPU backend draws each frame twice and compares.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn check_wanted() -> bool {
    config().quads && config().check
}

/// Whether the check leaves the 3D out of both drawings (`RC2K_QUAD_CHECK=2d`).
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn check_2d_only() -> bool {
    check_wanted() && config().check_2d_only
}

/// Which texels of the glyph table's glyph the atlas holds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Key {
    table: u32,
    glyph: u32,
    generation: u64,
}

/// A glyph the atlas holds, or one it does not (decoding refused it: it stays with the blitter).
enum Slot {
    Refused,
    Held(Held),
}

struct Held {
    decoded: Arc<Decoded>,
    /// The first texel in the atlas; none when the glyph draws no pixel at all.
    at: Option<(u32, u32)>,
}

/// A block of texels to send to the atlas.
#[derive(Debug)]
pub struct Upload {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    /// Row by row, four bytes each, in a canvas pixel's byte order.
    pub texels: Vec<[u8; 4]>,
}

struct Shelf {
    y: u32,
    height: u32,
    next_x: u32,
}

/// A palette as the atlas knows it.
#[derive(Clone)]
struct PaletteState {
    bytes: Arc<Vec<u8>>,
    /// Its first texel in the atlas.
    origin: (u32, u32),
}

struct Atlas {
    glyphs: HashMap<Key, Slot>,
    shelves: Vec<Shelf>,
    /// Where the next shelf starts.
    next_shelf: u32,
    /// Each palette state held and its number; none for one the atlas refuses (it holds the
    /// overlay's "unwritten" pixel).
    palette_numbers: HashMap<Vec<u8>, Option<usize>>,
    /// The state each palette number holds.
    palette_owner: Vec<Option<Vec<u8>>>,
    /// The number the next new state takes: they go round, the oldest state's place first.
    next_palette: usize,
    /// The palette numbers a noted quad uses (since the quads were last taken): never given
    /// to another state.
    palette_in_use: Vec<bool>,
    /// The state each palette handle last had.
    by_handle: HashMap<u32, Option<PaletteState>>,
    uploads: Vec<Upload>,
}

/// The atlas has no room (for a glyph or a palette).
struct Full;

impl Atlas {
    fn new() -> Self {
        Self {
            glyphs: HashMap::new(),
            shelves: Vec::new(),
            next_shelf: PALETTE_ATLAS_ROWS,
            palette_numbers: HashMap::new(),
            palette_owner: vec![None; MAX_PALETTES],
            next_palette: 0,
            palette_in_use: vec![false; MAX_PALETTES],
            by_handle: HashMap::new(),
            uploads: Vec::new(),
        }
    }

    /// The quads noted so far are taken (or drawn on the canvas): their palettes are free again.
    fn quads_taken(&mut self) {
        self.palette_in_use.fill(false);
    }

    fn clear(&mut self) {
        *self = Atlas::new();
    }

    /// A place for `width` x `height` texels.
    fn place(&mut self, width: u32, height: u32) -> Result<(u32, u32), Full> {
        if width > ATLAS_SIDE || height > ATLAS_SIDE {
            return Err(Full);
        }
        for shelf in &mut self.shelves {
            if shelf.height >= height
                && shelf.height <= 2 * height
                && shelf.next_x + width <= ATLAS_SIDE
            {
                let at = (shelf.next_x, shelf.y);
                shelf.next_x += width;
                return Ok(at);
            }
        }
        if self.next_shelf + height > ATLAS_SIDE {
            return Err(Full);
        }
        self.shelves.push(Shelf {
            y: self.next_shelf,
            height,
            next_x: width,
        });
        let at = (0, self.next_shelf);
        self.next_shelf += height;
        Ok(at)
    }

    /// Palette `handle` as it is now: its bytes and place; none for one that cannot be read or
    /// that holds the overlay's "unwritten" pixel.
    fn palette(
        &mut self,
        fonts: &crate::gfx::owner::FontStores,
        handle: u32,
    ) -> Result<Option<PaletteState>, Full> {
        let Some(bytes) = read_palette(fonts, handle) else {
            return Ok(None);
        };
        if let Some(Some(known)) = self.by_handle.get(&handle)
            && *known.bytes == bytes
        {
            let state = known.clone();
            self.palette_in_use[palette_number(state.origin)] = true;
            return Ok(Some(state));
        }
        let number = match self.palette_numbers.get(&bytes) {
            Some(&number) => number,
            None => {
                let held_unwritten = bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|d| u32::from_le_bytes(*d) == UNWRITTEN_PIXEL);
                let number = if held_unwritten {
                    None
                } else {
                    // The next number no noted quad uses.
                    let number = (0..MAX_PALETTES)
                        .map(|n| (self.next_palette + n) % MAX_PALETTES)
                        .find(|&n| !self.palette_in_use[n])
                        .ok_or(Full)?;
                    self.next_palette = (number + 1) % MAX_PALETTES;
                    // The state that was there is forgotten.
                    if let Some(old) = self.palette_owner[number].take() {
                        self.palette_numbers.remove(&old);
                        self.by_handle.retain(|_, kept| {
                            kept.as_ref()
                                .is_none_or(|k| palette_number(k.origin) != number)
                        });
                    }
                    self.palette_owner[number] = Some(bytes.clone());
                    let (x, y) = palette_origin(number);
                    let mut texels = vec![[0u8; 4]; PALETTE_TEXELS as usize];
                    for (texel, entry) in texels.iter_mut().zip(bytes.as_chunks::<4>().0) {
                        *texel = *entry;
                    }
                    self.uploads.push(Upload {
                        x,
                        y,
                        width: PALETTE_TEXELS,
                        height: 1,
                        texels,
                    });
                    Some(number)
                };
                self.palette_numbers.insert(bytes.clone(), number);
                number
            }
        };
        if let Some(number) = number {
            self.palette_in_use[number] = true;
        }
        let state = number.map(|number| PaletteState {
            bytes: Arc::new(bytes),
            origin: palette_origin(number),
        });
        self.by_handle.insert(handle, state.clone());
        Ok(state)
    }

    /// The atlas' held glyph for `key`, decoded now from `record` if it is not there.
    fn glyph(&mut self, key: Key, record: &Glyphs) -> Result<&Slot, Full> {
        if !self.glyphs.contains_key(&key) {
            let slot = match decode(record) {
                None => Slot::Refused,
                Some(decoded) => {
                    let drawn = decoded.texels.iter().any(|t| t[3] != 0);
                    let at = if drawn {
                        let at = self.place(decoded.width as u32, decoded.height as u32)?;
                        self.uploads.push(Upload {
                            x: at.0,
                            y: at.1,
                            width: decoded.width as u32,
                            height: decoded.height as u32,
                            texels: decoded.texels.clone(),
                        });
                        Some(at)
                    } else {
                        None
                    };
                    Slot::Held(Held {
                        decoded: Arc::new(decoded),
                        at,
                    })
                }
            };
            self.glyphs.insert(key, slot);
        }
        Ok(&self.glyphs[&key])
    }
}

/// The number of the palette whose first texel is `origin`.
fn palette_number((x, y): (u32, u32)) -> usize {
    (y * PALETTES_PER_ROW + x / PALETTE_TEXELS) as usize
}

fn palette_origin(number: usize) -> (u32, u32) {
    (
        (number as u32 % PALETTES_PER_ROW) * PALETTE_TEXELS,
        number as u32 / PALETTES_PER_ROW,
    )
}

/// Palette `handle`'s bytes: up to 256 dwords, fewer where the palette holds fewer.
fn read_palette(fonts: &crate::gfx::owner::FontStores, handle: u32) -> Option<Vec<u8>> {
    glyph_store::read_bytes(fonts, handle, 0, PALETTE_BYTES, "a palette")
        .or_else(|_| {
            let held = glyph_store::bytes(fonts, handle, "a palette")?;
            Ok::<_, crate::errors::BadInput>(
                (0..PALETTE_BYTES)
                    .map_while(|at| rc2k_core::strings::bytes::Bytes::byte(&held, at))
                    .collect(),
            )
        })
        .ok()
        .filter(|bytes| !bytes.is_empty())
}

/// A rectangle noted for the GPU, and what it takes to draw it onto the canvas instead.
struct Deferred {
    rect: Rect,
    fill: Fill,
}

/// What a noted rectangle shows.
enum Fill {
    Glyph {
        decoded: Arc<Decoded>,
        /// The palette state the glyph was drawn with; none for a direct-colour glyph.
        palette: Option<Arc<Vec<u8>>>,
        /// The glyph's first visible texel.
        from: (i32, i32),
    },
    /// One pixel's four bytes over the whole rectangle.
    Solid([u8; 4]),
}

/// A rectangle of texels at whole screen pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    /// The screen's pixel of the first texel, and the size.
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    /// The atlas' first texel.
    pub u: u32,
    pub v: u32,
    /// The first texel of the palette row for palette glyphs; none for direct colour.
    pub palette: Option<(u32, u32)>,
    /// The pixel's bytes of a solid rectangle (its texels and palette are not used).
    pub solid: Option<[u8; 4]>,
}

struct State {
    atlas: Atlas,
    pending: Vec<Deferred>,
    /// The screen's width and height the pending rectangles were made for.
    size: (u32, u32),
    /// The atlas is full and rectangles are pending: it is cleared at their demotion.
    reset_wanted: bool,
}

static STATE: std::sync::LazyLock<Mutex<State>> = std::sync::LazyLock::new(|| {
    Mutex::new(State {
        atlas: Atlas::new(),
        pending: Vec::new(),
        size: (0, 0),
        reset_wanted: false,
    })
});

/// `STATE.pending.len()`, readable without the lock (the screen's every write asks).
static PENDING: AtomicUsize = AtomicUsize::new(0);

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Counts for the platform's timing line.
pub static QUADS: AtomicU64 = AtomicU64::new(0);
pub static DEMOTED: AtomicU64 = AtomicU64::new(0);
pub static REFUSED: AtomicU64 = AtomicU64::new(0);
pub static ATLAS_CLEARS: AtomicU64 = AtomicU64::new(0);
/// Glyphs left to the canvas, by why: the atlas does not hold the glyph, the glyph has no usable
/// palette, the atlas is full.
pub static REFUSED_BY: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];
pub static ATLAS_BYTES: AtomicU64 = AtomicU64::new(0);
/// Solid rectangles noted as quads.
pub static SOLIDS: AtomicU64 = AtomicU64::new(0);

/// Whether a glyph drawn now on the screen may be noted as a quad: they are on, a GPU backend
/// that draws them holds the frame, and it is not a race's frame.
pub fn wanted() -> bool {
    config().quads
        && crate::render::gpu_running()
        && crate::render::frame_on_gpu()
        && !crate::platform::direct3d::game_scratch()
            .is_some_and(|scratch| crate::scene::workers::in_race(scratch.game_scratch()))
}

/// One call of the glyph funnel.
#[derive(Clone, Copy)]
pub struct Call {
    pub glyph: u32,
    pub table: u32,
    pub x: i32,
    pub y: i32,
    /// The font's palette (a glyph store handle).
    pub palette: Option<u32>,
}

/// Notes the glyph `call` (its `record` read from the table) as a quad if the atlas can hold
/// it; the blitter's work variables as they are left when it is drawn. `None`: the canvas draws
/// it (the caller writes it as before; the write draws the quads before it first).
pub fn note_glyph(
    fonts: &crate::gfx::owner::FontStores,
    record: &Glyphs,
    call: Call,
    surface: &Surface,
    clip: &Clip,
    before: Scratch,
) -> Option<Scratch> {
    if !wanted() || surface.bits_per_pixel != 32 || surface.bytes_per_pixel != 4 {
        return None;
    }
    let pitch = u32::try_from(surface.pitch)
        .ok()
        .filter(|p| *p > 0 && p % 4 == 0)?;
    let len = crate::gfx::locked_pixels::bytes_of(crate::gfx::locked_pixels::SCREEN)?.len() as u32;
    let size = (pitch / 4, len / pitch);
    if !len.is_multiple_of(pitch)
        || clip.left < 0
        || clip.top < 0
        || clip.right >= size.0 as i32
        || clip.bottom >= size.1 as i32
    {
        return None;
    }
    let key = Key {
        table: call.table,
        glyph: call.glyph,
        generation: glyph_store::table_generation(fonts, call.table)?,
    };
    let mut guard = state();
    let state = &mut *guard;
    if !state.pending.is_empty() && state.size != size {
        return None;
    }
    state.size = size;
    if state.pending.is_empty() && state.reset_wanted {
        state.atlas.clear();
        state.reset_wanted = false;
    }
    let outcome = note(state, fonts, record, key, call, clip, before);
    match outcome {
        Ok(done) => {
            PENDING.store(state.pending.len(), Relaxed);
            if done.is_none() {
                REFUSED.fetch_add(1, Relaxed);
            }
            done
        }
        Err(Full) => {
            REFUSED.fetch_add(1, Relaxed);
            REFUSED_BY[2].fetch_add(1, Relaxed);

            if state.pending.is_empty() {
                // Nothing noted depends on the atlas: start it again, and try once.
                state.atlas.clear();
                ATLAS_CLEARS.fetch_add(1, Relaxed);
                let again = note(state, fonts, record, key, call, clip, before);
                PENDING.store(state.pending.len(), Relaxed);
                again.ok().flatten()
            } else {
                state.reset_wanted = true;
                None
            }
        }
    }
}

/// Notes solid rectangles (x, y, width, height, each inside the screen) of one pixel, in the
/// order of the glyph quads, so that what is drawn later goes over what is drawn before. `pitch`
/// is the surface's row length in bytes. False when the canvas must be drawn on instead: quads
/// are not wanted, the screen is not 32-bit rows of this pitch, a rectangle is outside it, or the
/// pixel is the overlay's "unwritten" mark.
pub fn note_solid(rectangles: &[[i32; 4]], pixel: [u8; 4], pitch: i32) -> bool {
    if !wanted() || u32::from_le_bytes(pixel) == UNWRITTEN_PIXEL {
        return false;
    }
    let Some(pitch) = u32::try_from(pitch).ok().filter(|p| *p > 0 && p % 4 == 0) else {
        return false;
    };
    let Some(len) =
        crate::gfx::locked_pixels::bytes_of(crate::gfx::locked_pixels::SCREEN).map(|b| b.len())
    else {
        return false;
    };
    let len = len as u32;
    let size = (pitch / 4, len / pitch);
    if !len.is_multiple_of(pitch)
        || rectangles.iter().any(|&[x, y, w, h]| {
            x < 0
                || y < 0
                || w <= 0
                || h <= 0
                || x.saturating_add(w) > size.0 as i32
                || y.saturating_add(h) > size.1 as i32
        })
    {
        return false;
    }
    let mut guard = state();
    let state = &mut *guard;
    if !state.pending.is_empty() && state.size != size {
        return false;
    }
    state.size = size;
    for &[x, y, width, height] in rectangles {
        state.pending.push(Deferred {
            rect: Rect {
                x,
                y,
                width,
                height,
                u: 0,
                v: 0,
                palette: None,
                solid: Some(pixel),
            },
            fill: Fill::Solid(pixel),
        });
    }
    SOLIDS.fetch_add(rectangles.len() as u64, Relaxed);
    PENDING.store(state.pending.len(), Relaxed);
    true
}

/// Notes the pixels at byte `offsets` of the screen (a plan's) as solid rectangles of `pixel`;
/// false when the canvas must be drawn on (see `note_solid`).
pub fn note_pixels(offsets: &[i32], pixel: [u8; 4], bytes_per_pixel: i32, pitch: i32) -> bool {
    if !wanted() || bytes_per_pixel != 4 || pitch <= 0 {
        return false;
    }
    let Some(len) =
        crate::gfx::locked_pixels::bytes_of(crate::gfx::locked_pixels::SCREEN).map(|b| b.len())
    else {
        return false;
    };
    let Some(rectangles) = rc2k_core::gfx::raster::pixel_rectangles(
        offsets,
        bytes_per_pixel,
        pitch,
        (len / pitch as usize) as i32,
    ) else {
        return false;
    };
    note_solid(&rectangles, pixel, pitch)
}

/// The noted quads dropped without being drawn: something drawn over the whole screen follows.
pub fn discard_pending() {
    let mut guard = state();
    let state = &mut *guard;
    PENDING.store(0, Relaxed);
    state.pending.clear();
    state.atlas.quads_taken();
    if state.reset_wanted {
        state.atlas.clear();
        state.reset_wanted = false;
        ATLAS_CLEARS.fetch_add(1, Relaxed);
    }
}

fn note(
    state: &mut State,
    fonts: &crate::gfx::owner::FontStores,
    record: &Glyphs,
    key: Key,
    call: Call,
    clip: &Clip,
    before: Scratch,
) -> Result<Option<Scratch>, Full> {
    let Slot::Held(held) = state.atlas.glyph(key, record)? else {
        REFUSED_BY[0].fetch_add(1, Relaxed);
        return Ok(None);
    };
    let (decoded, at) = (Arc::clone(&held.decoded), held.at);
    let core_clip = rc2k_core::gfx::raster::Clip {
        left: clip.left,
        right: clip.right,
        top: clip.top,
        bottom: clip.bottom,
    };
    let plan = plan(
        call.x,
        call.y,
        [
            decoded.x_offset,
            decoded.y_offset,
            decoded.width,
            decoded.height,
        ],
        &core_clip,
    );
    let mut work = rc2k_core::gfx::blit::Scratch {
        left_skip: before.left_skip,
        visible_w: before.visible_w,
        top_skip: before.top_skip,
        rows_left: before.rows_left,
    };
    plan.write_work(&mut work);
    if plan.drawn {
        work.rows_left = plan.rows_left_after();
    }
    let after = Scratch {
        left_skip: work.left_skip,
        visible_w: work.visible_w,
        top_skip: work.top_skip,
        rows_left: work.rows_left,
    };
    let (Some(rect), Some(at)) = (placed(&plan), at) else {
        // Nothing of it is inside the clip, or it draws no pixel.
        return Ok(Some(after));
    };
    let palette = if decoded.direct {
        None
    } else {
        let usable = call
            .palette
            .map(|handle| state.atlas.palette(fonts, handle))
            .transpose()?
            .flatten()
            .filter(|p| p.bytes.len() >= 4 * (usize::from(decoded.max_index) + 1));
        let Some(palette) = usable else {
            REFUSED_BY[1].fetch_add(1, Relaxed);
            return Ok(None);
        };
        Some(palette)
    };
    state.pending.push(Deferred {
        rect: Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            u: at.0 + rect.from_x as u32,
            v: at.1 + rect.from_y as u32,
            palette: palette.as_ref().map(|p| p.origin),
            solid: None,
        },
        fill: Fill::Glyph {
            decoded,
            palette: palette.map(|p| p.bytes),
            from: (rect.from_x, rect.from_y),
        },
    });
    Ok(Some(after))
}

/// A write to the canvas is about to happen: the quads noted so far are drawn onto the canvas
/// first, in order, so that the write goes over them as it did when the canvas was all there was.
pub fn before_canvas_write() {
    if PENDING.load(Relaxed) == 0 {
        return;
    }
    let (pending, size) = {
        let mut guard = state();
        let state = &mut *guard;
        PENDING.store(0, Relaxed);
        state.atlas.quads_taken();
        if state.reset_wanted {
            state.atlas.clear();
            state.reset_wanted = false;
            ATLAS_CLEARS.fetch_add(1, Relaxed);
        }
        (std::mem::take(&mut state.pending), state.size)
    };
    DEMOTED.fetch_add(pending.len() as u64, Relaxed);
    draw_on_canvas(&pending, size);
}

/// The quads noted so far drawn onto the canvas, and the atlas given up: a GPU backend has
/// started or stopped.
pub fn gpu_changed() {
    before_canvas_write();
    state().atlas.clear();
}

/// The noted quads written as pixels, in the order they were noted.
fn draw_on_canvas(pending: &[Deferred], (width, _): (u32, u32)) {
    let canvas = crate::gfx::locked_pixels::Place::start(crate::gfx::locked_pixels::SCREEN);
    for quad in pending {
        let Deferred { rect, fill } = quad;
        let (decoded, palette, from) = match fill {
            Fill::Solid(pixel) => {
                let row = pixel.repeat(rect.width as usize);
                for y in rect.y..rect.y + rect.height {
                    let at = (y as u32 * width + rect.x as u32) * 4;
                    canvas.must_write_bytes_at(at, &row, "the surface");
                }
                continue;
            }
            Fill::Glyph {
                decoded,
                palette,
                from,
            } => (decoded, palette, from),
        };
        for row in 0..rect.height {
            let texels = decoded
                .texels
                .iter()
                .skip(((from.1 + row) * decoded.width + from.0) as usize)
                .take(rect.width as usize);
            let mut run: Vec<u8> = Vec::new();
            let mut run_x = 0;
            let flush = |run: &mut Vec<u8>, run_x: i32| {
                if !run.is_empty() {
                    let at = ((rect.y + row) as u32 * width + (rect.x + run_x) as u32) * 4;
                    canvas.must_write_bytes_at(at, run, "the surface");
                    run.clear();
                }
            };
            for (column, texel) in texels.enumerate() {
                if texel[3] == 0 {
                    flush(&mut run, run_x);
                    continue;
                }
                if run.is_empty() {
                    run_x = column as i32;
                }
                match palette {
                    Some(colours) => {
                        let at = 4 * usize::from(texel[2]);
                        run.extend_from_slice(&colours[at..at + 4]);
                    }
                    // A direct colour's alpha byte is not kept by the atlas: the canvas is
                    // opaque.
                    None => run.extend_from_slice(&[texel[0], texel[1], texel[2], 0xFF]),
                }
            }
            flush(&mut run, run_x);
        }
    }
}

/// What the GPU takes at a flush: the rectangles, and the texels to send to the atlas.
#[derive(Default)]
pub struct Taken {
    pub rects: Vec<Rect>,
    pub uploads: Vec<Upload>,
    /// The screen's size the rectangles were noted for.
    pub size: (u32, u32),
}

impl Taken {
    /// The rectangles for a screen of `screen`'s size: dropped, with a note in the log, when
    /// they were noted for another size (the texels for the atlas stay).
    pub fn fitted_to(mut self, screen: (u32, u32)) -> Taken {
        if self.size != screen && !self.rects.is_empty() {
            crate::logfile::program_log::log(&format!(
                "render: {} quads noted for a screen of {} x {}, drawn on one of {} x {}: dropped",
                self.rects.len(),
                self.size.0,
                self.size.1,
                screen.0,
                screen.1
            ));
            self.rects.clear();
        }
        self
    }
}

/// The dwords of one vertex the quad shaders read: its place on the screen and its atlas texel
/// (four floats), and its palette's first texel (two floats).
pub const VERTEX_WORDS: usize = 6;

/// The two triangles of each rectangle, a vertex a record of `VERTEX_WORDS` dwords (float bits).
pub fn vertices(rects: &[Rect]) -> Vec<[u32; VERTEX_WORDS]> {
    let mut out = Vec::with_capacity(rects.len() * 6);
    for rect in rects {
        let (x0, y0) = (rect.x as f32, rect.y as f32);
        let (x1, y1) = ((rect.x + rect.width) as f32, (rect.y + rect.height) as f32);
        let (u0, v0) = (rect.u as f32, rect.v as f32);
        let (u1, v1) = (
            (rect.u as i32 + rect.width) as f32,
            (rect.v as i32 + rect.height) as f32,
        );
        // A solid rectangle's colour goes where a palette's place would: minus (2 + red * 256
        // + green) and blue, the colour's channels as the texture's BGRA format reads them.
        let (px, py) = match (rect.solid, rect.palette) {
            (Some([blue, green, red, _]), _) => (
                -(2.0 + (u32::from(red) * 256 + u32::from(green)) as f32),
                f32::from(blue),
            ),
            (None, Some((x, y))) => (x as f32, y as f32),
            (None, None) => (-1.0, -1.0),
        };
        let corner = |x, y, u, v| [x, y, u, v, px, py].map(f32::to_bits);
        out.extend([
            corner(x0, y0, u0, v0),
            corner(x1, y0, u1, v0),
            corner(x0, y1, u0, v1),
            corner(x1, y0, u1, v0),
            corner(x1, y1, u1, v1),
            corner(x0, y1, u0, v1),
        ]);
    }
    out
}

/// The rectangles noted and the texels waiting for the atlas, taken.
pub fn take() -> Taken {
    let mut guard = state();
    let state = &mut *guard;
    PENDING.store(0, Relaxed);
    let rects: Vec<Rect> = state.pending.drain(..).map(|d| d.rect).collect();
    QUADS.fetch_add(rects.len() as u64, Relaxed);
    state.atlas.quads_taken();
    let uploads = std::mem::take(&mut state.atlas.uploads);
    ATLAS_BYTES.fetch_add(
        uploads
            .iter()
            .map(|u| u.texels.len() as u64 * 4)
            .sum::<u64>(),
        Relaxed,
    );
    if state.reset_wanted {
        state.atlas.clear();
        state.reset_wanted = false;
        ATLAS_CLEARS.fetch_add(1, Relaxed);
    }
    Taken {
        rects,
        uploads,
        size: state.size,
    }
}

/// Compares the work variables the quad path left with the blitter's (in the check), logging a
/// difference.
pub fn check_scratch(quad: Scratch, blit: Scratch, call: Call) {
    static LOGGED: AtomicU64 = AtomicU64::new(0);
    if quad != blit && LOGGED.fetch_add(1, Relaxed) < 50 {
        crate::logfile::program_log::log(&format!(
            "2d quads: MISMATCH: the work variables differ: quad {quad:?}, blit {blit:?}, glyph {} of table {:#x} at {},{}",
            call.glyph, call.table, call.x, call.y
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_atlas_places_glyphs_on_shelves_and_runs_out() {
        let mut atlas = Atlas::new();
        let first = atlas.place(10, 12).ok().unwrap();
        assert_eq!(first, (0, PALETTE_ATLAS_ROWS));
        // The same shelf for a glyph of about the same height.
        assert_eq!(atlas.place(10, 10).ok().unwrap(), (10, PALETTE_ATLAS_ROWS));
        // A much shorter one starts a shelf of its own.
        assert_eq!(
            atlas.place(4, 3).ok().unwrap(),
            (0, PALETTE_ATLAS_ROWS + 12)
        );
        assert!(atlas.place(ATLAS_SIDE + 1, 1).is_err());
        let mut count = 0;
        while atlas.place(1000, 500).is_ok() {
            count += 1;
        }
        assert!((3..=8).contains(&count));
    }

    #[test]
    fn a_rectangle_is_two_triangles_with_its_texels_and_palette_on_each_vertex() {
        let glyph = Rect {
            x: 10,
            y: 20,
            width: 3,
            height: 2,
            u: 100,
            v: 40,
            palette: Some((256, 5)),
            solid: None,
        };
        let solid = Rect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            u: 0,
            v: 0,
            palette: None,
            // Blue 9, green 3, red 2: minus (2 + 2 * 256 + 3) in x, and the blue in y.
            solid: Some([9, 3, 2, 0xFF]),
        };
        let floats = |vertex: [u32; VERTEX_WORDS]| vertex.map(f32::from_bits);
        let out = vertices(&[glyph, solid]);
        assert_eq!(out.len(), 12);
        let corners: Vec<_> = out[..6].iter().map(|v| floats(*v)).collect();
        let (left, top, right, bottom) = (10.0, 20.0, 13.0, 22.0);
        let at = |x, y, u, v| [x, y, u, v, 256.0, 5.0];
        assert_eq!(
            corners,
            [
                at(left, top, 100.0, 40.0),
                at(right, top, 103.0, 40.0),
                at(left, bottom, 100.0, 42.0),
                at(right, top, 103.0, 40.0),
                at(right, bottom, 103.0, 42.0),
                at(left, bottom, 100.0, 42.0),
            ]
        );
        assert!(
            out[6..]
                .iter()
                .all(|v| floats(*v)[4..] == [-(2.0 + 515.0), 9.0])
        );
        assert!(vertices(&[]).is_empty());
    }

    #[test]
    fn rectangles_noted_for_another_screen_are_dropped_and_their_texels_stay() {
        let rect = Rect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            u: 0,
            v: 0,
            palette: None,
            solid: Some([0; 4]),
        };
        let taken = |size| Taken {
            rects: vec![rect],
            uploads: vec![Upload {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                texels: vec![[1, 2, 3, 4]],
            }],
            size,
        };
        let same = taken((800, 600)).fitted_to((800, 600));
        assert_eq!((same.rects.len(), same.uploads.len()), (1, 1));
        let other = taken((640, 480)).fitted_to((800, 600));
        assert_eq!((other.rects.len(), other.uploads.len()), (0, 1));
    }

    /// A glyph table of one glyph, a gap of one then a pixel of index `index`.
    fn one_glyph_table(index: u8) -> Vec<u8> {
        // The first offset is the glyph count times four; the record follows the offsets.
        let mut table = 4u32.to_le_bytes().to_vec();
        table.extend_from_slice(&[0, 0, 2, 1, 1, 1, index, 0]);
        table
    }

    fn texel_of(atlas: &mut Atlas, table: u32) -> [u8; 4] {
        let fonts = &crate::test_support::own::gfx().fonts;
        let record = glyph_store::bytes(fonts, table, "a table").unwrap();
        // The record of glyph 0 starts where the table's first offset says.
        let offset = rc2k_core::strings::bytes::Bytes::u32_le(&record, 0).unwrap();
        let Glyphs::Held(held, _) = record else {
            panic!("a held table")
        };
        let at = Glyphs::Held(held, offset as usize);
        let key = Key {
            table,
            glyph: 0,
            generation: glyph_store::table_generation(fonts, table).unwrap(),
        };
        let Ok(Slot::Held(found)) = atlas.glyph(key, &at) else {
            panic!("a decoded glyph")
        };
        found.decoded.texels[1]
    }

    /// A table given back and made again at the same place, or written to, is another state: the
    /// atlas decodes it afresh, never giving the glyph it held for the old one.
    #[test]
    fn a_glyph_table_rebuilt_is_decoded_again() {
        let fonts = &crate::test_support::own::gfx().fonts;
        let _lock = crate::test_support::test_lock();
        let list = crate::memory::ListCells::new(crate::memory::MemoryList {
            head: 0,
            total: 0,
            cursor: 0,
            remaining: 0,
        });
        let mut atlas = Atlas::new();
        let table = glyph_store::new_table(fonts, &list).unwrap();
        glyph_store::write_bytes(fonts, table, 0, &one_glyph_table(7), "a table").unwrap();
        let first = glyph_store::table_generation(fonts, table).unwrap();
        assert_eq!(texel_of(&mut atlas, table), [0, 0, 7, 255]);
        // Written to: another state.
        glyph_store::write_bytes(fonts, table, 0, &one_glyph_table(9), "a table").unwrap();
        let second = glyph_store::table_generation(fonts, table).unwrap();
        assert_ne!(first, second);
        assert_eq!(texel_of(&mut atlas, table), [0, 0, 9, 255]);
        // Given back and made again, at the same place with other bytes.
        glyph_store::give_back(fonts, glyph_store::key(&list));
        assert_eq!(glyph_store::table_generation(fonts, table), None);
        let again = glyph_store::new_table(fonts, &list).unwrap();
        assert_eq!(again, table);
        glyph_store::write_bytes(fonts, again, 0, &one_glyph_table(3), "a table").unwrap();
        let third = glyph_store::table_generation(fonts, again).unwrap();
        assert!(third != first && third != second);
        assert_eq!(texel_of(&mut atlas, again), [0, 0, 3, 255]);
        // The old states are still held under their own keys, so each stays right until the
        // atlas is cleared; none was handed out for the new state.
        assert_eq!(atlas.glyphs.len(), 3);
        glyph_store::give_back(fonts, glyph_store::key(&list));
    }

    /// Quads drawn on the canvas (what a canvas write does with the quads noted before it) are
    /// the blitter's pixels, for a palette glyph and a direct-colour one.
    #[test]
    fn quads_demoted_to_the_canvas_equal_the_blit() {
        use rc2k_core::gfx::raster::{Clip as CoreClip, Surface as CoreSurface};
        let _lock = crate::test_support::test_lock();
        let (width, height) = (20u32, 12u32);
        let canvas = Arc::new(crate::memory::pool::PoolBytes::from_vec(vec![
            0xAA;
            (width * height * 4)
                as usize
        ]));
        crate::gfx::locked_pixels::register(crate::gfx::locked_pixels::SCREEN, &canvas);
        let palette: Vec<u8> = (0..256u32)
            .flat_map(|n| (0x0100_0000 | n.wrapping_mul(0x0001_0203)).to_le_bytes())
            .collect();
        let colour = |n: u8| u32::from_le_bytes(palette[4 * n as usize..][..4].try_into().unwrap());
        // A palette glyph, then a direct-colour one of dwords with a gap in its first row.
        let indexed = [0u8, 0, 3, 2, 2, 1, 7, 8, 0, 3, 0, 1, 2, 3, 0];
        let mut direct = 0xFFFF_FFFFu32.to_le_bytes().to_vec();
        for n in [1u16, 1, 3, 2] {
            direct.extend_from_slice(&n.to_le_bytes());
        }
        for dword in [
            1u32,
            1,
            0x0033_2211,
            0,
            3,
            0,
            0x0000_0001,
            0x0000_0002,
            0x0066_5544,
            0,
        ] {
            direct.extend_from_slice(&dword.to_le_bytes());
        }
        let clip = CoreClip {
            left: 0,
            right: width as i32 - 1,
            top: 0,
            bottom: height as i32 - 1,
        };
        let surface = CoreSurface {
            bits: 32,
            bytes_per_pixel: 4,
            pitch: width as i32 * 4,
        };
        let mut expected = vec![0xAAu8; (width * height * 4) as usize];
        let mut noted = Vec::new();
        for (record, x, y, with_palette) in [(&indexed[..], 4, 3, true), (&direct[..], 9, 6, false)]
        {
            let mut scratch = rc2k_core::gfx::blit::Scratch::default();
            rc2k_core::gfx::blit::blit_glyph(
                record,
                x,
                y,
                &colour,
                &surface,
                &clip,
                &mut scratch,
                &mut |at, bytes| expected[at as usize..][..bytes.len()].copy_from_slice(bytes),
            )
            .unwrap();
            let decoded = decode(record).unwrap();
            let shape = [
                decoded.x_offset,
                decoded.y_offset,
                decoded.width,
                decoded.height,
            ];
            let rect = placed(&plan(x, y, shape, &clip)).unwrap();
            noted.push(Deferred {
                rect: Rect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                    u: 0,
                    v: 0,
                    palette: with_palette.then_some((0, 0)),
                    solid: None,
                },
                fill: Fill::Glyph {
                    decoded: Arc::new(decoded),
                    palette: with_palette.then(|| Arc::new(palette.clone())),
                    from: (rect.from_x, rect.from_y),
                },
            });
        }
        draw_on_canvas(&noted, (width, height));
        let mut drawn = canvas.to_vec();
        // The direct colour's alpha byte is the canvas' opaque one.
        for (place, wanted) in drawn
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(expected.as_chunks::<4>().0)
        {
            if *wanted != [0xAA; 4] && place[3] == 0xFF {
                place[3] = wanted[3];
            }
        }
        assert_eq!(drawn, expected);
    }

    /// A solid rectangle demoted to the canvas fills exactly its pixels with its four bytes.
    #[test]
    fn a_solid_rectangle_demoted_fills_its_pixels() {
        let _lock = crate::test_support::test_lock();
        let (width, height) = (6u32, 4u32);
        let canvas = Arc::new(crate::memory::pool::PoolBytes::from_vec(vec![
            0xAA;
            (width * height * 4)
                as usize
        ]));
        crate::gfx::locked_pixels::register(crate::gfx::locked_pixels::SCREEN, &canvas);
        let pixel = [1u8, 2, 3, 4];
        let noted = [Deferred {
            rect: Rect {
                x: 1,
                y: 1,
                width: 3,
                height: 2,
                u: 0,
                v: 0,
                palette: None,
                solid: Some(pixel),
            },
            fill: Fill::Solid(pixel),
        }];
        draw_on_canvas(&noted, (width, height));
        let drawn = canvas.to_vec();
        for y in 0..height {
            for x in 0..width {
                let at = ((y * width + x) * 4) as usize;
                let inside = (1..4).contains(&x) && (1..3).contains(&y);
                let wanted = if inside { pixel } else { [0xAA; 4] };
                assert_eq!(drawn[at..at + 4], wanted, "pixel {x},{y}");
            }
        }
    }

    #[test]
    fn palettes_fill_atlas_rows_eight_to_a_row() {
        assert_eq!(palette_origin(0), (0, 0));
        assert_eq!(palette_origin(9), (256, 1));
        assert_eq!(palette_origin(MAX_PALETTES - 1).1, PALETTE_ATLAS_ROWS - 1);
    }
}
