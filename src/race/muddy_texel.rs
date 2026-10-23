//! `race::muddy_texel` (0x44A2E1): one texel of the mud texture being worked on
//! (`dword_623D5C`, 0 .. 3) brought toward the mud's colour: the place is the word
//! `g_shuffledWords[eax & 0xFFFF]` (x its low byte, y its high one), the strength the mud
//! map's byte at (x / 2, y / 2) of the texture's 128 x 128 (`g_mud` + 0x4000 n), kept in
//! `dword_62400C`. When `race::mud_texel` there has its top bit set, it is tinted by
//! `race::tint_to_mud_colour` (ebp = (strength + 1) / 2), the top byte set, and put back
//! (`race::put_mud_texel`).
//!
//! In:  eax = the place's number; ebp
//! Out: ecx = x, edx = y, esi the strength's address, ebp kept; when the texel was not
//!      tinted eax = `mud_texel`'s, ebx the strength; else eax = `put_mud_texel`'s, ebx
//!      the tinted colour; edi kept; no flags read
//!
//! See `race/mud.rs`. Callers: `race::throw_mud_texel` (twice). Refuses a texture past 3
//! (the map would be read past its 64 KB) and no mud, before anything; what
//! `put_mud_texel` refuses (`dword_62400C` put back). Self-test: the original and the Rust
//! version on test surfaces (16 and 32 bits) and a test mud map, each texture and halving,
//! places by lot; the surface compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::gfx::pixel_format::{PixelFormat, SCRATCH_RVA, TEST_FORMATS};
use crate::race::mud::{MUD_LEN, MUD_RVA, TestSurface};
use crate::race::mud_texel::{HALVINGS_RVA, mud_texel, texel_place};
use crate::race::put_mud_texel::put_mud_texel;
use crate::race::stage_prefs::left_of;
use crate::race::tint_to_mud_colour::tint_to_mud_colour;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore, state_entry};

pub const TEXTURE_RVA: usize = rva(0x623D5C); // dword_623D5C: the mud texture worked on
pub const STRENGTH_RVA: usize = rva(0x62400C); // dword_62400C, a scratch
pub const SHUFFLED_RVA: usize = rva(0xA6BB54); // g_shuffledWords, 0x10000 words
const FORMAT_RVA: usize = rva(0x4F49B8); // g_surface.format

/// The place (x, y) of number `n` (its low 16 bits).
pub fn shuffled_place(n: u32) -> (u32, u32) {
    let word: u16 = unsafe { game_read(SHUFFLED_RVA + 2 * (n & 0xFFFF) as usize) };
    rc2k_core::race::mud::shuffled_place(word)
}

/// The texture worked on and the mud, or the refusal.
pub fn texture_and_mud() -> Result<(u32, u32), Refused> {
    let n: u32 = unsafe { game_read(TEXTURE_RVA) };
    if n > rc2k_core::race::mud::MOST_TEXTURE {
        return Err(BadInput("a mud texture past 3 (the map would be read past its end)").into());
    }
    let mud: u32 = unsafe { game_read(MUD_RVA) };
    if mud == 0 {
        return Err(BadInput("no mud").into());
    }
    Ok((n, mud))
}

/// The strength's address for texture `n` at (x, y).
pub fn strength_at(mud: u32, n: u32, x: u32, y: u32) -> u32 {
    mud.wrapping_add(rc2k_core::race::mud::strength_index(n, x, y))
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn muddy_texel(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::mud::{shuffled_place, strength_index, muddy,
    // tint_amount}`.
    use rc2k_core::race::mud::{MUD_TOP, muddy, strength_row, tint_amount};
    let (n, mud) = texture_and_mud()?;
    let (x, y) = shuffled_place(entry.eax);
    let mut r = *entry;
    r.esi = strength_at(mud, n, x, y);
    let strength = u32::from(unsafe { *(r.esi as *const u8) });
    let kept: u32 = unsafe { game_read(STRENGTH_RVA) };
    unsafe { game_write(STRENGTH_RVA, strength) };
    r.ecx = x;
    r.edx = y;
    let texel = unsafe { mud_texel(strength_row(y), x, y) };
    if !muddy(texel) {
        r.eax = texel;
        r.ebx = strength;
        return Ok(r);
    }
    let tinted = tint_to_mud_colour(&Regs {
        ebx: texel,
        ebp: tint_amount(strength),
        ..r
    });
    r.ebx = tinted.ebx;
    let put = Regs {
        eax: tinted.eax | MUD_TOP,
        ..r
    };
    r.eax = match unsafe { put_mud_texel(&put) } {
        Ok(eax) => eax,
        Err(e) => {
            unsafe { game_write(STRENGTH_RVA, kept) };
            return Err(e);
        }
    };
    Ok(r)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { muddy_texel(r) }?))
}

