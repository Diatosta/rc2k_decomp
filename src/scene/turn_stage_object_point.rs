//! `scene::turn_stage_object_point` (0x426A5A): for the loading kinds 0x190 .. 0x257
//! (`g_loadingKind`), a stage object's point turned a quarter and shifted: eax = ecx,
//! ecx = -eax, ebx less `dword_613FC4`.
//!
//! In:  eax, ecx, ebx the point
//! Out: those three as said (else as they came); everything else preserved
//! One caller, 0x42686F (`scene::load_stage_objects`'s point loop). Shadow mode: both,
//! compared.
//! Self-test: the original against the Rust version for loading kinds around the bounds
//! (the game's variable, put back).
//!
//! Converted 2026-10-21: `rc2k_core::scene::stage_objects::turn_point`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, Regs, game_read, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// g_loadingKind, dword_613FC4.
const KIND_RVA: usize = rva(0x634FE8);
const SHIFT_RVA: usize = rva(0x613FC4);

// ---- Rust version ----

/// (eax, ecx, ebx), or None when the kind leaves them.
// Converted: `rc2k_core::scene::stage_objects::turn_point`
pub use rc2k_core::scene::stage_objects::turn_point as turn_stage_object_point;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (kind, shift) = unsafe { (game_read(KIND_RVA), game_read(SHIFT_RVA)) };
    Ok(
        match turn_stage_object_point(kind, shift, (r.eax, r.ecx, r.ebx)) {
            Some((a, c, b)) => Left::nothing().with(EAX, a).with(ECX, c).with(EBX, b),
            None => Left::nothing(),
        },
    )
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
