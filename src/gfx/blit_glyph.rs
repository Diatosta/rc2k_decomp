//! `gfx::blit_glyph` (0x4069B2): draws one run-length-encoded glyph on the global surface,
//! clipped to the global clip rectangle.
//!
//! In:  eax = glyph index, ecx = x, edx = y, ebx = palette (256 dwords), esi = glyph table
//! Out: esi, edi and ebp preserved. eax, ebx, ecx and edx hold leftovers of the drawing loops;
//!      the Rust version reproduces them, because the function has hundreds of callers.
//! No surface, a null table and a glyph index past the table are fatal errors in the original:
//! `glyph_record` returns them as errors and the hook leaves them to the original.
//!
//! A glyph table starts with one dword offset per glyph (so the first offset / 4 is the glyph
//! count). A normal record is `[x offset][y offset][width][height]` in bytes, then per row
//! runs of `[count][gap][count pixels]`, a count of 0 ending the row. Its pixels are palette
//! indices: stored raw on an 8-bit surface, through the palette on 16-bit and 32-bit ones.
//! An extended record starts with a dword >= 0xFFFFFFFC, then the same four values as words;
//! its counts, gaps and pixels are words (marker 0xFFFFFFFE) or dwords (any other marker) and
//! the pixels are copied as they are, whatever the surface depth.

use crate::errors::FatalError;
use crate::framework::{
    EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
    shadow,
};
use crate::gfx::surface::{
    BASE_RVA, BITS_PER_PIXEL_RVA, BYTES_PER_PIXEL_RVA, CLIP_BOTTOM_RVA, CLIP_LEFT_RVA,
    CLIP_RIGHT_RVA, CLIP_TOP_RVA, Clip, PITCH_RVA, Surface, clip, surface,
};
use std::cell::RefCell;
use std::ptr::copy_nonoverlapping;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, VirtualAlloc,
};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::blit_glyph", rva(0x4069B2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const LEFT_SKIP_RVA: usize = rva(0x4F504C); // g_blitLeftSkip
const VISIBLE_W_RVA: usize = rva(0x4F5050); // g_blitVisibleWidth
const TOP_SKIP_RVA: usize = rva(0x4F5054); // g_blitTopSkip
const ROWS_LEFT_RVA: usize = rva(0x4F5058); // g_blitRowsLeft

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
    unsafe {
        Scratch {
            left_skip: game_read(LEFT_SKIP_RVA),
            visible_w: game_read(VISIBLE_W_RVA),
            top_skip: game_read(TOP_SKIP_RVA),
            rows_left: game_read(ROWS_LEFT_RVA),
        }
    }
}

fn write_scratch(s: &Scratch) {
    unsafe {
        game_write(LEFT_SKIP_RVA, s.left_skip);
        game_write(VISIBLE_W_RVA, s.visible_w);
        game_write(TOP_SKIP_RVA, s.top_skip);
        game_write(ROWS_LEFT_RVA, s.rows_left);
    }
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
pub unsafe fn glyph_record(
    surface: &Surface,
    table: *const u8,
    glyph: u32,
) -> Result<*const u8, FatalError> {
    if surface.base == 0 {
        return Err(NO_SURFACE);
    }
    if table.is_null() {
        return Err(NO_GLYPH_TABLE);
    }
    unsafe {
        if glyph >= table.cast::<u32>().read_unaligned() >> 2 {
            return Err(GLYPH_OUT_OF_RANGE);
        }
        let offset = table
            .wrapping_add((glyph as usize).wrapping_mul(4))
            .cast::<u32>()
            .read_unaligned();
        Ok(table.wrapping_add(offset as usize))
    }
}

// ---- Rust version ----

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
pub struct Area {
    pub first: u32,
    pub pitch: i32,
    pub rows: u32,
    pub row_bytes: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    /// Byte runs copied raw, one byte per pixel.
    Raw8,
    /// Byte runs, each pixel the low word of its palette entry.
    Palette16,
    /// Byte runs, each pixel its palette entry.
    Palette32,
    /// Word runs copied raw.
    Direct16,
    /// Dword runs copied raw.
    Direct32,
}

impl Format {
    /// Size in bytes of a count, a gap and a source pixel.
    fn src_unit(self) -> u32 {
        match self {
            Format::Raw8 | Format::Palette16 | Format::Palette32 => 1,
            Format::Direct16 => 2,
            Format::Direct32 => 4,
        }
    }
    /// Size in bytes of a pixel as it is written.
    fn dst_unit(self) -> u32 {
        match self {
            Format::Raw8 => 1,
            Format::Palette16 | Format::Direct16 => 2,
            Format::Palette32 | Format::Direct32 => 4,
        }
    }
    fn is_direct(self) -> bool {
        matches!(self, Format::Direct16 | Format::Direct32)
    }
}

/// Reader over a record's run data.
struct Runs<'a> {
    format: Format,
    src: *const u8,
    palette: *const u32,
    put: &'a mut dyn FnMut(u32, &[u8]),
}

