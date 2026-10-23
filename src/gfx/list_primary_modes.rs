//! `gfx::list_primary_modes` (0x4043DA): the primary card's screen modes listed into the
//! modes' table and marked for Direct3D. (`gfx::list_secondary_modes` is the same for the
//! secondary card; the two share what is here.)
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! The primary: `sub_4048EC` with eax 0 (DirectDraw made for the card, "VID: Open ddraw";
//! failing, fatal errors 0x80208511 / 0x80208512, raised there), `sub_404A31` with eax
//! 0x1000 (DirectDraw's `EnumDisplayModes` with `sub_404A56`, which puts the modes in the
//! table), `gfx::mark_3d_modes` with eax 0x1000 if `dword_5AB4AC` (a 3D primary card) is
//! 1, then `sub_404935` (DirectDraw closed). The DirectDraw calls are the platform's, through the game; in the standalone program
//! they do nothing (its screen modes are its platform's: `standalone.rs`). One caller,
//! `start` (0x401098). Manual patch with a private copy (0x27 bytes, the four calls
//! relocated): a call starts at +3. Shadow mode runs the original alone (DirectDraw is
//! opened once). Self-test: a copy whose variables are the test's and whose four calls go
//! to stand-ins noting eax, against the Rust version, both switches.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::gfx::mark_3d_modes::{ModeVars, mark_3d_modes};

// ---- game data ----

pub const PRIMARY: u32 = 0x1000;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct CardVars {
    /// dword_5AB4AC: a 3D primary card
    pub primary_3d: u32,
    /// dword_5AB4B0: a 3D secondary card
    pub secondary_3d: u32,
    /// unk_5AB4B4: the secondary card's GUID
    pub secondary_guid: u32,
}

impl CardVars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            primary_3d: a(0x5AB4AC),
            secondary_3d: a(0x5AB4B0),
            secondary_guid: a(0x5AB4B4),
        }
    }
}

// ---- Rust version ----

pub unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// Each call with the entry's registers and eax as given (the callees keep them all).
pub fn call(entry: &Regs, place: u32, eax: u32, part: Part) -> Result<(), Refused> {
    part(place, &mut Regs { eax, ..*entry })
}

pub unsafe fn list_primary_modes(entry: &Regs, v: &CardVars, part: Part) -> Result<(), Refused> {
    call(entry, 0, 0, part)?;
    call(entry, 1, PRIMARY, part)?;
    if unsafe { get(v.primary_3d) } == 1 {
        call(entry, 2, PRIMARY, part)?;
    }
    call(entry, 3, PRIMARY, part)
}

/// In the game: DirectDraw through the game, the marking from the Rust version; in the
/// standalone program DirectDraw does nothing.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 2 {
        return unsafe {
            mark_3d_modes(
                r,
                &ModeVars::game(),
                &mut crate::gfx::mark_3d_modes::game_part,
            )
        };
    }
    if place == 1 {
        unsafe { standalone_modes(r.eax) };
    }
    Ok(())
}

const MODE_TABLE_RVA: usize = rva(0x4F4F3C); // g_screenModeTable: 8 bytes an entry
const MODE_COUNT_RVA: usize = rva(0x4F4F38); // g_screenModeCount
const SIZES_RVA: usize = rva(0x4F4E50); // g_screenSizes: 8 of (bit, width, height)
const PIXEL_KINDS_RVA: usize = rva(0x4F4E40); // g_pixelKinds: (0x200, 16), (0x800, 32)
const TABLE_ROOM: u32 = 0x20;

/// The standalone program's DirectDraw `EnumDisplayModes` for card `card` (0x1000 the
/// primary): every screen size the game knows in both pixel kinds (its window takes
/// any), entered as `gfx::mode_callback` (0x404A56) enters a mode: the card's, the kind's
/// and the size's bits, once each, while the table has room.
pub unsafe fn standalone_modes(card: u32) {
    unsafe {
        let table = game_addr(MODE_TABLE_RVA) as *mut u32;
        for size in 0..8usize {
            let size_bit: u32 = game_read(SIZES_RVA + 8 * size);
            for kind in 0..2usize {
                let kind_bit: u32 = game_read(PIXEL_KINDS_RVA + 8 * kind);
                let mode = card | kind_bit | size_bit;
                let count: u32 = game_read(MODE_COUNT_RVA);
                if (0..count as usize).any(|n| table.add(2 * n).read_unaligned() & mode == mode) {
                    continue;
                }
                if count >= TABLE_ROOM {
                    return;
                }
                table.add(2 * count as usize).write_unaligned(mode);
                game_write(MODE_COUNT_RVA, count + 1);
            }
        }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { list_primary_modes(&entry, &CardVars::game(), &mut game_part) }?;
    Ok(())
}
