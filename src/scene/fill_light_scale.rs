//! `scene::fill_light_scale` (0x44F1BC): a table of a colour from black up to itself.
//!
//! In:  esi = a colour, three dwords, edi = a table of 0x100 dwords
//! Out: eax = the last part worked out, ebx = 0x100, ecx = the last entry, edx = 0,
//!      ebp = 0xFF; esi and edi preserved
//!
//! Entry n (0 .. 0xFF) = each of the colour's three parts times n over 255, packed
//! as `scene::fill_light_ramp` packs them.
//! Eight call sites in `scene::fill_light_tables`.
//! Refuses a null pointer.
//! Self-test: 30 colours by lot into a table of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::scene::light_tables::{self};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.esi == 0 || r.edi == 0 {
        return Err(BadInput("a null colour or table").into());
    }
    Ok(unsafe { light_tables::fill_light_scale(r.esi as *const _, r.edi as *mut _) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
