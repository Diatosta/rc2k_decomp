//! `menu::show_car_specs` (0x46FD31): the car specifications screen for the kind of event
//! (`dword_755954`): 1 .. 3 the list `off_75A042`, 4 `off_759CAE`, each through
//! `menu::secret_car_list` (with cheat bit 1 `off_75A412` instead), then
//! `menu::car_specs_47af4c_loop` and, unless it was left (`dword_758806` 1),
//! `take_shown_car`; 5 `off_759D16` and the screen alone. Another kind is the original's
//! fatal error 0x80713214, "No GameMode Defined".
//!
//! In:  nothing
//! Out: the registers as its callees leave them (not `pusha`'d); esi the list
//! An item action (a reading: the main menu's "car specifications"). Its calls through
//! `part`: the list's choice and the car's taking in Rust, the screen at the game's
//! address (its hook chooses). Ungated thunk (a screen); shadow mode runs the original
//! alone. Self-test: a copy whose calls go to stand-ins (the screen answering
//! `dword_758806`), against the Rust version, kinds 1 .. 5 (not the fatal one: the copy
//! has no `pusha` for `fatal_landing`).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{FatalError, Refused};
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::menu::parts::{Part, parted_entry};

const CALLEES: [usize; 3] = [
    0x480D35, // 0 menu::secret_car_list
    0x47AF4C, // 1 menu::car_specs_47af4c_loop
    0x47B29D, // 2 menu::take_shown_car
];
const KIND_RVA: usize = rva(0x755954); // dword_755954
const LEFT_RVA: usize = rva(0x758806); // dword_758806

/// The Rust version: the registers it leaves.
pub unsafe fn show_car_specs(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::cars::specs_list`.
    use rc2k_core::menu::cars::{SpecsList, specs_list};
    let mut r = *entry;
    let kind: u32 = unsafe { game_read(KIND_RVA) };
    let Some((list, choice)) = specs_list(kind) else {
        return Err(FatalError {
            code: 0x8071_3214,
            message: "No GameMode Defined",
        }
        .into());
    };
    r.esi = game_addr(rva(match list {
        SpecsList::Events => 0x75A042,
        SpecsList::Kind4 => 0x759CAE,
        SpecsList::Kind5 => 0x759D16,
    })) as u32;
    if !choice {
        part(1, &mut r)?;
        return Ok(r);
    }
    part(0, &mut r)?;
    part(1, &mut r)?;
    if unsafe { game_read::<u32>(LEFT_RVA) } != 1 {
        part(2, &mut r)?;
    }
    Ok(r)
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        0 => crate::menu::arcade_steps::secret_car_list().apply(r),
        2 => unsafe { crate::menu::take_shown_car::take_shown_car(r.esi) }?,
        _ => crate::calls::call(rva(CALLEES[place as usize]), r),
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    parted_entry(r, &|r, part| unsafe { show_car_specs(r, part) }, game_part)
}
