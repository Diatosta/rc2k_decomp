//! `gfx::picture_as_surface` (0x405FE7): makes the picture buffer the surface that the
//! blitters draw on, so that sprites can be drawn into a picture.
//!
//! In:  nothing
//! Out: every register preserved
//! From `g_pictureFlags`: the low byte names the size (the table `dword_4F4E50`: 1 = 640 x
//! 480, 2 = 800 x 600, 4 = 960 x 720, 8 = 1024 x 768, 0x10 = 1152 x 864, 0x20 = 1280 x 1024,
//! 0x40 = 1600 x 1200, 0x80 = 1920 x 1280; anything else is fatal error 0x80A19091), bits 8
//! to 11 the pixels (the table `dword_4F4E40`: 0x200 = 16 bits, 0x800 = 32; anything else
//! is fatal error 0x80A19092). Into the surface block (`g_surface`, 0x4F4998) go the
//! width, the height, the bits and bytes of a pixel, the bytes of a row (also as the
//! pitch, with no gap after a row, and as dwords), the picture buffer as the pixels and
//! `g_surfaceFormat` as the format; the block's corner is left as it is. Then the clip
//! rectangle is set to the block's rectangle (`set_clip`).
//! Five callers, `menu::load_background` among them.
//! Shadow mode runs the original and checks the block and the clip rectangle against what
//! the Rust version works out. The name is a reading.

use crate::errors::{FatalError, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::picture::{FLAGS_RVA, PICTURE_RVA};
use crate::gfx::pixel_format::SURFACE_FORMAT_RVA;
use crate::gfx::set_clip::{clip_of, set_clip};
use crate::gfx::surface::{CLIP_BOTTOM_RVA, CLIP_LEFT_RVA, CLIP_RIGHT_RVA, CLIP_TOP_RVA};
use crate::redirect::RedirectedCopy;
use crate::system::find_entry::entries;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::picture_as_surface", rva(0x405FE7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// unk_4F4998: the surface block, twelve dwords: x, y, width, height, bits a pixel, bytes
/// a pixel, bytes of a row's pixels, dwords of a row, the pixel format, the pitch, the
/// pixels, the gap after a row.
pub const SURFACE_RVA: usize = rva(0x4F4998);
pub const SURFACE_DWORDS: usize = 12;
const SIZES_RVA: usize = rva(0x4F4E50); // dword_4F4E50: key, width and height as words
const SIZE_COUNT: u32 = 8;
const PIXEL_KINDS_RVA: usize = rva(0x4F4E40); // dword_4F4E40: key, bits
const PIXEL_KIND_COUNT: u32 = 2;

// ---- Rust version ----

/// The surface block for a picture with these flags, from its third dword on (the corner
/// is not touched).
pub unsafe fn surface_of(flags: u32, pixels: u32) -> Result<[u32; SURFACE_DWORDS - 2], Refused> {
    // Converted: `rc2k_core::gfx::screen_modes::picture_surface`.
    unsafe {
        rc2k_core::gfx::screen_modes::picture_surface(
            flags,
            entries(game_addr(SIZES_RVA) as u32, SIZE_COUNT),
            entries(game_addr(PIXEL_KINDS_RVA) as u32, PIXEL_KIND_COUNT),
            game_addr(SURFACE_FORMAT_RVA) as u32,
            pixels,
        )
    }
    .map_err(|e| {
        FatalError {
            code: e.code(),
            message: "",
        }
        .into()
    })
}

unsafe fn surface_now() -> [u32; SURFACE_DWORDS] {
    unsafe { game_read(SURFACE_RVA) }
}

pub unsafe fn picture_as_surface() -> Result<(), Refused> {
    unsafe {
        let block = surface_of(game_read(FLAGS_RVA), game_read(PICTURE_RVA))?;
        game_write(SURFACE_RVA + 8, block);
        set_clip(game_addr(SURFACE_RVA) as *const [u32; 4])?;
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        if let Err(e) = unsafe { picture_as_surface() } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let mine = unsafe { surface_of(game_read(FLAGS_RVA), game_read(PICTURE_RVA)) };
    let mut o = *r;
    unsafe { call_original(&mut o) };
    // It came back, so it found both codes: what it left is compared with the arithmetic.
    let block = unsafe { surface_now() };
    let clip: [u32; 4] = unsafe {
        [
            game_read(CLIP_LEFT_RVA),
            game_read(CLIP_TOP_RVA),
            game_read(CLIP_RIGHT_RVA),
            game_read(CLIP_BOTTOM_RVA),
        ]
    };
    let as_rust = mine.as_ref().is_ok_and(|mine| {
        block[2..] == mine[..] && clip == clip_of([block[0], block[1], block[2], block[3]])
    });
    if !as_rust || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "flags={:#x}: original left {block:x?} clip {clip:?}; rust {mine:x?}",
                unsafe { game_read::<u32>(FLAGS_RVA) }
            )
        });
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { picture_as_surface() }?;
    Ok(())
}

