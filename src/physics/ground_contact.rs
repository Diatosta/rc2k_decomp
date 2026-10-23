//! `physics::ground_contact` (0x45EFF3): the deepest probe point against the ground.
//! `dword_6D3028` = 0; `physics::lowest_body_point`; nothing more unless the depth
//! (`dword_6D3068`) is below 0x1000 and, with `unk_71A22C` at 0, below 0 (each by the
//! sign of the `cmp`). Then `dword_6D3028` = 1; for the point n (`dword_6D306C`) its
//! three floats from `unk_71AAC8[n]`, `unk_71ABC8[n]`, `unk_71ACC8[n]` into `unk_71A1FC`
//! ..; the contact taken (`physics::crash_contact`'s `take_contact`); the slide limit
//! `flt_6D3100` = 0; `physics::contact_force` into contact record 2 (`g_contactRecords`
//! +0x40), `physics::place_sound_e1`; `unk_71A208` raised to |`flt_6D3108`|; the force
//! into `dword_6D3018` ..; with the depth below 0, `physics::restore_saved_car` and
//! `physics::feel_ground`.
//!
//! In:  nothing
//! Out: the registers as `lowest_body_point` and, on the long path, `contact_force`,
//!      `restore_saved_car` and `feel_ground` leave them, eax the depth; no flags read
//!      (the caller calls on)
//!
//! Caller: `sub_43B635`, every physics step.
//! Refuses a point past 0x3F on the long path (the tables' room), and what its calls
//! refuse; what was written is put back.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with test faces (`g_courseFaces` pointed at them for the test): depths 0 ..
//! 0x1800, the contact's figures and the matrices by lot, both paths, the current car not
//! listed (no sound), under three precisions. Not reached: a depth below 0
//! (`restore_saved_car` and `feel_ground` need the stage; their own tests cover them).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::contact_force::{self, RECORD, contact_force};
use crate::physics::crash_contact::{point_number, sign, sound_regions, take_contact};
use crate::physics::feel_ground::feel_ground;
use crate::physics::flow::{a, copy3, g, guard, p};
use crate::physics::lowest_body_point::lowest_body_point;
use crate::physics::place_sound_e1::place_sound_e1;
use crate::physics::restore_saved_car::restore_saved_car;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};

/// Contact record 2 (`g_contactRecords` + 0x40).
const FORCE: usize = 0x614A90;

// Converted: `rc2k_core::physics::contacts::touches` and `walls::raised_hit` (the x87 steps stay).
fn steps(r: &mut Regs) -> Result<(), Refused> {
    p(0x6D3028, 0);
    unsafe { lowest_body_point(r) }?;
    let depth = g(0x6D3068);
    r.eax = depth;
    let limit = rc2k_core::physics::contacts::GROUND_LIMIT;
    if !rc2k_core::physics::contacts::touches(depth, limit, g(0x71A22C) != 0) {
        return Ok(());
    }
    let n = point_number()?;
    r.ebx = n as u32;
    p(0x6D3028, 1);
    p(0x71A1FC, g(0x71AAC8 + 4 * n));
    p(0x71A200, g(0x71ABC8 + 4 * n));
    p(0x71A204, g(0x71ACC8 + 4 * n));
    take_contact(n);
    p(0x6D3100, 0);
    p(RECORD, a(FORCE));
    unsafe { contact_force() }.apply(r);
    unsafe { place_sound_e1() }?;
    if let Some(hit) = rc2k_core::physics::walls::raised_hit(
        f32::from_bits(g(0x6D3108)),
        f32::from_bits(g(0x71A208)),
    ) {
        p(0x71A208, hit.to_bits());
    }
    copy3(0x6D3018, FORCE);
    r.eax = g(0x6D3068);
    if sign(r.eax) {
        unsafe { restore_saved_car() }?.apply(r);
        unsafe { feel_ground(r) }?;
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn ground_contact(entry: &Regs) -> Result<Left, Refused> {
    guard(&regions(), || {
        let mut r = *entry;
        steps(&mut r)?;
        Ok(left_of(&r))
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { ground_contact(r) }
}

fn regions() -> Vec<Region> {
    let mut all = contact_force::regions();
    all.extend([
        (a(0x6D3028), 4),
        (a(0x6D3068), 12),
        (a(0x71A1FC), 0x10),
        (a(FORCE), 0x18),
        (a(0x6D3018), 12),
        (a(RECORD), 4),
        // `restore_saved_car`'s
        (a(0x614738), 4),
        (a(0x71A10C), 12),
        (a(0x71A08C), 0x40),
        (a(0x71ADC8), 0x100),
    ]);
    all.extend(crate::physics::feel_ground::regions());
    all.extend(sound_regions());
    all
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