impl Runs<'_> {
    /// Reads a count or a gap and steps past it.
    unsafe fn field(&mut self) -> u32 {
        let unit = self.format.src_unit();
        let v = unsafe {
            match unit {
                1 => *self.src as u32,
                2 => self.src.cast::<u16>().read_unaligned() as u32,
                _ => self.src.cast::<u32>().read_unaligned(),
            }
        };
        self.src = self.src.wrapping_add(unit as usize);
        v
    }

    /// Steps over `pixels` source pixels (backwards if negative).
    fn skip_pixels(&mut self, pixels: i32) {
        let bytes = (pixels as isize).wrapping_mul(self.format.src_unit() as isize);
        self.src = self.src.wrapping_offset(bytes);
    }

    /// Writes the next `count` source pixels at address `dst` and steps past them.
    unsafe fn draw(&mut self, count: u32, dst: u32) {
        if count == 0 {
            return;
        }
        let n = count as usize;
        unsafe {
            match self.format {
                Format::Raw8 | Format::Direct16 | Format::Direct32 => {
                    let bytes = n * self.format.src_unit() as usize;
                    (self.put)(dst, from_raw_parts(self.src, bytes));
                    self.src = self.src.wrapping_add(bytes);
                }
                Format::Palette16 | Format::Palette32 => {
                    let d = self.format.dst_unit() as usize;
                    let mut out = vec![0u8; n * d];
                    for i in 0..n {
                        let colour = self.palette.add(*self.src.add(i) as usize).read_unaligned();
                        out[i * d..(i + 1) * d].copy_from_slice(&colour.to_le_bytes()[..d]);
                    }
                    (self.put)(dst, &out);
                    self.src = self.src.wrapping_add(n);
                }
            }
        }
    }
}

/// `a - b` and whether a `jle` after that `sub` is taken.
fn sub_le(a: i32, b: i32) -> (i32, bool) {
    (a.wrapping_sub(b), a <= b)
}

/// `a + b` and whether a `jle` after that `add` is taken.
fn add_le(a: i32, b: i32) -> (i32, bool) {
    (a.wrapping_add(b), a as i64 + b as i64 <= 0)
}

/// Where the walk over a clipped row stands. The original jumps between these points with
/// the flags of its last subtraction or addition still set; `bool` carries that `jle`.
enum Step {
    /// Still left of the clip edge, `left` pixels to go.
    Skip,
    /// Inside the clip rectangle with `left` pixels of room.
    Next,
    AfterGap(bool),
    AfterCount(bool),
    /// Past the right clip edge: only stepping over the rest of the row.
    Rest,
}

