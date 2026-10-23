//! `race::draw_flare` (0x455189) and the shared test of `race::draw_lamp_flare`: a flare
//! from the flare tables (or a lamp) placed by `race::place_object_matrix` and drawn
//! (`scene::draw_scene_object`), see `race/frame_draw.rs`.
//!
//! In:  ebx = the flare
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `race::draw_list_entry` (flares 0x1000000 ..). Shadow mode runs the original
//! alone (it draws). Self-test: a copy whose drawing call goes to a stand-in that notes
//! eax .. ecx (its matrix call real), against the Rust version on the same; flares of the
//! test's in the tables; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, rva};
use crate::race::frame_draw::*;
use crate::race::level_init::{note, take_called};
use crate::race::place_object_matrix::object_matrix_regions;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_flare(r, &mut entry_part) }?;
    Ok(())
}

/// A copy of `address` whose drawing call (at `call`) goes to stand-in 6.
pub fn copy_with_draw(
    hook: &RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
) -> Result<RedirectedCopy, String> {
    crate::install::stand_ins::set_handler(note);
    crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len,
            entry,
            calls,
            jumps_out: &[],
        },
        &|c| (c == rva(0x451E3E)).then(|| crate::install::stand_ins::stand_in(6)),
        &[],
        0,
    )
}

/// What a flare's or lamp's drawing writes besides the matrices.
pub fn flare_draw_regions() -> Vec<Region> {
    let mut v = object_matrix_regions();
    v.extend([
        (game_addr(rva(0x63501C)) as u32, 0xC),
        at(rva(0x635038)),
        at(rva(0x635054)),
        at(rva(0x6214FC)),
        at(rva(0x6365C8)),
        at(rva(0x62DAF4)),
    ]);
    v
}

/// Runs a copy and the Rust version (`rust`) on the same, `prepare` before each.
pub fn compare_draws(
    t: &mut SelfTest,
    what: &str,
    copy: &RedirectedCopy,
    entry: Regs,
    prepare: &dyn Fn(),
    regions: &[Region],
    rust: &dyn Fn(&Regs) -> Result<(), crate::errors::Refused>,
) {
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    prepare();
    take_called();
    let mut ro = entry;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = (regs(&ro), take_called(), capture(regions));
    prepare();
    let done = rust(&entry);
    let mine = (regs(&entry), take_called(), capture(regions));
    t.check(done.is_ok() && theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?} {done:?}; memory first differs at {diff:x?}",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
}
