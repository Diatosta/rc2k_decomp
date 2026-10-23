//! `race::draw_skidmarks` (0x458C03): the skidmarks shown in the drawn view, a quad for
//! each link with a link before (see `race/skidmarks.rs`; the x87 runs the original's
//! instructions, the turn by `scene::rotate_point`'s Rust version), each faded
//! (`scene::fog_polygon`), cut by depth and to the screen and drawn in black in mode 3
//! without the depth test (`scene::depth_test_off`, `scene::draw_polygon_in_game`,
//! `scene::depth_test_on`).
//!
//! In:  nothing
//! Out: edx 0x400, eax, ecx, esi as it leaves them (a `retn` follows)
//!
//! Caller: `race::draw_skidmarks_if_on`. Refuses what the scene's functions refuse
//! (mid-way; not met with four vertices). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose depth-test and drawing calls go to stand-ins (the fade and the
//! clips real), against the Rust version on the same; chains of links of the test's in
//! the views and not; all put back (skipped without a race).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_write, rva};
use crate::race::skidmarks::*;
use crate::service::harness::{Region, at};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let after = unsafe { draw_skidmarks(r, &mut skid_part) }?;
        let esp = r.esp;
        *r = after;
        r.esp = esp;
    }
    Ok(())
}

/// What a skidmark test watches and puts back.
pub fn skid_regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x975510, 0x4AF0),
        g(0x623E40, 0x80),
        g(0x6240A4, 0x94),
        at(rva(0x63DAB4)),
        at(rva(0x643AE4)),
        at(rva(0x62400C)),
    ]
}

/// The skidmark inputs a test sets.
pub fn skid_inputs() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x643AE8, 0x2000),
        g(0x645B18, 0x6000),
        at(rva(0x643AD4)),
        at(rva(0x62389C)),
        at(rva(0x63DB9C)),
        at(rva(0x6416B8)),
    ]
}

/// A test's skidmarks: a chain of `n` links (each one's link before the last, views by
/// `views`), points along the road ahead of the view.
pub fn set_skids(n: u32, views: u16, mode: u32) {
    unsafe {
        let links = game_addr(rva(0x643AE8)) as u32;
        let points = game_addr(rva(0x645B18)) as u32;
        for k in 0..0x400u32 {
            ((links + 8 * k) as *mut i16).write_unaligned(-1);
        }
        let view = game_addr(rva(0x606298)) as u32;
        let (vx, vy, vz) = (
            (view as *const u32).read_unaligned(),
            ((view + 4) as *const u32).read_unaligned(),
            ((view + 8) as *const u32).read_unaligned(),
        );
        for k in 0..n {
            ((links + 8 * k) as *mut i16).write_unaligned(if k == 0 { -1 } else { k as i16 - 1 });
            ((links + 8 * k + 6) as *mut u16).write_unaligned(views ^ (k as u16 & 1));
            for s in 0..2u32 {
                let p = points + 0xC * (2 * k + s);
                (p as *mut u32).write_unaligned(vx.wrapping_add(0x8000 * s).wrapping_sub(0x4000));
                ((p + 4) as *mut u32).write_unaligned(vy.wrapping_sub(0x1_0000));
                ((p + 8) as *mut u32)
                    .write_unaligned(vz.wrapping_add(0x2_0000 * k).wrapping_sub(0x4_0000));
            }
        }
        game_write(rva(0x643AD4), 64.0f32.to_bits());
        game_write(rva(0x62389C), mode);
        game_write(rva(0x63DB9C), 1u32);
        game_write(rva(0x6416B8), 0u32);
    }
}
