//! `scene::load_front_end_objects` (0x425CA9): loads the shapes of all 0x100 object
//! slots.
//!
//! In:  nothing
//! Out: eax = the faces' memory used, ecx = 0x100, edi = the end of the kinds' table,
//!      ebx = the last slot that had a kind (unchanged if none had)
//!
//! `scene::reset_objects`; `dword_631B68` and `dword_631F6C` = 0; for each slot its
//! kind from `dword_5CCE7C` to `dword_634FE8`, the slot to `dword_634FEC`, and
//! `scene::load_object`; then `dword_634FA4` = the points used, in bytes, and
//! `dword_634FA8` = the faces' memory used.
//! Two callers (the front end's start and a stage's leaving).
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose two calls note themselves.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::scene::load_object::load_object_in_game;
use crate::scene::objects::{FACE_CURSOR_RVA, KIND_RVA, POINT_COUNT_RVA, SLOT_RVA, SLOTS};
use crate::scene::reset_objects::reset_objects;

// ---- game data ----

const TEXTURE_COUNT_RVA: usize = rva(0x631B68); // dword_631B68
const NO_NORMALS_RVA: usize = rva(0x631F6C); // dword_631F6C
const SLOT_KINDS_RVA: usize = rva(0x5CCE7C); // dword_5CCE7C
/// `dword_634FA4` the points used in bytes, `dword_634FA8` the faces' memory used.
const USED_RVA: usize = rva(0x634FA4);

// ---- Rust version ----

/// `reset`: what stands for `scene::reset_objects`; `load`: for `scene::load_object`.
pub unsafe fn load_front_end_objects(
    r: &mut Regs,
    reset: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
    load: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        reset(r)?;
        game_write(TEXTURE_COUNT_RVA, 0u32);
        game_write(NO_NORMALS_RVA, 0u32);
        for slot in 0..SLOTS {
            r.ecx = slot;
            r.eax = game_read(SLOT_KINDS_RVA + slot as usize * 4);
            game_write(KIND_RVA, r.eax);
            game_write(SLOT_RVA, slot);
            load(r)?;
        }
        r.ecx = SLOTS;
        let cursor: u32 = game_read(FACE_CURSOR_RVA);
        game_write(
            USED_RVA,
            [game_read::<u32>(POINT_COUNT_RVA).wrapping_mul(12), cursor],
        );
        r.eax = cursor;
        Ok(())
    }
}

/// With the Rust versions of its two callees, for Rust callers.
pub unsafe fn load_front_end_objects_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        load_front_end_objects(
            r,
            &mut |r| {
                reset_objects()?.apply(r);
                Ok(())
            },
            &mut |r| load_object_in_game(r),
        )
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    let done = unsafe {
        load_front_end_objects(
            r,
            &mut |r| {
                reset_objects()?.apply(r);
                Ok(())
            },
            &mut |r| load_object_in_game(r),
        )
    };
    if let Err(e) = done {
        *r = entry;
        return Err(e);
    }
    Ok(())
}
