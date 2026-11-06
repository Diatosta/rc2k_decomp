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
use crate::math::x87_float::{int, ld, st, to_int};
use crate::physics::flow::c0;
use crate::race::muddy_texel::texture_and_mud;
use crate::race::owner::race;
use crate::race::splash_mud::{LAST_SPLASHED, splash_mud};
use crate::race::throw_mud_texel::{LAST_THROWN, throw_mud_texel};
use crate::race::vars_store::RACE_VARS;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::session::{Cell32, Dword, Float};

/// `fld dword [f]; fistp dword [t]`.
fn round_units(f: &impl Cell32, t: &impl Cell32) {
    t.set_bits(to_int(ld(f.bits())) as u32);
}

/// `fld dword [a]; fisub dword [u]; fstp dword [a]`.
fn subtract_units(a: &impl Cell32, u: &impl Cell32) {
    a.set_bits(st(ld(a.bits()) - int(u.bits() as i32)));
}

/// `fld [from]; fistp [to]`, then `fld zero; fld [to]; fcompp` and, with C0, `fld zero;
/// fstp [to]`: the units of a float count, negatives 0 (as the original, by bits).
fn whole_units(from: &Float, to: &Dword) -> i32 {
    round_units(from, to);
    let zero: u32 = race().course.numbers.zero.bits();
    if c0(to.bits(), zero) {
        to.set_bits(zero);
    }
    to.bits() as i32
}

/// `fld [at]; fisub [units]; fstp [at]`.
fn take_units(at: &Float, units: &Dword) {
    subtract_units(at, units);
}

/// The Rust version (see the header).
pub fn update_mud_texture() -> Result<(), Refused> {
    let frame = &race().frame;
    let n = texture_and_mud()?;
    let k = n as usize;
    crate::scene::forget_slot_texture::forget_slot_texture(
        KIND_TEXTURES.kind_first_slots[256].bits().wrapping_add(n),
    )?;
    frame.panel_and_overlay.panel.texture_changes.set(
        frame
            .panel_and_overlay
            .panel
            .texture_changes
            .get()
            .wrapping_add(0x1_0000),
    );
    let texture = KIND_TEXTURES.kind_first_textures[256]
        .bits()
        .wrapping_add(n);
    RACE_VARS.mud_locked.set_bits(texture);
    if texture == 0 {
        return Ok(());
    }
    if crate::d3d::lock_texture::lock_texture(texture)?.carry {
        return Ok(());
    }
    // The throws.
    let throws = whole_units(&RACE_VARS.mud_thrown_in[k], &RACE_VARS.mud_throws);
    let count: u32 = RACE_VARS.mud_thrown_counts[k].bits();
    if count <= LAST_THROWN {
        let over = (count & 0xFFFF).wrapping_add(throws as u32);
        if (over.wrapping_sub(0x100_0000) as i32) >= 0 {
            let over = over.wrapping_sub(0x100_0000);
            RACE_VARS
                .mud_throws
                .set_bits((throws as u32).wrapping_sub(over));
        }
        take_units(&RACE_VARS.mud_thrown_in[k], &RACE_VARS.mud_throws);
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
    whole_units(&RACE_VARS.mud_splashes_in[k], &RACE_VARS.mud_splashes);
    if RACE_VARS.mud_splash_counts[k].bits() < LAST_SPLASHED {
        take_units(&RACE_VARS.mud_splashes_in[k], &RACE_VARS.mud_splashes);
        let splashes = RACE_VARS.mud_splashes.bits() as i32;
        if splashes > 0 {
            let mut done = 0u32;
            loop {
                if RACE_VARS.mud_splash_counts[k].bits() >= LAST_SPLASHED {
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

#[cfg(all(test, target_arch = "x86"))]
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
                plain: |a| {
                    round_units(
                        &crate::scene::x87_mem::HostCell(a[0]),
                        &crate::scene::x87_mem::HostCell(a[1]),
                    );
                },
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
                plain: |a| {
                    subtract_units(
                        &crate::scene::x87_mem::HostCell(a[0]),
                        &crate::scene::x87_mem::HostCell(a[1]),
                    );
                },
            },
        ];
        sparse(&blocks, 5000, 207);
    }
}
