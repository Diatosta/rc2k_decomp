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
//! See `race/mud.rs`. One caller, `race::mud_texture_turn`. Refuses (before anything) a texture
//! past 3 and no mud, and its callees' refusals (what the counts, the throws and the random
//! state were put back).

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::physics::flow::c0;
use crate::race::muddy_texel::texture_and_mud;
use crate::race::numbers_store::NUMBERS;
use crate::race::splash_mud::{LAST_SPLASHED, SPLASHED_RVA, splash_mud};
use crate::race::throw_mud_texel::{LAST_THROWN, THROWN_RVA, throw_mud_texel};
use crate::race::vars_store::RACE_VARS;
use crate::scene::x87_mem::{fild, fistp, fld, fstp};

const FIRST_SLOT_RVA: usize = rva(0x5D1EE8); // g_kindFirstSlots[0x100]
const FIRST_TEXTURE_RVA: usize = rva(0x5D0EE8); // g_kindFirstTextures[0x100]
const CHANGES_RVA: usize = rva(0x6D1508); // dword_6D1508
const THROWS_IN_RVA: usize = rva(0x623D60); // dword_623D60: each texture's thrown mud (float)
const SPLASHES_IN_RVA: usize = rva(0x623D70); // dword_623D70: each texture's splashes (float)
const THROWS_RVA: usize = rva(0x623920); // dword_623920
const SPLASHES_RVA: usize = rva(0x623924); // dword_623924

/// `fld dword [f]; fistp dword [t]`.
fn round_units(f: usize, t: usize) {
    unsafe { fistp(t as *mut u8, 0, fld(f as *const u8, 0)) }
}

/// `fld dword [a]; fisub dword [u]; fstp dword [a]`.
fn subtract_units(a: usize, u: usize) {
    unsafe {
        fstp(
            a as *mut u8,
            0,
            fld(a as *const u8, 0) - fild(u as *const u8, 0),
        )
    }
}

/// `fld [from]; fistp [to]`, then `fld zero; fld [to]; fcompp` and, with C0, `fld zero;
/// fstp [to]`: the units of a float count, negatives 0 (as the original, by bits).
fn whole_units(from: usize, to: usize) -> i32 {
    round_units(game_addr(from), game_addr(to));
    let zero: u32 = NUMBERS.zero.bits();
    if c0(unsafe { game_read(to) }, zero) {
        unsafe { game_write(to, zero) };
    }
    unsafe { game_read(to) }
}

/// `fld [at]; fisub [units]; fstp [at]`.
fn take_units(at: usize, units: usize) {
    subtract_units(game_addr(at), game_addr(units));
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
        RACE_VARS.mud_locked.set_bits(texture);
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
                RACE_VARS
                    .mud_throws
                    .set_bits((throws as u32).wrapping_sub(over));
            }
            take_units(THROWS_IN_RVA + k, THROWS_RVA);
            let throws = RACE_VARS.mud_throws.bits() as i32;
            if throws > 0 {
                let mut done = 0u32;
                loop {
                    throw_mud_texel()?;
                    done = done.wrapping_add(1);
                    if done == RACE_VARS.mud_throws.bits() {
                        break;
                    }
                }
            }
        }
        // The splashes.
        whole_units(SPLASHES_IN_RVA + k, SPLASHES_RVA);
        if game_read::<u32>(SPLASHED_RVA + k) < LAST_SPLASHED {
            take_units(SPLASHES_IN_RVA + k, SPLASHES_RVA);
            let splashes = RACE_VARS.mud_splashes.bits() as i32;
            if splashes > 0 {
                let mut done = 0u32;
                loop {
                    if game_read::<u32>(SPLASHED_RVA + k) >= LAST_SPLASHED {
                        break;
                    }
                    splash_mud()?;
                    done = done.wrapping_add(1);
                    if done == RACE_VARS.mud_splashes.bits() {
                        break;
                    }
                }
            }
        }
        crate::d3d::unlock_texture::unlock_texture(RACE_VARS.mud_locked.bits())?;
        Ok(())
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions.
    reference!(ref_round(f, t) "fld dword ptr [{f}]", "fistp dword ptr [{t}]");
    reference!(ref_subtract(a, u)
        "fld dword ptr [{a}]",
        "fisub dword ptr [{u}]",
        "fstp dword ptr [{a}]");

    #[test]
    fn mud_texture_matches_the_assembly() {
        let blocks = vec![
            Block {
                name: "whole_units",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_round(a[0], a[1]) },
                plain: |a| round_units(a[0], a[1]),
            },
            Block {
                name: "take_units",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[(0, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_subtract(a[0], a[1]) },
                plain: |a| subtract_units(a[0], a[1]),
            },
        ];
        sparse(&blocks, 5000, 207);
    }
}
