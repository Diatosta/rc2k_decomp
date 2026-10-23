//! `scene::draw_scene_object` (0x451E3E): chooses the set of textures for the object
//! `g_objectNumber` and draws it the way the scene's mode asks for.
//!
//! In:  esi, passed on to `scene::draw_described_object`
//! Out: the registers as its parts leave them; before the drawing eax = what was last
//!      looked at, ebx = the object's number, edx = the car's remainder if a car was
//!      looked at
//!
//! An object's number of 0x100 or more is fatal error 0x80600A01.
//! `dword_639954` (added to every face's texture number) = 0; then 0x108 for, outside
//! a race, an object of kind 0x23A, and in a race for objects 0x50 .. 0xEF (the cars,
//! 0x14 objects each) whose car's block has 0x21 at +0x24.
//! Nothing more for an object without faces (`g_objectFaceCounts`). Else by
//! `dword_6365C4`: 0 `scene::draw_shown_object`; 1 and 2 `scene::draw_described_object` (it
//! takes an object's description in esi and comes back here); anything else nothing.
//! Twelve callers.
//! Refuses a car without its block.
//! Shadow mode runs the original alone and, for mode 0, checks the texture set it
//! chose against the Rust arithmetic.
//! Self-test: a redirected copy whose three calls note themselves.
//!
//! Converted 2026-10-21: the texture set and the way on through
//! `rc2k_core::scene::shown`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::scene::draw_object::draw_object_in_game;
use crate::scene::draw_shown_object::draw_shown_object;

// ---- game data ----

const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const TEXTURE_SET_RVA: usize = rva(0x639954); // dword_639954
const IN_RACE_RVA: usize = rva(0x5C5CEC); // g_inRace
const KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds
const BLOCKS_RVA: usize = rva(0x71B3B8); // g_carBlocks
const FACE_COUNTS_RVA: usize = rva(0x632F70); // dword_632F70
const MODE_RVA: usize = rva(0x6365C4); // dword_6365C4
// scene::draw_described_object

const TOO_HIGH: FatalError = FatalError {
    code: 0x8060_0A01,
    message: "object number too high",
};

/// The two ways on from here.
pub use rc2k_core::scene::shown::Way;

// ---- Rust version ----

/// The texture set and the registers the choosing leaves. Writes only `r`.
// Converted: `rc2k_core::scene::shown::texture_set`
unsafe fn texture_set(r: &mut Regs) -> Result<u32, Refused> {
    use rc2k_core::scene::shown::SetError;
    unsafe {
        let object: u32 = game_read(OBJECT_RVA);
        r.ebx = object;
        let t = rc2k_core::scene::shown::texture_set(
            object,
            game_read::<u32>(IN_RACE_RVA) != 0,
            r.eax,
            |object| game_read(KINDS_RVA + object as usize * 4),
            |car| {
                let block: u32 = game_read(BLOCKS_RVA + car as usize * 4);
                (block != 0).then(|| ((block + 0x24) as *const u32).read_unaligned())
            },
        )
        .map_err(|e| -> Refused {
            match e {
                SetError::TooHigh => TOO_HIGH.into(),
                SetError::NoBlock => BadInput("a car without its block").into(),
            }
        })?;
        r.eax = t.eax;
        r.edx = t.edx.unwrap_or(r.edx);
        Ok(t.set)
    }
}

/// `draw`: what stands for the two calls.
// Converted: `rc2k_core::scene::shown::way`
pub unsafe fn draw_scene_object(
    r: &mut Regs,
    draw: &mut dyn FnMut(Way, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let mut left = *r;
        let set = texture_set(&mut left)?;
        *r = left;
        game_write(TEXTURE_SET_RVA, set);
        let faces: u32 = game_read(FACE_COUNTS_RVA + r.ebx as usize * 4);
        match rc2k_core::scene::shown::way(faces, game_read(MODE_RVA)) {
            Some(way) => draw(way, r),
            None => Ok(()),
        }
    }
}

/// `draw_scene_object` with the game's parts (`scene::draw_shown_object` and its drawing;
/// `scene::draw_described_object`), for Rust callers.
pub unsafe fn draw_scene_object_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        draw_scene_object(r, &mut |way, r| match way {
            Way::Shown => draw_shown_object(r, &mut |r| draw_object_in_game(r)).map(|_| ()),
            Way::Described => crate::scene::draw_described_object::draw_described_object(r),
        })
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_scene_object_in_game(r) }?;
    Ok(())
}
