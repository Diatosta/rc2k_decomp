//! `gfx::show_picture_file` (0x41172B): loads a picture and shows it at once, on every
//! page of the screen: the loading picture.
//!
//! In:  eax = the picture's path
//! Out: eax = what `dword_51B9C4` held (1, unless the file could not be read: then the
//!      path); the rest as its callees leave them
//! The window's messages are passed on (`sub_410535`); `dword_51B9C4` = 1; the picture's
//! size is set to 800 x 600 with row length 0xFFFFFFFD and the file is read
//! (`gfx::read_picture_file`); the size then becomes the screen's, the row length the
//! screen's width * 4.
//! - Not read: `gfx::clear_picture`, and on as for a picture of the screen's width.
//! - Read: `dword_51B9C4` is taken (set to 0); if it was not 0 and the fonts are built
//!   (`g_fontPalettes`), the picture's flags become 0x802, the picture is made the
//!   surface (`gfx::picture_as_surface`) and string 0xC0 is drawn on it at (790, 47) with
//!   the font at 0x51EAA4.
//! - A picture of the screen's width: `gfx::convert_picture`, then twice
//!   `gfx::copy_picture_to_surface` and `gfx::show_frame`, and a third time if the
//!   screen mode has bit 0x40000 (three pages).
//! - Any other width: as many times, `gfx::lock_surface` (failing ends it),
//!   `gfx::rescale_to_surface`, `gfx::unlock_surface`, `gfx::show_frame`.
//!
//! One caller, `menu::show_loading_picture`.
//! Every call goes through the game's address, so each callee's own hook decides which
//! version of it runs; the registers are handed from one to the next as they come back.
//! Shadow mode runs the original alone (it shows frames). Manual patch with a private
//! copy (0x138 bytes, 24 calls relocated): the function begins with a call.
//! Self-test: a redirected copy whose eleven kinds of call go to the test.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use rc2k_core::gfx::picture_size as sizes;

// ---- game data ----

const SHOWING_RVA: usize = rva(0x51B9C4); // dword_51B9C4
const PICTURE_FLAGS_RVA: usize = rva(0x4F49CC); // g_pictureFlags
const PICTURE_WIDTH_RVA: usize = rva(0x4F49D0); // g_pictureWidth, height, row bytes
const SCREEN_WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const SCREEN_HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const SCREEN_MODE_FLAGS_RVA: usize = rva(0x4F4934); // g_screenModeFlags
const DECODED_WIDTH_RVA: usize = rva(0x519124); // g_decodedWidth
const FONT_PALETTES_RVA: usize = rva(0x51DEB0); // g_fontPalettes
const LOADING_FONT_RVA: usize = rva(0x51EAA4); // stru_51EAA4

/// The callees, in the order of the test's stand-ins.
const PUMP_MESSAGES: usize = 0x410535;
const READ_PICTURE_FILE: usize = 0x4119F4;
const PICTURE_AS_SURFACE: usize = 0x405FE7;
const DRAW_STRING_BY_ID: usize = 0x40234A;
const LOCK_SURFACE: usize = 0x4058AD;
const RESCALE_TO_SURFACE: usize = 0x4063B7;
const UNLOCK_SURFACE: usize = 0x405949;
const SHOW_FRAME: usize = 0x4057D1;
const CLEAR_PICTURE: usize = 0x4060D1;
const CONVERT_PICTURE: usize = 0x406126;
const COPY_PICTURE_TO_SURFACE: usize = 0x4060E3;

// ---- Rust version ----

/// Calls the function at that address (as IDA shows it) with the registers, which come
/// back as it leaves them.
pub type CallAt<'a> = &'a mut dyn FnMut(usize, &mut Regs);

/// `r` holds the registers at entry (eax the path) and at the end those to return with.
pub unsafe fn show_picture_file(r: &mut Regs, call: CallAt) {
    unsafe {
        call(PUMP_MESSAGES, r);
        game_write(SHOWING_RVA, 1u32);
        // Converted: `rc2k_core::gfx::picture_size` (`READ`, `screen`, `pages`,
        // `rescaled`).
        game_write(PICTURE_WIDTH_RVA, sizes::READ.dwords());
        call(READ_PICTURE_FILE, r);
        let read = !r.carry();
        let (width, height): (u32, u32) =
            (game_read(SCREEN_WIDTH_RVA), game_read(SCREEN_HEIGHT_RVA));
        game_write(PICTURE_WIDTH_RVA, sizes::screen(width, height).dwords());
        let pages = sizes::pages(game_read::<u32>(SCREEN_MODE_FLAGS_RVA));
        if !read {
            call(CLEAR_PICTURE, r);
        } else {
            r.eax = game_read(SHOWING_RVA);
            game_write(SHOWING_RVA, 0u32);
            if r.eax != 0 && game_read::<u32>(FONT_PALETTES_RVA) != 0 {
                // Between `pusha` and `popa`: what these two leave is not kept.
                let mut inner = *r;
                game_write(PICTURE_FLAGS_RVA, 0x802u32);
                call(PICTURE_AS_SURFACE, &mut inner);
                inner.eax = 0xC0;
                inner.ebx = game_addr(LOADING_FONT_RVA) as u32;
                inner.ecx = 0x316;
                inner.edx = 0x2F;
                call(DRAW_STRING_BY_ID, &mut inner);
            }
            if sizes::rescaled(game_read::<u32>(DECODED_WIDTH_RVA), width) {
                for _ in 0..pages {
                    call(LOCK_SURFACE, r);
                    if r.carry() {
                        return;
                    }
                    call(RESCALE_TO_SURFACE, r);
                    call(UNLOCK_SURFACE, r);
                    call(SHOW_FRAME, r);
                }
                return;
            }
        }
        call(CONVERT_PICTURE, r);
        for _ in 0..pages {
            call(COPY_PICTURE_TO_SURFACE, r);
            call(SHOW_FRAME, r);
        }
    }
}

fn in_game(at: usize, r: &mut Regs) {
    unsafe { game_call(at, r) };
}

/// The callers' calls: `text::draw_string_by_id` in Rust (its hook is gone; a refusal
/// runs its original), the rest in the game.
pub unsafe fn game_call(at: usize, r: &mut Regs) {
    unsafe {
        if at == DRAW_STRING_BY_ID {
            let entry = *r;
            if crate::text::draw_string_by_id::draw_string_by_id_regs(r).is_ok() {
                return;
            }
            *r = entry;
        }
        crate::calls::call(rva(at), r)
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let esp = r.esp;
    unsafe { show_picture_file(r, &mut in_game) };
    r.esp = esp;
    Ok(())
}