/// Draws the glyph `record` at (`x`, `y`). `put(address, bytes)` receives every write.
/// Returns the registers the original leaves and, unless the glyph is clipped away, the part
/// of the surface it may have touched.
pub unsafe fn blit_glyph(
    record: *const u8,
    x: i32,
    y: i32,
    palette: *const u32,
    surface: &Surface,
    clip: &Clip,
    scratch: &mut Scratch,
    put: &mut dyn FnMut(u32, &[u8]),
) -> (Exit, Option<Area>) {
    unsafe {
        let marker = record.cast::<u32>().read_unaligned();
        let word = |at: usize| record.add(at).cast::<u16>().read_unaligned() as i32;
        let (x_offset, y_offset, width, height, data, format) = if marker < 0xFFFF_FFFC {
            let format = match surface.bits_per_pixel {
                16 => Format::Palette16,
                32 => Format::Palette32,
                _ => Format::Raw8,
            };
            (
                *record as i32,
                *record.add(1) as i32,
                *record.add(2) as i32,
                *record.add(3) as i32,
                record.add(4),
                format,
            )
        } else {
            let format = if marker == 0xFFFF_FFFE {
                Format::Direct16
            } else {
                Format::Direct32
            };
            (word(4), word(6), word(8), word(10), record.add(12), format)
        };
        let x = x.wrapping_add(x_offset);
        let y = y.wrapping_add(y_offset);
        let clipped_away = |eax: i32, ebx: i32| {
            let exit = Exit {
                eax: eax as u32,
                ebx: ebx as u32,
                ecx: x as u32,
                edx: y as u32,
            };
            (exit, None)
        };

        // Horizontal clip.
        scratch.left_skip = 0;
        if x < clip.left {
            scratch.left_skip = clip.left.wrapping_sub(x);
        }
        let last = x.wrapping_add(width).wrapping_sub(1);
        let over = last.wrapping_sub(clip.right);
        let mut visible = width;
        if last > clip.right {
            visible = visible.wrapping_sub(over);
        }
        if visible <= scratch.left_skip {
            return clipped_away(visible.wrapping_sub(scratch.left_skip), over);
        }
        visible = visible.wrapping_sub(scratch.left_skip);
        scratch.visible_w = visible;

        // Vertical clip.
        scratch.top_skip = 0;
        if y < clip.top {
            scratch.top_skip = clip.top.wrapping_sub(y);
        }
        let last = y.wrapping_add(height).wrapping_sub(1);
        let over = last.wrapping_sub(clip.bottom);
        let mut rows = height;
        if last > clip.bottom {
            rows = rows.wrapping_sub(over);
        }
        if rows <= scratch.top_skip {
            return clipped_away(rows.wrapping_sub(scratch.top_skip), over);
        }
        rows = rows.wrapping_sub(scratch.top_skip);
        scratch.rows_left = rows;

        // Address of the glyph's column 0 in the first visible row. The x term uses the
        // surface's bytes per pixel, while the runs below advance by the format's own unit.
        let d = format.dst_unit();
        let pitch = surface.pitch;
        let mut row = (y
            .wrapping_add(scratch.top_skip)
            .wrapping_mul(pitch)
            .wrapping_add(x.wrapping_mul(surface.bytes_per_pixel)) as u32)
            .wrapping_add(surface.base);
        let area = Area {
            first: row.wrapping_add((scratch.left_skip as u32).wrapping_mul(d)),
            pitch,
            rows: rows as u32,
            row_bytes: (visible as u32).wrapping_mul(d),
        };
        let done = |eax: u32, ebx: u32, edx: u32| {
            let exit = Exit {
                eax,
                ebx,
                ecx: 0,
                edx,
            };
            (exit, Some(area))
        };

        let mut runs = Runs {
            format,
            src: data,
            palette,
            put,
        };

        // Step over the rows clipped off at the top.
        let mut skip = scratch.top_skip;
        if skip != 0 {
            loop {
                let count = runs.field();
                if count == 0 {
                    skip -= 1;
                    if skip == 0 {
                        break;
                    }
                } else {
                    runs.field();
                    runs.skip_pixels(count as i32);
                }
            }
        }

        // The gap of the last run read: what the original leaves in eax.
        let mut gap = 0u32;

        if visible == width {
            // Nothing clipped horizontally: every run is drawn whole.
            let mut rows_left = rows;
            let mut dst = row;
            loop {
                let count = runs.field();
                if count == 0 {
                    rows_left -= 1;
                    if rows_left == 0 {
                        return done(gap, row, 0);
                    }
                    row = row.wrapping_add(pitch as u32);
                    dst = row;
                    continue;
                }
                gap = runs.field();
                dst = dst.wrapping_add(gap.wrapping_mul(d));
                runs.draw(count, dst);
                dst = dst.wrapping_add(count.wrapping_mul(d));
            }
        }

        loop {
            // One clipped row. `left` is the original's edx: pixels still to skip while left
            // of the clip edge, then pixels of room inside it.
            let mut dst = row;
            let mut left = scratch.left_skip;
            let mut count = 0u32;
            let mut step = if left != 0 {
                Step::Skip
            } else {
                left = scratch.visible_w;
                Step::Next
            };
            loop {
                match step {
                    Step::Skip => {
                        count = runs.field();
                        if count == 0 {
                            break;
                        }
                        gap = runs.field();
                        dst = dst.wrapping_add(gap.wrapping_mul(d));
                        let (rest, le) = sub_le(left, gap as i32);
                        left = rest;
                        if le {
                            // The gap reached the clip edge.
                            let (room, le) = add_le(left, scratch.visible_w);
                            left = room;
                            step = Step::AfterGap(le);
                            continue;
                        }
                        runs.skip_pixels(count as i32);
                        dst = dst.wrapping_add(count.wrapping_mul(d));
                        let (rest, le) = sub_le(left, count as i32);
                        left = rest;
                        if !le {
                            continue; // the whole run is left of the clip edge
                        }
                        // The run crosses the clip edge: go back to its first visible pixel.
                        count = left.wrapping_neg() as u32;
                        runs.skip_pixels(left);
                        dst = dst.wrapping_add((left as u32).wrapping_mul(d));
                        let (room, le) = add_le(left, scratch.visible_w);
                        left = room;
                        step = Step::AfterCount(le);
                    }
                    Step::Next => {
                        count = runs.field();
                        if count == 0 {
                            break;
                        }
                        gap = runs.field();
                        dst = dst.wrapping_add(gap.wrapping_mul(d));
                        let (room, le) = sub_le(left, gap as i32);
                        left = room;
                        step = Step::AfterGap(le);
                    }
                    Step::AfterGap(no_room) => {
                        if no_room {
                            runs.skip_pixels(count as i32);
                            step = Step::Rest;
                        } else {
                            let (room, le) = sub_le(left, count as i32);
                            left = room;
                            step = Step::AfterCount(le);
                        }
                    }
                    Step::AfterCount(last_run) => {
                        if last_run {
                            // Only the part up to the right clip edge.
                            count = count.wrapping_add(left as u32);
                            runs.draw(count, dst);
                            runs.skip_pixels(left.wrapping_neg());
                            if format.is_direct() {
                                left = left.wrapping_neg(); // these two paths `neg edx`
                            }
                            step = Step::Rest;
                        } else {
                            runs.draw(count, dst);
                            dst = dst.wrapping_add(count.wrapping_mul(d));
                            step = Step::Next;
                        }
                    }
                    Step::Rest => {
                        count = runs.field();
                        if count == 0 {
                            break;
                        }
                        runs.field();
                        runs.skip_pixels(count as i32);
                    }
                }
            }
            scratch.rows_left = scratch.rows_left.wrapping_sub(1);
            if scratch.rows_left == 0 {
                return done(gap, row, left as u32);
            }
            row = row.wrapping_add(pitch as u32);
        }
    }
}

