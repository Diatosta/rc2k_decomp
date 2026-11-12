//! `race::update_mud_texture` (0x44A12E): the mud thrown and splashed since the last turn
//! put onto the texture worked on (`mud_texture`, n). The texture's slot (kind 0x100's
//! first slot + n) forgotten (`scene::forget_slot_texture`), `texture_changes` + 0x10000; the
//! texture (kind 0x100's first texture + n, into `mud_locked`; none: nothing more)
//! locked (`d3d::lock_texture`, the platform's; carry: nothing more). Then:
//! - the throws: `mud_thrown_in`[n]` (the wheels' thrown mud, a float) rounded (`fistp`)
//!   into `mud_throws`, which read back as a float (sic) and below 0.0 becomes 0.0's
//!   bits: negative counts are 0. Unless the texture's count `mud_thrown_counts`[n]` is past
//!   0x60000: held so that its low word plus the throws stays below 0x1000000, taken off
//!   `mud_thrown_in`[n]` (`fisub`), and above 0 that many `race::throw_mud_texel`;
//! - the splashes: the same from `mud_splashes_in`[n]` into `mud_splashes`, unless the count
//!   `mud_splash_counts`[n]` has reached 0x1000 (also checked before each): `race::splash_mud`;
//!
//! Then the texture is unlocked (`d3d::unlock_texture`).
//!
//! See `race/mud.rs`. One caller, `race::mud_texture_turn`. Refuses (before anything) a texture
//! past 3 and no mud, and its callees' refusals (what the counts, the throws and the random
//! state were put back).

use crate::effects::cell_x87::CellX87;
use crate::errors::Refused;
use crate::math::x87_float::{int, to_int};
use crate::physics::flow::c0;
use crate::race::muddy_texel::texture_and_mud;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::splash_mud::{LAST_SPLASHED, splash_mud};
use crate::race::throw_mud_texel::{LAST_THROWN, throw_mud_texel};
use crate::session::{Cell32, Dword, Float};

/// `fld dword [f]; fistp dword [t]`.
fn round_units(f: &impl Cell32, t: &impl Cell32) {
    t.set_bits(to_int(f.ld()) as u32);
}

/// `fld dword [a]; fisub dword [u]; fstp dword [a]`.
fn subtract_units(a: &impl Cell32, u: &impl Cell32) {
    a.st(a.ld() - int(u.bits() as i32));
}

/// `fld [from]; fistp [to]`, then `fld zero; fld [to]; fcompp` and, with C0, `fld zero;
/// fstp [to]`: the units of a float count, negatives 0 (as the original, by bits).
fn whole_units(course: &Course, from: &Float, to: &Dword) -> i32 {
    round_units(from, to);
    let zero: u32 = course.numbers.zero.bits();
    if c0(to.get(), zero) {
        to.set(zero);
    }
    to.get() as i32
}

/// `fld [at]; fisub [units]; fstp [at]`.
fn take_units(at: &Float, units: &Dword) {
    subtract_units(at, units);
}

pub fn update_mud_texture(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let textures = &scene.textures;
    let frame = &race.frame;
    let n = texture_and_mud(&race.shared)?;
    let k = n as usize;
    crate::scene::forget_slot_texture::forget_slot_texture(
        textures.kinds.kind_first_slots[256].get().wrapping_add(n),
        scene,
    )?;
    frame.panel_and_overlay.panel.texture_changes.set(
        frame
            .panel_and_overlay
            .panel
            .texture_changes
            .get()
            .wrapping_add(0x1_0000),
    );
    let texture = textures.kinds.kind_first_textures[256]
        .get()
        .wrapping_add(n);
    race.shared.vars.mud_locked.set(texture);
    if texture == 0 {
        return Ok(());
    }
    if crate::d3d::lock_texture::lock_texture(texture)?.carry {
        return Ok(());
    }
    // The throws.
    let throws = whole_units(
        &race.course,
        &race.shared.vars.mud_thrown_in[k],
        &race.shared.vars.mud_throws,
    );
    let count: u32 = race.shared.vars.mud_thrown_counts[k].get();
    if count <= LAST_THROWN {
        let over = (count & 0xFFFF).wrapping_add(throws as u32);
        if (over.wrapping_sub(0x100_0000) as i32) >= 0 {
            let over = over.wrapping_sub(0x100_0000);
            race.shared
                .vars
                .mud_throws
                .set((throws as u32).wrapping_sub(over));
        }
        take_units(
            &race.shared.vars.mud_thrown_in[k],
            &race.shared.vars.mud_throws,
        );
        let throws = race.shared.vars.mud_throws.get() as i32;
        if throws > 0 {
            let mut done = 0u32;
            loop {
                throw_mud_texel(race, scene, gfx)?;
                done = done.wrapping_add(1);
                if done == race.shared.vars.mud_throws.get() {
                    break;
                }
            }
        }
    }
    // The splashes.
    whole_units(
        &race.course,
        &race.shared.vars.mud_splashes_in[k],
        &race.shared.vars.mud_splashes,
    );
    if race.shared.vars.mud_splash_counts[k].get() < LAST_SPLASHED {
        take_units(
            &race.shared.vars.mud_splashes_in[k],
            &race.shared.vars.mud_splashes,
        );
        let splashes = race.shared.vars.mud_splashes.get() as i32;
        if splashes > 0 {
            let mut done = 0u32;
            loop {
                if race.shared.vars.mud_splash_counts[k].get() >= LAST_SPLASHED {
                    break;
                }
                splash_mud(&race.shared, scene, gfx)?;
                done = done.wrapping_add(1);
                if done == race.shared.vars.mud_splashes.get() {
                    break;
                }
            }
        }
    }
    crate::d3d::unlock_texture::unlock_texture(race.shared.vars.mud_locked.get())?;
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