/// What one call may write: the strength, the format's scratch, the texel's pixel.
pub fn regions(n: u32) -> Vec<Region> {
    let mut v = vec![at(STRENGTH_RVA), at(SCRATCH_RVA)];
    let (x, y) = shuffled_place(n);
    if let Some((x, y)) = texel_place(x, y) {
        unsafe {
            if x < game_read::<u32>(rva(0x4F49A0)) && y < game_read::<u32>(rva(0x4F49A4)) {
                let (at, wide) = crate::gfx::put_surface_pixel::place(x, y);
                v.push((at, wide as usize));
            }
        }
    }
    v
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// All of `g_surface`'s pixels (pitch x height; nothing without a surface): what a call
/// whose places are not known before may write.
pub fn surface_region() -> Region {
    unsafe {
        let base: u32 = game_read(rva(0x4F49C0));
        let len = game_read::<u32>(rva(0x4F49BC)).wrapping_mul(game_read(rva(0x4F49A4)));
        if base == 0 || len > 0x100_0000 {
            (base, 0)
        } else {
            (base, len as usize)
        }
    }
}

/// A mud map of the test's in `g_mud` until dropped (64 KB of a pattern), the texture
/// number and the halvings saved.
pub struct TestMud {
    pub map: Vec<u8>,
    saved: Vec<u8>,
}

fn kept() -> [Region; 4] {
    [
        at(MUD_RVA),
        at(TEXTURE_RVA),
        at(HALVINGS_RVA),
        at(STRENGTH_RVA),
    ]
}

impl TestMud {
    pub fn new() -> Self {
        let saved = capture(&kept());
        let mut map: Vec<u8> = (0..MUD_LEN)
            .map(|n| (n.wrapping_mul(0x9D) >> 3) as u8)
            .collect();
        unsafe { game_write(MUD_RVA, map.as_mut_ptr() as u32) };
        Self { map, saved }
    }
}

impl Default for TestMud {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestMud {
    fn drop(&mut self) {
        restore(&kept(), &self.saved);
    }
}

/// A mud test's case: its name, the surface's pixels, the reset.
pub type MudCase<'a> = &'a mut dyn FnMut(&str, Region, &dyn Fn());

/// The mud tests' set-up: a test mud map, then each test surface (16 and 32 bits) under
/// the game's format and two of the test's with an alpha mask (only a texel with its top bit
/// set is changed), each texture and halving; `f` gets the case's name, the surface's
/// pixels and a `reset` that puts them back and sets the texture and halving. The format's
/// scratch saved and put back.
pub fn each_mud_case(f: MudCase) {
    let scratch = [at(SCRATCH_RVA)];
    let saved = capture(&scratch);
    let _mud = TestMud::new();
    for bits in [16u32, 32] {
        let surface = TestSurface::new(bits);
        let pixels: Region = (surface.pixels.as_ptr() as u32, surface.pixels.len());
        let start = capture(&[pixels]);
        let mut formats = vec![None];
        formats.extend([1usize, 2].map(|k| Some(TEST_FORMATS[k])));
        for format in formats {
            let mut f_ = format.unwrap_or(TEST_FORMATS[0]);
            if format.is_some() {
                // `TestSurface` puts the format pointer back.
                unsafe { game_write(FORMAT_RVA, &mut f_ as *mut PixelFormat as u32) };
            }
            for texture in 0..4u32 {
                for halvings in [0u32, 1, 2] {
                    let reset = || unsafe {
                        restore(&[pixels], &start);
                        game_write(TEXTURE_RVA, texture);
                        game_write(HALVINGS_RVA, halvings);
                    };
                    let what = format!(
                        "{bits} bits, format {format:x?}, texture {texture}, halvings {halvings}"
                    );
                    f(&what, pixels, &reset);
                }
            }
        }
        drop(surface);
    }
    restore(&scratch, &saved);
}