/// Draws glyph `glyph` of `table` on the game's surface with the game's clip rectangle, and
/// updates the blitter's work globals: what a call of the original does. Inside a swallowing
/// `record_calls` it only reports the call.
pub unsafe fn blit_glyph_on_surface(
    glyph: u32,
    table: *const u8,
    x: i32,
    y: i32,
    palette: *const u32,
) -> Result<Exit, FatalError> {
    // Rust callers do not pass through the hook, so they report to the recording here. Without
    // this a swallowing self-test would really draw whenever a hooked caller runs its Rust
    // version (shadow off).
    let swallowed = note_call(BlitCall {
        glyph,
        x,
        y,
        palette: palette as u32,
        table: table as u32,
    });
    if swallowed {
        return Ok(Exit {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
        });
    }
    let surface = surface();
    let record = unsafe { glyph_record(&surface, table, glyph) }?;
    let mut scratch = read_scratch();
    let (exit, _) = unsafe {
        blit_glyph(
            record,
            x,
            y,
            palette,
            &surface,
            &clip(),
            &mut scratch,
            &mut |addr, bytes| copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, bytes.len()),
        )
    };
    write_scratch(&scratch);
    Ok(exit)
}

// ---- call recorder ----
//
// Callers of blit_glyph (draw_text and others) are checked by comparing the calls the
// original makes with the calls the Rust version makes. The hook is where every call of the
// original passes, so it can list them.

