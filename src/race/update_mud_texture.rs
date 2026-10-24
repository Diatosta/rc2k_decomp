//! `race::update_mud_texture` (0x44A12E): the mud thrown and splashed since the last turn
//! put onto the texture worked on (`dword_623D5C`, n). The texture's slot (kind 0x100's
//! first slot + n) forgotten (`scene::forget_slot_texture`), `dword_6D1508` + 0x10000; the
//! texture (kind 0x100's first texture + n, into `dword_623D54`; none: nothing more)
//! locked (`d3d::lock_texture`, the platform's; carry: nothing more). Then:
//! - the throws: `dword_623D60[n]` (the wheels' thrown mud, a float) rounded (`fistp`)
//!   into `dword_623920`, which read back as a float (sic) and below 0.0 becomes 0.0's
//!   bits: negative counts are 0. Unless the texture's count `dword_623D80[n]` is past
//!   0x60000: held so that its low word plus the throws stays below 0x1000000, taken off
//!   `dword_623D60[n]` (`fisub`), and above 0 that many `race::throw_mud_texel`;
//! - the splashes: the same from `dword_623D70[n]` into `dword_623924`, unless the count
//!   `dword_623D90[n]` has reached 0x1000 (also checked before each): `race::splash_mud`;
//!
//! Then the texture is unlocked (`d3d::unlock_texture`).
//!
//! In:  nothing
//! Out: nothing its caller reads (conversion phase 3, 2026-10-24)
//!
//! See `race/mud.rs`. One caller, `race::mud_texture_turn`. Its calls through `part`: 0
//! the forgetting (eax), 1 the lock (eax; carry), 2 the unlock (eax); in the game the
//! forgetting's Rust version and the platform's two through the game (section 15).
//! Refuses (before anything) a texture past 3 and no mud, and its callees' refusals (what
//! the counts, the throws and the random state were put back). Shadow mode runs the
//! original alone (it tells the device). Self-test: a copy whose three calls go to
//! stand-ins (the lock failing or not), against the Rust version, on the mud tests'
//! set-up with patterns of the test's, throws and splashes of each sign and size, under
//! the three precisions; the surface, the counts, the floats and the random state compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::physics::flow::{c0, x87_run};
use crate::race::muddy_texel::texture_and_mud;
use crate::race::splash_mud::{LAST_SPLASHED, SPLASHED_RVA, splash_mud};
use crate::race::throw_mud_texel::{LAST_THROWN, THROWN_RVA, throw_mud_texel};

const FIRST_SLOT_RVA: usize = rva(0x5D1EE8); // g_kindFirstSlots[0x100]
const FIRST_TEXTURE_RVA: usize = rva(0x5D0EE8); // g_kindFirstTextures[0x100]
const CHANGES_RVA: usize = rva(0x6D1508); // dword_6D1508
const LOCKED_RVA: usize = rva(0x623D54); // dword_623D54: the texture locked
const THROWS_IN_RVA: usize = rva(0x623D60); // dword_623D60: each texture's thrown mud (float)
const SPLASHES_IN_RVA: usize = rva(0x623D70); // dword_623D70: each texture's splashes (float)
const THROWS_RVA: usize = rva(0x623920); // dword_623920
const SPLASHES_RVA: usize = rva(0x623924); // dword_623924
const ZERO_RVA: usize = rva(0x5C5D90); // flt_5C5D90: 0.0

/// `fld [from]; fistp [to]`, then `fld zero; fld [to]; fcompp` and, with C0, `fld zero;
/// fstp [to]`: the units of a float count, negatives 0 (as the original, by bits).
fn whole_units(from: usize, to: usize) -> i32 {
    x87_run!(
        "fld dword ptr [{f}]",
        "fistp dword ptr [{t}]";
        f = game_addr(from), t = game_addr(to)
    );
    let zero: u32 = unsafe { game_read(ZERO_RVA) };
    if c0(unsafe { game_read(to) }, zero) {
        unsafe { game_write(to, zero) };
    }
    unsafe { game_read(to) }
}

/// `fld [at]; fisub [units]; fstp [at]`.
fn take_units(at: usize, units: usize) {
    x87_run!(
        "fld dword ptr [{a}]",
        "fisub dword ptr [{u}]",
        "fstp dword ptr [{a}]";
        a = game_addr(at), u = game_addr(units)
    );
}

/// The Rust version (see the header).
pub unsafe fn update_mud_texture() -> Result<(), Refused> {
    let (n, _) = texture_and_mud()?;
    let k = 4 * n as usize;
    unsafe {
        crate::scene::forget_slot_texture::forget_slot_texture(
            game_read::<u32>(FIRST_SLOT_RVA).wrapping_add(n),
        )?;
        game_write(
            CHANGES_RVA,
            game_read::<u32>(CHANGES_RVA).wrapping_add(0x1_0000),
        );
        let texture = game_read::<u32>(FIRST_TEXTURE_RVA).wrapping_add(n);
        game_write(LOCKED_RVA, texture);
        if texture == 0 {
            return Ok(());
        }
        if crate::d3d::lock_texture::lock_texture(texture)?.carry {
            return Ok(());
        }
        // The throws.
        let throws = whole_units(THROWS_IN_RVA + k, THROWS_RVA);
        let count: u32 = game_read(THROWN_RVA + k);
        if count <= LAST_THROWN {
            let over = (count & 0xFFFF).wrapping_add(throws as u32);
            if (over.wrapping_sub(0x100_0000) as i32) >= 0 {
                let over = over.wrapping_sub(0x100_0000);
                game_write(THROWS_RVA, (throws as u32).wrapping_sub(over));
            }
            take_units(THROWS_IN_RVA + k, THROWS_RVA);
            let throws: i32 = game_read(THROWS_RVA);
            if throws > 0 {
                let mut done = 0u32;
                loop {
                    throw_mud_texel()?;
                    done = done.wrapping_add(1);
                    if done == game_read::<u32>(THROWS_RVA) {
                        break;
                    }
                }
            }
        }
        // The splashes.
        whole_units(SPLASHES_IN_RVA + k, SPLASHES_RVA);
        if game_read::<u32>(SPLASHED_RVA + k) < LAST_SPLASHED {
            take_units(SPLASHES_IN_RVA + k, SPLASHES_RVA);
            let splashes: i32 = game_read(SPLASHES_RVA);
            if splashes > 0 {
                let mut done = 0u32;
                loop {
                    if game_read::<u32>(SPLASHED_RVA + k) >= LAST_SPLASHED {
                        break;
                    }
                    splash_mud()?;
                    done = done.wrapping_add(1);
                    if done == game_read::<u32>(SPLASHES_RVA) {
                        break;
                    }
                }
            }
        }
        crate::d3d::unlock_texture::unlock_texture(game_read(LOCKED_RVA))?;
        Ok(())
    }
}
