//! `gfx::blit_glyph` (0x4069B2): draws one run-length-encoded glyph on the global surface,
//! clipped to the global clip rectangle.
//!
//! In:  eax = glyph index, ecx = x, edx = y, ebx = palette (256 dwords), esi = glyph table
//! Out: esi, edi and ebp preserved. eax, ebx, ecx and edx hold leftovers of the drawing loops;
//!      the Rust version reproduces them, because the function has hundreds of callers.
//! No surface, a null table and a glyph index past the table are fatal errors in the original:
//! `glyph_record` returns them as errors.
//!
//! A glyph table starts with one dword offset per glyph (so the first offset / 4 is the glyph
//! count). A normal record is `[x offset][y offset][width][height]` in bytes, then per row
//! runs of `[count][gap][count pixels]`, a count of 0 ending the row. Its pixels are palette
//! indices: stored raw on an 8-bit surface, through the palette on 16-bit and 32-bit ones.
//! An extended record starts with a dword >= 0xFFFFFFFC, then the same four values as words;
//! its counts, gaps and pixels are words (marker 0xFFFFFFFE) or dwords (any other marker) and
//! the pixels are copied as they are, whatever the surface depth.

use crate::errors::{BadInput, FatalError, Refused};
use crate::gfx::blit::BLIT;
use crate::gfx::surface::{Clip, Surface, clip, surface};
use crate::pointers::{GameBytes, GamePtr};
use std::ptr::copy_nonoverlapping;

// ---- game data ----

/// The four work globals the blitter leaves behind. Each is only written once the clipping
/// has got that far, so a glyph that is clipped away leaves the later ones as they were.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Scratch {
    /// Pixels clipped off on the left.
    pub left_skip: i32,
    /// Pixels of each row inside the clip rectangle.
    pub visible_w: i32,
    /// Rows clipped off at the top.
    pub top_skip: i32,
    /// Visible rows. Counted down to 0 while drawing, unless nothing is clipped horizontally.
    pub rows_left: i32,
}

fn read_scratch() -> Scratch {
    Scratch {
        left_skip: BLIT.left_skip.get() as i32,
        visible_w: BLIT.visible_width.get() as i32,
        top_skip: BLIT.top_skip.get() as i32,
        rows_left: BLIT.rows_left.get() as i32,
    }
}

fn write_scratch(s: &Scratch) {
    BLIT.left_skip.set(s.left_skip as u32);
    BLIT.visible_width.set(s.visible_w as u32);
    BLIT.top_skip.set(s.top_skip as u32);
    BLIT.rows_left.set(s.rows_left as u32);
}

/// The three conditions the original reports as fatal errors, each with a null message.
pub const NO_SURFACE: FatalError = FatalError {
    code: 0x80A1_9D97,
    message: "",
};
pub const NO_GLYPH_TABLE: FatalError = FatalError {
    code: 0x80A7_DF31,
    message: "",
};
pub const GLYPH_OUT_OF_RANGE: FatalError = FatalError {
    code: 0x80A7_DF32,
    message: "",
};

/// Record number `glyph` of a glyph table (`add esi, [esi+eax*4]`), after the three checks
/// the original makes first.
pub fn glyph_record(surface: &Surface, table: GamePtr, glyph: u32) -> Result<GameBytes, Refused> {
    if surface.base == 0 {
        return Err(NO_SURFACE.into());
    }
    if table.addr() == 0 {
        return Err(NO_GLYPH_TABLE.into());
    }
    if glyph >= table.read::<u32>("a glyph table")? >> 2 {
        return Err(GLYPH_OUT_OF_RANGE.into());
    }
    let offset = table
        .offset(glyph.wrapping_mul(4))
        .read::<u32>("a glyph table's offset")?;
    Ok(table.offset(offset).bytes("a glyph")?)
}

// ---- Rust version ----
// Converted: `rc2k_core::gfx::blit`. Here the record is read as raw game memory and the core's
// offsets from the surface's start are made addresses again.

/// The four registers the original does not preserve, as it leaves them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Exit {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

/// The part of the surface a glyph may touch: `rows` spans of `row_bytes` bytes, the first at
/// address `first`, each `pitch` bytes after the one before.
#[derive(Clone, Copy, Debug)]
pub struct Area {}