/// One call of `blit_glyph`, as its caller made it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlitCall {
    pub glyph: u32,
    pub x: i32,
    pub y: i32,
    pub palette: u32,
    pub table: u32,
}

struct Recorder {
    calls: Vec<BlitCall>,
    swallow: bool,
}

thread_local! {
    static RECORDER: RefCell<Option<Recorder>> = const { RefCell::new(None) };
}

/// Runs `f` and returns the calls of `blit_glyph` that reached the hook on this thread
/// meanwhile. With `swallow`, those calls are only listed: nothing is drawn and the original
/// is not run, so a test can run original code that draws without touching the surface.
/// Other threads are not affected. Calls made by original code are seen only while the hook is
/// enabled (`recording_works`); calls made by Rust code through `blit_glyph_on_surface` always.
pub fn record_calls(swallow: bool, f: impl FnOnce()) -> Vec<BlitCall> {
    let outer = RECORDER.with(|r| r.borrow_mut().take());
    // Inside a swallowing recording everything is swallowed.
    let swallow = swallow || outer.as_ref().is_some_and(|o| o.swallow);
    RECORDER.with(|r| {
        *r.borrow_mut() = Some(Recorder {
            calls: Vec::new(),
            swallow,
        })
    });
    f();
    let mine = RECORDER
        .with(|r| r.borrow_mut().take())
        .map(|r| r.calls)
        .unwrap_or_default();
    // An enclosing recording sees these calls too.
    if let Some(mut outer) = outer {
        outer.calls.extend_from_slice(&mine);
        RECORDER.with(|r| *r.borrow_mut() = Some(outer));
    }
    mine
}

/// Tells this thread's recording, if there is one, about a call. True if the call is to be
/// swallowed: the caller must then draw nothing.
fn note_call(call: BlitCall) -> bool {
    RECORDER.with(|rec| {
        rec.borrow_mut().as_mut().is_some_and(|rec| {
            rec.calls.push(call);
            rec.swallow
        })
    })
}

/// False if the hook is switched off, in which case `record_calls` sees nothing.
pub fn recording_works() -> bool {
    HOOK.is_enabled()
}

// ---- hook ----

impl Area {
    fn row_start(&self, row: u32) -> u32 {
        self.first
            .wrapping_add((row as i32).wrapping_mul(self.pitch) as u32)
    }

    /// The bytes of all spans, row after row.
    unsafe fn read(&self) -> Vec<u8> {
        let mut image = Vec::with_capacity((self.rows * self.row_bytes) as usize);
        for row in 0..self.rows {
            let span = self.row_start(row) as *const u8;
            image.extend_from_slice(unsafe { from_raw_parts(span, self.row_bytes as usize) });
        }
        image
    }

    /// Applies one write to an image made by `read`. False if it is not inside one span.
    fn apply(&self, image: &mut [u8], addr: u32, bytes: &[u8]) -> bool {
        for row in 0..self.rows {
            let at = addr.wrapping_sub(self.row_start(row)) as usize;
            if at + bytes.len() <= self.row_bytes as usize {
                let start = (row * self.row_bytes) as usize + at;
                image[start..start + bytes.len()].copy_from_slice(bytes);
                return true;
            }
        }
        false
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let surface = surface();
    let (glyph, table) = (r.eax, r.esi as *const u8);

    let swallowed = note_call(BlitCall {
        glyph,
        x: r.ecx as i32,
        y: r.edx as i32,
        palette: r.ebx,
        table: r.esi,
    });
    if swallowed {
        return;
    }

    let Ok(record) = (unsafe { glyph_record(&surface, table, glyph) }) else {
        unsafe { call_original(r) };
        return;
    };
    let (x, y, palette) = (r.ecx as i32, r.edx as i32, r.ebx as *const u32);
    let clip = clip();
    let mut scratch = read_scratch();

    if !shadow() {
        let (exit, _) = unsafe {
            blit_glyph(
                record,
                x,
                y,
                palette,
                &surface,
                &clip,
                &mut scratch,
                &mut |addr, bytes| {
                    copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, bytes.len())
                },
            )
        };
        write_scratch(&scratch);
        (r.eax, r.ebx, r.ecx, r.edx) = (exit.eax, exit.ebx, exit.ecx, exit.edx);
        return;
    }

