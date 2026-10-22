//! `race::mud_frame` (0x44A054): each frame drawn, the mud's frame count `dword_623D58`
//! one more; at 0xE it is 0 again and the next of the four mud textures is worked on
//! (`dword_623D5C`, kept to 0 .. 3: `race::mud_texture_turn`, then one more, `and 3`).
//!
//! In:  nothing
//! Out: unchanged until the 14th frame; then the registers as `mud_texture_turn` leaves
//!      them; no flags read (the caller in `sub_45429B` goes on with a `mov`)
//!
//! See `race/mud.rs`. One caller, `sub_45429B` (0x454442). Its call through `part` (0, in
//! the game the Rust version with its own calls). Refuses what `mud_texture_turn`
//! refuses (the counts put back). Shadow mode runs the original alone (the device is
//! told). Self-test: a copy whose call goes to a stand-in, against the Rust version, frame
//! counts below, at and past 0xE, texture numbers 0 .. 5; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::physics::flow::guard;
use crate::race::mud::{copy, run_both, test_part};
use crate::race::muddy_texel::TEXTURE_RVA;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::mud_frame", rva(0x44A054), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FRAMES_RVA: usize = rva(0x623D58); // dword_623D58

/// A call by place: 0 `race::mud_texture_turn`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn mud_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::mud::{count_frame, texture_turn, next_texture}`.
    use rc2k_core::race::mud::{count_frame, next_texture, texture_turn};
    guard(&[at(FRAMES_RVA), at(TEXTURE_RVA)], || unsafe {
        let mut r = *entry;
        let (frames, due) = count_frame(game_read(FRAMES_RVA));
        game_write(FRAMES_RVA, frames);
        if !due {
            return Ok(r);
        }
        game_write(TEXTURE_RVA, texture_turn(game_read(TEXTURE_RVA)));
        part(0, &mut r)?;
        game_write(TEXTURE_RVA, next_texture(game_read(TEXTURE_RVA)));
        Ok(r)
    })
}

/// In the game: `mud_texture_turn`'s Rust version with its own calls.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::mud_texture_turn::{game_part, mud_texture_turn};
    *r = unsafe { mud_texture_turn(r, &mut game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { mud_frame(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { mud_frame(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // inc dword_623D58
    let copy = match copy(
        0x44A054,
        0x33,
        &[0xFF, 0x05, 0x58, 0x3D, 0x62, 0x00],
        &[0x20],
        &[(0x44A087, 0)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [at(FRAMES_RVA), at(TEXTURE_RVA)];
    let saved = capture(&regions);
    for frames in [0u32, 0xC, 0xD, 0xE, 0xFFFF_FFFF] {
        for texture in 0..6u32 {
            run_both(
                &mut t,
                &format!("frames {frames:#x}, texture {texture}"),
                &copy,
                &regions,
                &|| unsafe {
                    game_write(FRAMES_RVA, frames);
                    game_write(TEXTURE_RVA, texture);
                },
                &|r| unsafe { mud_frame(r, &mut test_part) },
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
