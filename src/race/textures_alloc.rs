//! `race::textures_alloc` (0x443390): `call scene::set_up_textures; retn`.
//!
//! In:  nothing
//! Out: as `scene::set_up_textures` (carry clear; eax and ebx as `gfx::allocate_textures`
//!      leaves them)
//! Callers: `race::load_stage` (0x444480) and 0x474F73. In the game the Rust
//! `set_up_textures` with its two calls as its hook makes them (the catalogue in Rust,
//! `gfx::allocate_textures` 0x4051ED, Direct3D, through the game); its refusal (the
//! fatal error 0x80600943) is passed on.
//! A call in its first five bytes: private copy (6 bytes, the call relocated), manual
//! patch.
//! Shadow mode runs the original alone (textures are allocated).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::sound::turn_parts::{Part, both, copy_of, dispatch_turn, run_copy};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::textures_alloc", rva(0x443390), thunk, &ORIGINAL)
    .private_copy(6)
    .relocate_calls(&[0])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SET_UP: u32 = 0x44E7A5;
const ENTRY: [u8; 6] = [0xE8, 0x10, 0xB4, 0x00, 0x00, 0xC3];

/// `scene::set_up_textures` in the game.
fn set_up_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        crate::scene::set_up_textures::set_up_textures(
            r,
            &mut |_| crate::scene::build_texture_catalogue::build_texture_catalogue_in_game(),
            &mut |r| {
                call_game(r, game_addr(rva(0x4051ED)));
                !r.carry()
            },
        )?;
    }
    r.set_carry(false);
    Ok(())
}

// ---- Rust version ----

pub fn textures_alloc_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    part(SET_UP, r)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, textures_alloc_game, None);
}

fn textures_alloc_game(r: &mut Regs, _: Part) -> Result<(), Refused> {
    let entry = *r;
    textures_alloc_in(r, &mut |_, r| set_up_in_game(r)).inspect_err(|_| *r = entry)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x443390, 6, &ENTRY, &[0], &[SET_UP]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    both(
        &mut t,
        "the call",
        &run_copy(&copy),
        Regs::SENTINEL,
        &|| {},
        &[],
        textures_alloc_in,
        false,
    );
    t.finish()
}