    // Shadow: Rust only records its writes; the original draws on the real surface. The
    // visible spans are read before and after, and must end up as Rust's writes make them.
    let mut writes: Vec<(u32, Vec<u8>)> = Vec::new();
    let (exit, area) = unsafe {
        blit_glyph(
            record,
            x,
            y,
            palette,
            &surface,
            &clip,
            &mut scratch,
            &mut |addr, bytes| writes.push((addr, bytes.to_vec())),
        )
    };
    let mut expected = area.map(|a| unsafe { a.read() });
    let mut stray = false;
    for (addr, bytes) in &writes {
        stray |= match (&area, &mut expected) {
            (Some(a), Some(image)) => !a.apply(image, *addr, bytes),
            _ => true,
        };
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };

    let pixels_ok = match (&area, &expected) {
        (Some(a), Some(image)) => *image == unsafe { a.read() },
        _ => true,
    };
    let original_exit = Exit {
        eax: o.eax,
        ebx: o.ebx,
        ecx: o.ecx,
        edx: o.edx,
    };
    let original_scratch = read_scratch();
    if stray || !pixels_ok || original_exit != exit || original_scratch != scratch {
        HOOK.mismatch(|| {
            format!(
                "glyph={glyph} x={x} y={y} bits={}: pixels_ok={pixels_ok} stray_write={stray} original {original_exit:x?} {original_scratch:?} rust {exit:x?} {scratch:?}",
                surface.bits_per_pixel
            )
        });
    }
    (r.eax, r.ebx, r.ecx, r.edx) = (o.eax, o.ebx, o.ecx, o.edx);
}

// ---- self-test ----
//
// The original reads the surface and clip globals, which the game's own drawing uses all the
// time, so they cannot be pointed at a test surface. Instead the test runs a private copy of
// the original's code in which every reference to those globals is redirected to
// `TEST_GLOBALS`. The copy never touches game state.

const CODE_START: usize = 0x4069B2;
const CODE_LEN: usize = 0x407062 - CODE_START;

/// Order of `TEST_GLOBALS`.
const GLOBAL_RVAS: [usize; 12] = [
    BITS_PER_PIXEL_RVA,
    BYTES_PER_PIXEL_RVA,
    PITCH_RVA,
    BASE_RVA,
    CLIP_LEFT_RVA,
    CLIP_RIGHT_RVA,
    CLIP_TOP_RVA,
    CLIP_BOTTOM_RVA,
    LEFT_SKIP_RVA,
    VISIBLE_W_RVA,
    TOP_SKIP_RVA,
    ROWS_LEFT_RVA,
];
static TEST_GLOBALS: [AtomicU32; 12] = [const { AtomicU32::new(0) }; 12];

mod patched {
    #![allow(dead_code)]
    use crate::framework::{Regs, hook_asm};
    use std::sync::atomic::AtomicUsize;

    /// Address of the patched copy of the original; 0 until it is built.
    pub static COPY: AtomicUsize = AtomicUsize::new(0);

    extern "C" fn unused(_: &mut Regs) {}

    // Only `call_copy` is used: it runs the code at COPY with a full register set.
    hook_asm!(unused_thunk, call_copy, dispatch = unused, original = COPY);
}

/// Builds the patched copy once. Err says why it could not be built.
unsafe fn build_copy() -> Result<(), &'static str> {
    if patched::COPY.load(Ordering::Relaxed) != 0 {
        return Ok(());
    }
    unsafe {
        let mem = VirtualAlloc(
            std::ptr::null(),
            CODE_LEN,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        ) as *mut u8;
        if mem.is_null() {
            return Err("VirtualAlloc failed");
        }
        // Unhooked, so the copy starts with the original's own first bytes.
        HOOK.run_unhooked(|| {
            copy_nonoverlapping(game_addr(rva(CODE_START)) as *const u8, mem, CODE_LEN)
        });
        let code = from_raw_parts_mut(mem, CODE_LEN);
        if code[..5] != [0x56, 0x57, 0x55, 0x8B, 0xEB] {
            return Err("unexpected first bytes in the original");
        }
        for (i, &global) in GLOBAL_RVAS.iter().enumerate() {
            let from = (game_addr(global) as u32).to_le_bytes();
            let to = (&TEST_GLOBALS[i] as *const AtomicU32 as u32).to_le_bytes();
            let (mut at, mut hits) = (0, 0);
            while at + 4 <= CODE_LEN {
                if code[at..at + 4] == from {
                    code[at..at + 4].copy_from_slice(&to);
                    hits += 1;
                    at += 4;
                } else {
                    at += 1;
                }
            }
            if hits == 0 {
                return Err("a global is not referenced in the original");
            }
        }
        patched::COPY.store(mem as usize, Ordering::Relaxed);
    }
    Ok(())
}

