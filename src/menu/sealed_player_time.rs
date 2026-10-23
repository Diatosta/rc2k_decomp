//! `menu::sealed_player_time` (0x4A3D38): the current player's (`dword_75598C`) time
//! (+0x88) sealed (`records::seal_time`) with bl its +8, cl `dword_75ACD8` and dl eax's low
//! byte; the result also into `dword_7559EC` and `hhmm`.
//!
//! In:  eax (its low byte sealed with the time)
//! Out: eax, ebx the sealed block; the rest kept
//!
//! Callers: `sub_489F76`, `sub_48C8B6`, `menu::enter_record_entry`. The seal is its Rust
//! version. Refuses a current player past 3 or missing, before anything. Self-test: the
//! original and the Rust version on the game's player, times by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const SLOT_RVA: usize = rva(0x7559EC); // dword_7559EC
const HHMM_RVA: usize = rva(0x7559F0); // hhmm
const SEAL_BYTE_RVA: usize = rva(0x75ACD8); // dword_75ACD8
/// The Rust version (see the header): `byte` the original's eax (its low byte sealed);
/// the sealed block's two dwords (its eax, ebx) out.
pub unsafe fn sealed_player_time(byte: u32) -> Result<[u32; 2], Refused> {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    let p = player(n);
    let sealed = unsafe {
        crate::records::seal_time::seal_time(
            get(p + 0x88),
            [
                get(p + 8) as u8,
                game_read::<u32>(SEAL_BYTE_RVA) as u8,
                byte as u8,
            ],
        )
    };
    unsafe {
        game_write(SLOT_RVA, sealed[0]);
        game_write(HHMM_RVA, sealed[1]);
    }
    Ok(sealed)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [eax, ebx] = unsafe { sealed_player_time(r.eax) }?;
    Ok(left_of(&Regs { eax, ebx, ..*r }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
