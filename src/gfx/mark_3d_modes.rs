//! `gfx::mark_3d_modes` (0x40442C): the screen modes Direct3D can draw in, marked in the
//! modes' table.
//!
//! In:  eax = the kind of card the modes were listed for (0x1000 the primary, 0x2000 the
//!      secondary: `gfx::list_primary_modes`, `gfx::list_secondary_modes`)
//! Out: every register preserved (`pusha` .. `popa`)
//! Nothing unless `g_hasFpu` (0x5183EC, set by `system::test_processor`) is 1.
//! `sub_419853` (through `part`: Direct3D's devices looked at, "D3D: detect card", the
//! depth formats into `dword_4F492C` and `dword_4F4930`; the carry: none fit) the
//! platform's, through the game; in the standalone program it says none fit. Then over the
//! table `dword_4F4F3C` (`dword_4F4F38` entries of 8 bytes: flags, a word of extra flags):
//! the next entry with all of eax's bits (`system::find_entry_with_bits`), from there the
//! next with any of `dword_4F492C`'s (`system::find_entry_with_any_bit`); that one gets
//! 0x100000 and, with `dword_4F4930` not 0, 0x200000 and `dword_4F4930` in its second
//! dword; one entry on (`system::next_entry`, its carry not looked at), again, until a
//! search finds nothing. Refuses (before anything) more than 0x20 entries, the table's
//! room (`sub_404A56` stops there): past it the original would mark what lies after.
//! Shadow mode runs the original alone (the platform's call is made once). Self-test: a
//! copy whose five variables are the test's and whose Direct3D call goes to a stand-in
//! (which sets the depth formats, or fails), on tables of the test's, against the Rust
//! version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_write, rva, standalone};

// ---- game data ----

/// The call: 0 `sub_419853` (the carry: no Direct3D device fits).
const DETECT_RVA: usize = rva(0x419853);
pub use rc2k_core::gfx::screen_modes::MODES_ROOM;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct ModeVars {
    /// g_hasFpu
    pub has_fpu: u32,
    /// dword_4F4F3C: the table
    pub table: u32,
    /// dword_4F4F38: its entries
    pub count: u32,
    /// dword_4F492C: the depth formats' bits
    pub depth: u32,
    /// dword_4F4930: the extra flags
    pub extra: u32,
}

impl ModeVars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            has_fpu: a(0x5183EC),
            table: a(0x4F4F3C),
            count: a(0x4F4F38),
            depth: a(0x4F492C),
            extra: a(0x4F4930),
        }
    }
}

// ---- Rust version ----

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The marking over the table (after the platform's call).
pub unsafe fn mark_modes(kind: u32, v: &ModeVars) {
    // Converted: `rc2k_core::gfx::screen_modes::mark_3d`.
    unsafe {
        let before: Vec<_> = crate::system::find_entry::entries(v.table, get(v.count)).collect();
        let mut table = before.clone();
        rc2k_core::gfx::screen_modes::mark_3d(&mut table, kind, get(v.depth), get(v.extra));
        for (n, e) in table.into_iter().enumerate() {
            if e != before[n] {
                ((v.table + 8 * n as u32) as *mut rc2k_core::system::tables::Entry)
                    .write_unaligned(e);
            }
        }
    }
}

pub unsafe fn mark_3d_modes(entry: &Regs, v: &ModeVars, part: Part) -> Result<(), Refused> {
    unsafe {
        if get(v.has_fpu) != 1 {
            return Ok(());
        }
        if get(v.count) > MODES_ROOM {
            return Err(BadInput("more screen modes than the table has room for").into());
        }
        let mut r = *entry;
        part(0, &mut r)?;
        if r.carry() {
            return Ok(());
        }
        mark_modes(entry.eax, v);
        Ok(())
    }
}

/// In the game: Direct3D through the game. In the standalone program its rasteriser
/// answers as a device that draws in both pixel kinds with a depth buffer of 16 or 32
/// bits and has every capability the game asks about (`sub_419853`'s results:
/// `g_depthFormatBits`, `g_depthExtraFlags` and the `caps3d..` switches 0x5AB98C ..
/// 0x5AB9AC that the graphics options read).
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    if standalone() {
        unsafe {
            // 16 and 32 bits (`g_pixelKinds`' 0x200 and 0x800) for both
            game_write(rva(0x4F492C), 0xA00u32);
            game_write(rva(0x4F4930), 0xA00u32);
            for at in (0x5AB98C..=0x5AB9AC).step_by(4) {
                game_write(rva(at), 1u32);
            }
        }
        r.set_carry(false);
        return Ok(());
    }
    crate::calls::call(DETECT_RVA, r);
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { mark_3d_modes(&entry, &ModeVars::game(), &mut game_part) }?;
    Ok(())
}