/// Runs the patched copy of the original with the given surface, clip and scratch values.
unsafe fn run_copy(
    before: &Regs,
    surface: &Surface,
    clip: &Clip,
    scratch: &Scratch,
) -> (Regs, Scratch) {
    let values = [
        surface.bits_per_pixel,
        surface.bytes_per_pixel as u32,
        surface.pitch as u32,
        surface.base,
        clip.left as u32,
        clip.right as u32,
        clip.top as u32,
        clip.bottom as u32,
        scratch.left_skip as u32,
        scratch.visible_w as u32,
        scratch.top_skip as u32,
        scratch.rows_left as u32,
    ];
    for (slot, v) in TEST_GLOBALS.iter().zip(values) {
        slot.store(v, Ordering::Relaxed);
    }
    let mut regs = *before;
    unsafe { patched::call_copy(&mut regs) };
    let after = |i: usize| TEST_GLOBALS[i].load(Ordering::Relaxed) as i32;
    let scratch = Scratch {
        left_skip: after(8),
        visible_w: after(9),
        top_skip: after(10),
        rows_left: after(11),
    };
    (regs, scratch)
}

/// Rows of (gap, count) runs for a glyph `width` pixels wide: empty rows, rows that fill the
/// width exactly, runs with no gap and runs with gaps.
fn test_rows(width: u32, height: u32, seed: u32) -> Vec<Vec<(u32, u32)>> {
    let mut state = seed.wrapping_mul(2_654_435_761).wrapping_add(12_345);
    let mut next = |below: u32| {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (state >> 16) % below
    };
    (0..height)
        .map(|row| {
            let mut runs = Vec::new();
            if width == 0 || row % 5 == 3 {
                return runs; // an empty row
            }
            if row % 5 == 4 {
                runs.push((0, width)); // one run across the whole width
                return runs;
            }
            let mut at = 0;
            while at < width {
                let gap = next(4).min(width - at);
                let count = (1 + next(5)).min(width - at - gap);
                if count == 0 {
                    break;
                }
                runs.push((gap, count));
                at += gap + count;
            }
            runs
        })
        .collect()
}

/// One glyph record. `unit` 1 = normal record, 2 or 4 = extended record with that pixel size.
fn test_record(
    unit: usize,
    x_offset: u32,
    y_offset: u32,
    width: u32,
    rows: &[Vec<(u32, u32)>],
) -> Vec<u8> {
    let field = |out: &mut Vec<u8>, v: u32| out.extend_from_slice(&v.to_le_bytes()[..unit]);
    let mut out = Vec::new();
    let height = rows.len() as u32;
    if unit == 1 {
        out.extend([x_offset as u8, y_offset as u8, width as u8, height as u8]);
    } else {
        let marker: u32 = if unit == 2 { 0xFFFF_FFFE } else { 0xFFFF_FFFC };
        out.extend_from_slice(&marker.to_le_bytes());
        for v in [x_offset, y_offset, width, height] {
            out.extend_from_slice(&(v as u16).to_le_bytes());
        }
    }
    let mut pixel = 1u32;
    for runs in rows {
        for &(gap, count) in runs {
            field(&mut out, count);
            field(&mut out, gap);
            for _ in 0..count {
                // Never 0xAA in any byte, so a drawn pixel always differs from the fill.
                pixel = pixel % 0x99 + 1;
                field(&mut out, pixel * 0x0101_0101);
            }
        }
        field(&mut out, 0);
    }
    out
}

