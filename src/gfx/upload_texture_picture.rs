//! `gfx::upload_texture_picture` (0x41A68B): a picture into every level of a system
//! texture.
//!
//! In:  eax = the system texture, ebx = the picture's pixels (0: nothing is done),
//!      ecx, edx = its width and height
//! Out: carry clear; every register preserved
//!
//! - The picture made ready (`gfx/texture_upload.rs`: the least colour, the alpha from
//!   the kept picture or the colours keyed, the grading).
//! - Picture and texture the same number of halvings from 256 across and down, else a
//!   fatal error (0x80A18741 .. 0x80A18747).
//! - For each of the record's levels (+0x10; one at least): the picture halved (that
//!   many times for the first, once for each after), the level's surface (+0x14 ..)
//!   locked through its `Lock` (DirectDraw, vtable +0x64, into the description at
//!   `dword_5ABA18`), every pixel through `components_to_pixel` with the texture's
//!   format into it as words or dwords by the kind's bytes a pixel (another number: a
//!   fatal error 0x80A18775), rows `dword_5ABA28` apart from `dword_5ABA3C`, and
//!   `Unlock` (+0x80).
//! - A `Lock` or `Unlock` that fails is a fatal error (0x80218772, 0x80218773) when
//!   `dword_4F52AC` is 1; otherwise the original returns with three dwords still
//!   pushed, so its `popa` and `ret` go astray: refused.
//!
//! One caller, `gfx::load_picture_to_texture` (the other reference, 0x425A52, is in code
//! nothing reaches).
//! The two DirectDraw calls go through `Surfaces`, which the standalone program can
//! answer with textures of its own.
//! Refuses a texture of 0x140 or more, a record without its format, more than nine
//! levels, a picture that halves to nothing, and what `ready_picture` refuses; a
//! refusal after the picture was changed hands the original the changed picture.
//! Shadow mode runs the original alone.
//! Self-test: the original and the Rust version on records of the test's whose
//! surfaces are objects of the test's, 12 cases.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::gfx::block::{game_addr, game_read};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::reports::Slot as RepSlot;
use crate::gfx::texture_pixels::{halvable, halve_texture};
use crate::gfx::texture_upload::{Record, fatal, halvings_apart, ready_picture};
use crate::gfx::upload::Slot as UpSlot;
use rc2k_core::gfx::uploads::{self};

// ---- game data ----

/// `dword_5ABA18`: the surface description `Lock` fills (0x6C bytes).
const DESCRIPTION_RVA: UpSlot = UpSlot::at(0x5ABA18);
const PITCH_RVA: UpSlot = UpSlot::at(0x5ABA28); // dword_5ABA28: its pitch
const PIXELS_RVA: UpSlot = UpSlot::at(0x5ABA3C); // dword_5ABA3C: its pixels
const REPORTS_RVA: RepSlot = RepSlot::at(0x4F52AC); // dword_4F52AC: Direct3D's errors are fatal

// ---- Rust version ----

/// A level's surface locked and unlocked; an error is what DirectDraw returned.
pub trait Surfaces {
    /// The pixels and the pitch.
    fn lock(&mut self, surface: u32) -> Result<(u32, u32), i32>;
    fn unlock(&mut self, surface: u32) -> Result<(), i32>;
}

/// The game's: DirectDraw's own calls, the description being the game's.
pub struct GameSurfaces;

type Lock = unsafe extern "system" fn(u32, u32, u32, u32, u32) -> i32;
type Unlock = unsafe extern "system" fn(u32, u32) -> i32;

unsafe fn method(object: u32, offset: u32) -> usize {
    unsafe {
        (((object as *const u32).read_unaligned() + offset) as *const u32).read_unaligned() as usize
    }
}

impl Surfaces for GameSurfaces {
    fn lock(&mut self, surface: u32) -> Result<(u32, u32), i32> {
        unsafe {
            let lock: Lock = std::mem::transmute(method(surface, 0x64));
            match lock(surface, 0, game_addr(DESCRIPTION_RVA) as u32, 1, 0) {
                0 => Ok((game_read(PIXELS_RVA), game_read(PITCH_RVA))),
                e => Err(e),
            }
        }
    }

    fn unlock(&mut self, surface: u32) -> Result<(), i32> {
        unsafe {
            let unlock: Unlock = std::mem::transmute(method(surface, 0x80));
            match unlock(surface, 0) {
                0 => Ok(()),
                e => Err(e),
            }
        }
    }
}

fn direct_draw_failed(code: u32) -> Refused {
    if unsafe { game_read::<u32>(REPORTS_RVA) } == 1 {
        fatal(code)
    } else {
        BadInput("DirectDraw failed and the original's stack goes astray").into()
    }
}

pub unsafe fn upload_texture_picture(
    texture: u32,
    pixels: u32,
    width: u32,
    height: u32,
    surfaces: &mut dyn Surfaces,
) -> Result<(), Refused> {
    if pixels == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::gfx::uploads` (`levels`, `halvings_before`; the halvings
    // through `texture_upload::halvings_apart`).
    let record = Record::of(texture)?;
    let levels = uploads::levels(record.get().levels).map_err(BadInput::from)?;
    unsafe {
        ready_picture(&record, pixels, width, height)?;
        let apart = halvings_apart(
            &record,
            width,
            height,
            [
                0x80A1_8741,
                0x80A1_8742,
                0x80A1_8744,
                0x80A1_8745,
                0x80A1_8747,
            ],
        )?;
        if !halvable(width, height, apart + levels - 1) {
            return Err(BadInput("a picture that halves to nothing").into());
        }
        let format = record.format();
        let bytes = record.bytes_per_pixel();
        let (mut width, mut height) = (width, height);
        for level in 0..levels {
            let times = uploads::halvings_before(level, apart);
            if times != 0 {
                (width, height) = halve_texture(pixels as *mut u32, width, height, times);
            }
            let surface = record.get().surfaces[level as usize];
            let (mut target, pitch) = surfaces
                .lock(surface)
                .map_err(|_| direct_draw_failed(0x8021_8772))?;
            let mut source = pixels as *const u32;
            for _ in 0..height {
                match bytes {
                    2 => {
                        for n in 0..width {
                            let pixel = components_to_pixel(&format, source.read_unaligned());
                            ((target + n * 2) as *mut u16).write_unaligned(pixel as u16);
                            source = source.add(1);
                        }
                    }
                    4 => {
                        for n in 0..width {
                            let pixel = components_to_pixel(&format, source.read_unaligned());
                            ((target + n * 4) as *mut u32).write_unaligned(pixel);
                            source = source.add(1);
                        }
                    }
                    _ => return Err(fatal(0x80A1_8775)),
                }
                target = target.wrapping_add(pitch);
            }
            surfaces
                .unlock(surface)
                .map_err(|_| direct_draw_failed(0x8021_8773))?;
        }
    }
    Ok(())
}
