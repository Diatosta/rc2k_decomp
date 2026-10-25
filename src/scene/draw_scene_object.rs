//! `scene::draw_scene_object` (0x451E3E): chooses the set of textures for the object
//! `g_objectNumber` and draws it the way the scene's mode asks for.
//!
//! In:  esi, passed on to `scene::draw_described_object`
//! Out: the registers as its parts leave them; before the drawing eax = what was last
//!      looked at, ebx = the object's number, edx = the car's remainder if a car was
//!      looked at
//!
//! An object's number of 0x100 or more is fatal error 0x80600A01. `dword_639954` (added to
//! every face's texture number) = 0; then 0x108 for, outside a race, an object of kind 0x23A,
//! and in a race for objects 0x50 .. 0xEF (the cars, 0x14 objects each) whose car's block has
//! 0x21 at +0x24. Nothing more for an object without faces (`g_objectFaceCounts`). Else by
//! `dword_6365C4`: 0 `scene::draw_shown_object`; 1 and 2 `scene::draw_described_object` (it
//! takes an object's description in esi and comes back here); anything else nothing. Twelve
//! callers. Refuses a car without its block.
//!
//! Converted 2026-10-21: the texture set and the way on through
//! `rc2k_core::scene::shown`.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::race::numbers_store::NUMBERS;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_object::draw_object_in_game;
use crate::scene::draw_shown_object::draw_shown_object;

// ---- game data ----

const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const TEXTURE_SET_RVA: usize = rva(0x639954); // dword_639954
const KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds
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

/// The texture set for `g_objectNumber`; the object's number.
// Converted: `rc2k_core::scene::shown::texture_set`
unsafe fn texture_set() -> Result<(u32, u32), Refused> {
    use rc2k_core::scene::shown::SetError;
    unsafe {
        let object: u32 = game_read(OBJECT_RVA);
        let t = rc2k_core::scene::shown::texture_set(
            object,
            NUMBERS.in_race.bits() != 0,
            0,
            |object| game_read(KINDS_RVA + object as usize * 4),
            |car| {
                let block: u32 = RACE_WORK.car_block(car as usize);
                (block != 0).then(|| ((block + 0x24) as *const u32).read_unaligned())
            },
        )
        .map_err(|e| -> Refused {
            match e {
                SetError::TooHigh => TOO_HIGH.into(),
                SetError::NoBlock => BadInput("a car without its block").into(),
            }
        })?;
        Ok((t.set, object))
    }
}

/// `draw`: what stands for the two calls. What the original leaves in the registers (ebx
/// the object's number among them) none of its callers reads (register liveness,
/// 2026-10-23: only the unreached `sub_49081A` after `scene::draw_menu_object`).
// Converted: `rc2k_core::scene::shown::way`
pub unsafe fn draw_scene_object(
    draw: &mut dyn FnMut(Way) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let (set, object) = texture_set()?;
        game_write(TEXTURE_SET_RVA, set);
        let faces: u32 = game_read(FACE_COUNTS_RVA + object as usize * 4);
        match rc2k_core::scene::shown::way(faces, game_read(MODE_RVA)) {
            Some(way) => draw(way),
            None => Ok(()),
        }
    }
}

/// `draw_scene_object` with the game's parts (`scene::draw_shown_object` and its drawing;
/// `scene::draw_described_object`, given `description`, esi at the original's entry: the
/// object's description, read only on that way, the scene's mode 1 or 2).
pub unsafe fn draw_scene_object_in_game(description: u32) -> Result<(), Refused> {
    unsafe {
        draw_scene_object(&mut |way| match way {
            Way::Shown => draw_shown_object(&mut || draw_object_in_game()).map(|_| ()),
            Way::Described => {
                crate::scene::draw_described_object::draw_described_object(description)
            }
        })
    }
}
