//! `physics::wall_contacts` (0x45D7B6): the probe points (`unk_71A238` of them) against
//! walls. For each point k, its face now (`unk_71ADC8[k]`, into `dword_6D3174`) and before
//! (`dword_6D2814[k]`, into `dword_6D3170` when they differ): their words +2 or-ed
//! together; when they differ and the pair's has both 0x400 and 0x800, the wall met:
//! `dword_6D3214` = k, the point's three floats from `unk_71AAC8[k]` .. into
//! `unk_71A1FC` .., `physics::wall_edge_contact`, `unk_71A208` raised to |`flt_6D3108`|,
//! `unk_71A20C` = 1, `unk_71A1CC` = 0x32, `physics::restore_saved_car`,
//! `physics::feel_ground`. None met: `dword_6D3174` = `dword_6D3170`, the faces' words
//! or-ed & 0xC00 into `dword_6D3038`; at 0xC00, the car-frame speed (`unk_71A17C` ..)
//! into `unk_71A1FC` .., `physics::wall_push_back`, the same two words,
//! `restore_saved_car`, `feel_ground`.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them (ebp the point, or -1 after the
//!      loop); no flags read (callers go on)
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`.
//! Refuses (before anything) a count of 0 or past 0x40 (the original's loop runs on, or
//! past the tables), and what its calls refuse (what was written put back).
//! Self-test (`test_walls`, in a race; skipped otherwise): the original and the Rust
//! version on the stage's own faces (everything written saved and put back): probe
//! lists of faces chosen by their wall bits, none met, a wall met, the 0xC00 fallback.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{g, p};
use crate::physics::touch_face::FACES;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::walls;

/// `g_workVector`.
pub const WORK: usize = 0x5C5F60;
/// Contact record 4 (`g_contactRecords` + 0x80).
pub const WALL_RECORD: usize = 0x614AD0;

fn word(face: u32) -> u32 {
    let at = (face << 6).wrapping_add(g(FACES)) + 2;
    unsafe { (at as *const u16).read_unaligned() as u32 }
}

fn after(r: &mut Regs) -> Result<(), Refused> {
    p(0x71A20C, 1);
    p(0x71A1CC, 0x32);
    unsafe { crate::physics::restore_saved_car::restore_saved_car() }?.apply(r);
    unsafe { crate::physics::feel_ground::feel_ground(r) }
}

/// The steps, not guarded (`check` before).
// Converted: `rc2k_core::physics::walls::walk`.
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    r.edx = 0;
    r.ebp = 0;
    let walked = walls::walk(g(0x71A238), |k| {
        r.ebp = k;
        let k = k as usize;
        r.ebx = g(0x71ADC8 + 4 * k);
        p(0x6D3174, r.ebx);
        r.esi = (r.ebx << 6).wrapping_add(g(FACES));
        let now_word = word(r.ebx);
        r.eax = now_word;
        let before = g(0x6D2814 + 4 * k);
        let before_word = (r.ebx != before).then(|| {
            p(0x6D3170, before);
            r.esi = (before << 6).wrapping_add(g(FACES));
            r.ebx = word(before);
            r.eax = (now_word | r.ebx) & 0xC00;
            r.ebx
        });
        Ok::<_, Refused>(walls::ProbeFaces {
            now_word,
            before_word,
        })
    })?;
    match walked {
        walls::Walk::Met { probe, words } => {
            // 0x45D83E: a wall met.
            r.edx = words;
            let k = probe as usize;
            p(0x6D3214, r.ebp);
            p(0x71A1FC, g(0x71AAC8 + 4 * k));
            p(0x71A200, g(0x71ABC8 + 4 * k));
            p(0x71A204, g(0x71ACC8 + 4 * k));
            crate::physics::wall_edge_contact::steps(r)?;
            if let Some(hit) = rc2k_core::physics::walls::raised_hit(
                f32::from_bits(g(0x6D3108)),
                f32::from_bits(g(0x71A208)),
            ) {
                p(0x71A208, hit.to_bits());
            }
            after(r)
        }
        walls::Walk::Clear { bits } => {
            r.ebp = u32::MAX;
            r.eax = g(0x6D3170);
            p(0x6D3174, r.eax);
            r.edx = bits;
            p(0x6D3038, r.edx);
            if r.edx != walls::WALL_BITS {
                return Ok(());
            }
            p(0x71A1FC, g(0x71A17C));
            p(0x71A200, g(0x71A180));
            p(0x71A204, g(0x71A184));
            crate::physics::wall_push_back::steps(r)?;
            after(r)
        }
    }
}

/// The probe count checked: 0 or past 0x40 refused.
pub fn check() -> Result<(), Refused> {
    Ok(walls::check_probe_count(g(0x71A238))?)
}

/// The Rust version.
pub unsafe fn wall_contacts(entry: &Regs) -> Result<Left, Refused> {
    check()?;
    {
        let mut r = *entry;
        steps(&mut r)?;
        Ok(left_of(&r))
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { wall_contacts(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
