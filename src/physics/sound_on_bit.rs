//! `physics::sound_on_bit` (0x441324): when bit cl of `g_car` +0x298 (`unk_71A270`) is
//! set, it is cleared and `physics::place_car_sound` runs.
//!
//! In:  cl the bit
//! Out: eax 1 << cl; the rest kept; no flags read (the callers load floats next)
//!
//! Callers: `sub_4411EC` (four).
//! Refuses what `place_car_sound` refuses (before anything: the bit is cleared after the
//! check).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with a test sound object: bits set and clear.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, game_read, game_write, rva};
use crate::physics::car_sounds::object;
use crate::physics::place_car_sound::place_car_sound;
use crate::screens::harness::Left;
use rc2k_core::physics::parts::take_bit;

const BITS_RVA: usize = rva(0x71A270); // g_car + 0x298

/// The Rust version.
// Converted: `rc2k_core::physics::parts::take_bit`.
pub unsafe fn sound_on_bit(cl: u32) -> Result<Left, Refused> {
    let (bit, cleared) = take_bit(unsafe { game_read(BITS_RVA) }, cl);
    let left = Left::nothing().with(EAX, bit);
    let Some(cleared) = cleared else {
        return Ok(left);
    };
    object()?;
    unsafe { game_write(BITS_RVA, cleared) };
    unsafe { place_car_sound() }?;
    Ok(left)
}
