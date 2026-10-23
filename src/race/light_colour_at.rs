//! `race::light_colour_at` (0x4537CE): the light at the lit place (`g_litPlace`,
//! `dword_63658C`, `dword_636590`) into `g_lightColour`: 0, then the cars' headlights
//! (`sub_44F601`, each car by `sub_44F691`: the place turned into the car's light frame
//! (+0x9CC, `fistp`'d into `dword_6365B8` ..), a spread by distance and side, times +0xABC,
//! a colour from `g_carLightTable`) and the lamps (`sub_44F863`: by `scene::approximate_distance`,
//! times the lamp's strength, a grey times its colour), each added byte by byte (held to
//! 0xFF; three bytes without MMX, four with, as the original's two paths).
//!
//! In:  nothing (the place)
//! Out: the registers as it leaves them (ecx the cars, esi, eax, ebx, edx as the last car
//!      and lamp left them)
//!
//! `sub_44F601`, `sub_44F691` and `sub_44F863` are covered here (their only callers are in
//! this tree). The car's second spread (from z' - 0x100000) is worked out and not used, as
//! the original (`notes/bugs.md`). See `race/frame_draw.rs`. Callers: `race::view_light`,
//! 0x4537B3, 0x453A35, 0x453C05, 0x453DD5. Refuses (before anything) no race cars or more
//! than eight, a null block, more than 16 lamps. Shadow mode: by state. Self-test: the
//! game's blocks given lights, places and frames by pattern, test lamps (points of the
//! test's), with and without MMX, the modes and player counts; all put back (skipped with a
//! null block).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { light_colour_at(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