/// A glyph table: one dword offset per glyph, then the records.
fn test_table(records: &[Vec<u8>]) -> Vec<u8> {
    let mut table = Vec::new();
    let mut at = records.len() * 4;
    for r in records {
        table.extend_from_slice(&(at as u32).to_le_bytes());
        at += r.len();
    }
    for r in records {
        table.extend_from_slice(r);
    }
    table
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Err(why) = unsafe { build_copy() } {
        return t.skip(why);
    }

    // (x offset, y offset, width, height) of the test glyphs, in each of the three encodings.
    let shapes = [
        (0, 0, 1, 1),
        (1, 2, 5, 7),
        (0, 0, 8, 8),
        (3, 0, 12, 10),
        (0, 1, 20, 6),
        (2, 2, 0, 5), // no width
        (2, 2, 6, 0), // no rows
    ];
    let mut records = Vec::new();
    for unit in [1, 2, 4] {
        for (i, &(xo, yo, w, h)) in shapes.iter().enumerate() {
            records.push(test_record(
                unit,
                xo,
                yo,
                w,
                &test_rows(w, h, (unit * 16 + i) as u32),
            ));
        }
    }
    let table = test_table(&records);
    let palette: Vec<u32> = (0..256u32)
        .map(|i| (i.wrapping_mul(0x0101_0101) ^ 0x1122_3344) & 0x7F7F_7F7F)
        .collect();

    // A 64 x 48 pixel surface with spare bytes around it; the clip rectangle lies well inside.
    const MARGIN: usize = 4096;
    const PITCH: usize = 64 * 4 + 16;
    const ROWS: usize = 48;
    let mut pixels = vec![0xAAu8; MARGIN + PITCH * ROWS + MARGIN];
    let clip = Clip {
        left: 10,
        right: 41,
        top: 8,
        bottom: 31,
    };
    let xs = [-30, -3, 2, 6, 9, 10, 12, 20, 25, 33, 38, 41, 42, 60];
    let ys = [-20, -2, 3, 7, 8, 12, 20, 24, 28, 31, 32, 50];
    let stale = Scratch {
        left_skip: 0x5EED_0001,
        visible_w: 0x5EED_0002,
        top_skip: 0x5EED_0003,
        rows_left: 0x5EED_0004,
    };

    for (bits, bytes_per_pixel) in [(8u32, 1), (16, 2), (24, 3), (32, 4)] {
        let surface = Surface {
            base: unsafe { pixels.as_mut_ptr().add(MARGIN) } as u32,
            pitch: PITCH as i32,
            bytes_per_pixel,
            bits_per_pixel: bits,
        };
        for glyph in 0..records.len() as u32 {
            for &x in &xs {
                for &y in &ys {
                    let before = Regs {
                        eax: glyph,
                        ecx: x as u32,
                        edx: y as u32,
                        ebx: palette.as_ptr() as u32,
                        esi: table.as_ptr() as u32,
                        ..Regs::SENTINEL
                    };
                    pixels.fill(0xAA);
                    let (ro, scratch_o) = unsafe { run_copy(&before, &surface, &clip, &stale) };
                    let pixels_o = pixels.clone();

                    pixels.fill(0xAA);
                    let mut scratch_r = stale;
                    let Ok(record) = (unsafe { glyph_record(&surface, table.as_ptr(), glyph) })
                    else {
                        t.check(false, || format!("glyph={glyph}: glyph_record refused it"));
                        continue;
                    };
                    let (exit, _) = unsafe {
                        blit_glyph(
                            record,
                            x,
                            y,
                            palette.as_ptr(),
                            &surface,
                            &clip,
                            &mut scratch_r,
                            &mut |addr, bytes| {
                                copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, bytes.len())
                            },
                        )
                    };

                    let exit_o = Exit {
                        eax: ro.eax,
                        ebx: ro.ebx,
                        ecx: ro.ecx,
                        edx: ro.edx,
                    };
                    let same_pixels = pixels_o == pixels;
                    let preserved = ro.preserved_except(&before, EAX | EBX | ECX | EDX);
                    t.check(
                        same_pixels && exit_o == exit && scratch_o == scratch_r && preserved,
                        || {
                            format!(
                                "bits={bits} glyph={glyph} x={x} y={y}: same_pixels={same_pixels} preserved={preserved} original {exit_o:x?} {scratch_o:x?} rust {exit:x?} {scratch_r:x?}"
                            )
                        },
                    );
                }
            }
        }
    }
    t.finish()
}
