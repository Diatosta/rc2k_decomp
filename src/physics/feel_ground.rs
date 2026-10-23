//! `physics::feel_ground` (0x43F1A8): the work car's ground this step:
//! `physics::note_surface_mode`, `physics::probe_ground`, `physics::lowest_body_point`;
//! then which wheels are off the ground: `dword_71A220` 1 when the first two (the qword
//! at 0x71A5D8) are both 0, `dword_71A224` the same for the last two (0x71A5E0),
//! `dword_71A22C` 1 when all four are; `dword_71A228` the driven ones' (the first pair's,
//! or all four's with `g_carDriveKind` not 0), and `dword_71A1C0` 0 when they are not off;
//! `dword_6141DC` the word at +0x26 of the face under probe point 4 (`dword_71ADD8`). A
//! reading: the wheels in the air, the ground's kind.
//!
//! In:  nothing
//! Out: eax the face's word, ebx the face, esi its address; ecx, edx as `probe_ground` /
//!      `lowest_body_point` leave them; no flags read
//!
//! It begins with a call: `.private_copy` (its three calls relocated) with
//! `.manual_patch`.
//! Callers: `sub_43B635` (0x43B6F6, 0x43B860, 0x43B8FB), `sub_45D737`, `sub_45D7B6` (twice),
//! `sub_45E3BF`, `sub_45EFF3`.
//! Refuses (before anything) more than 0x40 probe points; `probe_ground`'s refusals
//! mid-way (it puts back what it wrote; `dword_71A6C8` is the original's to write again).
//! Self-test: in a race only: a copy of the original (its calls the game's) against the
//! Rust version on the game's own state with the work car's switches by lot; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDX, ESI, Regs, game_addr, game_read, game_write, rva};
use crate::physics::ground::Ground;
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};
use rc2k_core::physics::walls::WheelsOff;

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

/// The Rust version; `r` as the original leaves it.
// Converted: `rc2k_core::physics::walls::WheelsOff`.
pub unsafe fn feel_ground(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if g(0x71A238) > rc2k_core::physics::walls::PROBE_ROOM {
            return Err(BadInput("more than 0x40 probe points").into());
        }
        crate::physics::note_surface_mode::note_surface_mode();
        crate::physics::probe_ground::probe_ground(r)?;
        crate::physics::lowest_body_point::lowest_body_point(r)?;
        let off = WheelsOff::of([g(0x71A5D8), g(0x71A5DC), g(0x71A5E0), g(0x71A5E4)]);
        p(0x71A220, u32::from(off.front));
        p(0x71A224, u32::from(off.back));
        p(0x71A22C, u32::from(off.all));
        r.eax = u32::from(off.driven(g(0x71A018)));
        p(0x71A228, r.eax);
        if r.eax == 0 {
            p(0x71A1C0, 0);
        }
        r.ebx = g(0x71ADD8);
        r.esi = (r.ebx << 6).wrapping_add(g(0x5CCCBC));
        r.eax = u32::from(((r.esi + 0x26) as *const u16).read_unaligned());
        p(0x6141DC, r.eax);
        Ok(())
    }
}

fn left_of(g: &Regs) -> Left {
    Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx)
        .with(EDX, g.edx)
        .with(ESI, g.esi)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut gr = *r;
    unsafe { feel_ground(&mut gr) }?;
    Ok(left_of(&gr))
}

/// Everything it writes.
pub fn regions() -> Vec<Region> {
    let a = |x: u32| game_addr(rva(x as usize)) as u32;
    let mut r = crate::physics::probe_ground::regions(&Ground::game());
    r.extend([
        (a(0x71A6C8), 4),
        (a(0x6D3068), 12),
        (a(0x71A220), 0x10),
        (a(0x71A1C0), 4),
        (a(0x6141DC), 4),
    ]);
    r
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