/// Draws the glyph `record` at (`x`, `y`). `put(address, bytes)` receives every write.
/// Returns the registers the original leaves and, unless the glyph is clipped away, the part
/// of the surface it may have touched.
#[allow(clippy::too_many_arguments)] // the original's five inputs, the surface, the clip and the sink
pub unsafe fn blit_glyph(
    record: &GameBytes,
    x: i32,
    y: i32,
    palette: GamePtr,
    surface: &Surface,
    clip: &Clip,
    scratch: &mut Scratch,
    put: &mut dyn FnMut(u32, &[u8]),
) -> Result<(Exit, Option<Area>), Refused> {
    use rc2k_core::gfx::blit as core;
    use rc2k_core::strings::bytes::Bytes as _;
    let base = surface.base;
    let mut work = core::Scratch {
        left_skip: scratch.left_skip,
        visible_w: scratch.visible_w,
        top_skip: scratch.top_skip,
        rows_left: scratch.rows_left,
    };
    // The palette read inside its allocation, and only when a colour is looked up (some
    // callers pass a leftover value for glyphs that never need one, as the original never
    // reads it then); an entry outside answers 0 here and the blit is refused afterwards
    // (the core's colour has no failure).
    let colours = std::cell::OnceCell::new();
    let past_palette = std::cell::Cell::new(false);
    let colour = |n: u8| {
        let at = 4 * n as usize;
        let Ok(colours) = colours.get_or_init(|| palette.bytes("a palette")) else {
            past_palette.set(true);
            return 0;
        };
        match (0..4)
            .map(|k| colours.byte(at + k))
            .collect::<Option<Vec<u8>>>()
        {
            Some(b) => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            None => {
                past_palette.set(true);
                0
            }
        }
    };
    let blitted = core::blit_glyph(
        record,
        x,
        y,
        &colour,
        &rc2k_core::gfx::raster::Surface {
            bits: surface.bits_per_pixel,
            bytes_per_pixel: surface.bytes_per_pixel,
            pitch: surface.pitch,
        },
        &rc2k_core::gfx::raster::Clip {
            left: clip.left,
            right: clip.right,
            top: clip.top,
            bottom: clip.bottom,
        },
        &mut work,
        &mut |offset, bytes| put(base.wrapping_add(offset), bytes),
    );
    let Ok((exit, area)) = blitted else {
        return Err(BadInput("a read past a glyph's record").into());
    };
    if past_palette.get() {
        return Err(BadInput("a colour past a palette's allocation").into());
    }
    *scratch = Scratch {
        left_skip: work.left_skip,
        visible_w: work.visible_w,
        top_skip: work.top_skip,
        rows_left: work.rows_left,
    };
    let exit = Exit {
        eax: exit.eax,
        ebx: match exit.ebx {
            core::Ebx::Value(v) => v,
            core::Ebx::Row(offset) => base.wrapping_add(offset),
        },
        ecx: exit.ecx,
        edx: exit.edx,
    };
    let area = area.map(|_a| Area {});
    Ok((exit, area))
}

/// Draws glyph `glyph` of `table` on the game's surface with the game's clip rectangle, and
/// updates the blitter's work globals: what a call of the original does.
pub unsafe fn blit_glyph_on_surface(
    glyph: u32,
    table: GamePtr,
    x: i32,
    y: i32,
    palette: GamePtr,
) -> Result<Exit, Refused> {
    let surface = surface();
    let record = glyph_record(&surface, table, glyph)?;
    let mut scratch = read_scratch();
    let (exit, _) = unsafe {
        blit_glyph(
            &record,
            x,
            y,
            palette,
            &surface,
            &clip(),
            &mut scratch,
            &mut |addr, bytes| copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, bytes.len()),
        )
    }?;
    write_scratch(&scratch);
    Ok(exit)
}

// ---- a call, as the canvases pass it on ----

/// One call of `blit_glyph`, as its caller made it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlitCall {
    pub glyph: u32,
    pub x: i32,
    pub y: i32,
    pub palette: u32,
    pub table: u32,
    /// `g_clipRight` at the time of the call: one caller narrows it around a blit.
    pub clip_right: i32,
}

impl BlitCall {
    /// A call made now, with the clip rectangle as the game has it at this moment.
    pub fn new(glyph: u32, x: i32, y: i32, palette: u32, table: u32) -> Self {
        Self {
            glyph,
            x,
            y,
            palette,
            table,
            clip_right: clip().right,
        }
    }
}
