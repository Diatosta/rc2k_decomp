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
//! Callers: `sub_43B635` (0x43B6F6, 0x43B860, 0x43B8FB), `sub_45D737`, `sub_45D7B6`
//! (twice), `sub_45E3BF`, `sub_45EFF3`. Refuses (before anything) more than 0x40 probe
//! points; `probe_ground`'s refusals mid-way (it puts back what it wrote; `dword_71A6C8` is
//! the original's to write again).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use rc2k_core::physics::walls::WheelsOff;

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

/// The Rust version. The registers it leaves (eax the face's word +0x26, ebx the face,
/// esi its address) are read by none of its callers.
// Converted: `rc2k_core::physics::walls::WheelsOff`.
pub unsafe fn feel_ground() -> Result<(), Refused> {
    unsafe {
        if g(0x71A238) > rc2k_core::physics::walls::PROBE_ROOM {
            return Err(BadInput("more than 0x40 probe points").into());
        }
        crate::physics::note_surface_mode::note_surface_mode();
        crate::physics::probe_ground::probe_ground()?;
        crate::physics::lowest_body_point::lowest_body_point()?;
        let off = WheelsOff::of([g(0x71A5D8), g(0x71A5DC), g(0x71A5E0), g(0x71A5E4)]);
        p(0x71A220, u32::from(off.front));
        p(0x71A224, u32::from(off.back));
        p(0x71A22C, u32::from(off.all));
        let driven = u32::from(off.driven(g(0x71A018)));
        p(0x71A228, driven);
        if driven == 0 {
            p(0x71A1C0, 0);
        }
        let face = (g(0x71ADD8) << 6).wrapping_add(g(0x5CCCBC));
        p(
            0x6141DC,
            u32::from(((face + 0x26) as *const u16).read_unaligned()),
        );
        Ok(())
    }
}