// ---- self-test: a copy of the original that reads the flags from a test variable, writes a
// surface block of the test's and hands the clip rectangle to a stand-in; the real surface
// and clip rectangle are what the game's drawing reads all the time ----

static T_FLAGS: AtomicU32 = AtomicU32::new(0);
static T_SURFACE: [AtomicU32; SURFACE_DWORDS] = [const { AtomicU32::new(0) }; SURFACE_DWORDS];
/// The rectangle the copy handed to `set_clip`.
static T_CLIPPED: AtomicU32 = AtomicU32::new(0);

#[unsafe(naked)]
unsafe extern "C" fn set_clip_stand_in() {
    core::arch::naked_asm!(
        "mov dword ptr [{clipped}], ebx",
        "ret",
        clipped = sym T_CLIPPED,
    )
}

const LEN: usize = 0xA8;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xA1, 0xCC, 0x49, 0x4F, 0x00, 0x25, 0xFF];
/// `find_entry` twice, which stays, and `set_clip`.
const CALLS: [usize; 3] = [0x16, 0x48, 0xA1];
const SET_CLIP: usize = rva(0x406988);
/// Where the copy names the flags.
const FLAGS_OPERANDS: [usize; 2] = [0x02, 0x34];
/// Where it names a dword of the surface block.
const SURFACE_OPERANDS: [usize; 11] = [
    0x26, 0x2F, 0x57, 0x5F, 0x66, 0x6B, 0x70, 0x76, 0x82, 0x8C, 0x97,
];
/// Where it takes the block's address for `set_clip`.
const BLOCK_OPERAND: usize = 0x9D;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let redirect =
        |target: usize| (target == SET_CLIP).then_some(set_clip_stand_in as *const () as usize);
    let copy = match unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x405FE7)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &redirect,
        )
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let code = unsafe { std::slice::from_raw_parts(copy.addr() as *const u8, LEN) }.to_vec();
    let named = |operand: usize| u32::from_le_bytes(code[operand..operand + 4].try_into().unwrap());
    let (surface, flags) = (game_addr(SURFACE_RVA) as u32, game_addr(FLAGS_RVA) as u32);
    let test_block = T_SURFACE.as_ptr() as u32;
    // Each operand must name what the list says it does, or the function is not the one
    // the list was made from.
    if FLAGS_OPERANDS
        .iter()
        .any(|&operand| named(operand) != flags)
        || named(BLOCK_OPERAND) != surface
        || SURFACE_OPERANDS
            .iter()
            .any(|&operand| named(operand).wrapping_sub(surface) >= SURFACE_DWORDS as u32 * 4)
    {
        return t.skip("the function is not what the operand list was made from");
    }
    unsafe {
        for operand in FLAGS_OPERANDS {
            copy.patch_u32(operand, &T_FLAGS as *const AtomicU32 as u32);
        }
        for operand in SURFACE_OPERANDS {
            copy.patch_u32(operand, test_block + (named(operand) - surface));
        }
        copy.patch_u32(BLOCK_OPERAND, test_block);
    }
    let pixels: u32 = unsafe { game_read(PICTURE_RVA) };
    // Every size with both kinds of pixel; the codes that are fatal are left to the Rust
    // version's reading alone.
    for size in [1u32, 2, 4, 8, 0x10, 0x20, 0x40, 0x80] {
        for kind in [0x200u32, 0x800] {
            for other in [0u32, 0xFFFF_F000] {
                let flags = size | kind | other;
                for (n, variable) in T_SURFACE.iter().enumerate() {
                    variable.store(0xAAAA_0000 + n as u32, Ordering::SeqCst);
                }
                T_FLAGS.store(flags, Ordering::SeqCst);
                T_CLIPPED.store(0, Ordering::SeqCst);
                let before = Regs::SENTINEL;
                let mut ro = before;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs: Vec<u32> = T_SURFACE.iter().map(|v| v.load(Ordering::SeqCst)).collect();
                let mine = unsafe { surface_of(flags, pixels) };
                let preserved = ro.preserved_except(&before, 0);
                let ok = mine.as_ref().is_ok_and(|mine| theirs[2..] == mine[..])
                    && theirs[..2] == [0xAAAA_0000, 0xAAAA_0001]
                    && T_CLIPPED.load(Ordering::SeqCst) == test_block
                    && preserved;
                t.check(ok, || {
                    format!(
                        "flags={flags:#x}: original {theirs:x?} clipped to {:#x} preserved={preserved}; rust {mine:x?}",
                        T_CLIPPED.load(Ordering::SeqCst)
                    )
                });
            }
        }
    }
    t.finish()
}
