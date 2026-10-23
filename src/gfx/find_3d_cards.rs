//! `gfx::find_3d_cards` (0x404270): which display cards can draw in 3D.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! `dword_5AB4AC` (a 3D primary card) and `dword_5AB4B0` (a 3D secondary card) 0; unless
//! `dword_5AB94C` (a setting) is not 0: "Finding 3d hardware" (log level 1, a line), then
//! `DirectDrawEnumerateA(Callback, 0)`: the game's `Callback` (0x4042AE) makes DirectDraw
//! for each card, logs " dd:" its description and name, and for one with 3D ("3d:") sets
//! "primary card" (`dword_5AB4AC` 1, unless `dword_5AB950`) or "secondary card"
//! (`dword_5AB4B0` 1 and its GUID to `unk_5AB4B4`, unless `dword_5AB954`). Then a line
//! break in the log. The enumeration (and the callback, DirectDraw throughout) is the
//! platform's: in the game through the import's thunk (0x4D5E96); in the standalone program
//! nothing (no 3D card is found: its 3D is its platform's). One caller, `start` (0x401093).
//! Shadow mode runs the original alone (DirectDraw is made once per card). Self-test: a
//! copy whose three variables are the test's and whose calls go to stand-ins (the
//! enumeration's notes its arguments), against the Rust version, the setting 0 and not.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

// ---- game data ----

const FINDING_TEXT_RVA: usize = rva(0x5C2CF7); // "Finding 3d hardware"
/// The calls by place: 0 `logfile::log_write_l1_nl` (eax), 1 the enumeration, 2
/// `logfile::log_newline_l1`.
const CALLEES: [usize; 2] = [0x40142B, 0x4013DE];

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct FindVars {
    pub primary_3d: u32,
    pub secondary_3d: u32,
    /// dword_5AB94C: not 0, no search
    pub no_search: u32,
}

impl FindVars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            primary_3d: a(0x5AB4AC),
            secondary_3d: a(0x5AB4B0),
            no_search: a(0x5AB94C),
        }
    }
}

// ---- Rust version ----

pub unsafe fn find_3d_cards(entry: &Regs, v: &FindVars, part: Part) -> Result<(), Refused> {
    unsafe {
        (v.primary_3d as *mut u32).write_unaligned(0);
        (v.secondary_3d as *mut u32).write_unaligned(0);
        if (v.no_search as *const u32).read_unaligned() == 0 {
            part(
                0,
                &mut Regs {
                    eax: game_addr(FINDING_TEXT_RVA) as u32,
                    ..*entry
                },
            )?;
            let mut r = *entry;
            part(1, &mut r)?;
        }
        let mut r = *entry;
        part(2, &mut r)
    }
}

/// In the game: the logs and the enumeration through the game; in the standalone program
/// the enumeration does nothing.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        // DirectDraw's drivers: the standalone program's is its rasteriser, a 3D primary
        // card (as the enumeration's callback sets it, unless `dword_5AB950` says not to).
        1 => unsafe {
            if game_read::<u32>(rva(0x5AB950)) == 0 {
                game_write(rva(0x5AB4AC), 1u32);
            }
        },
        _ => crate::calls::call(rva(CALLEES[(place / 2) as usize]), r),
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { find_3d_cards(&entry, &FindVars::game(), &mut game_part) }?;
    Ok(())
}
