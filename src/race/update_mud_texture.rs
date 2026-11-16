//! `race::update_mud_texture`: puts the mud thrown and splashed since the last turn onto the
//! texture worked on (`mud_texture`, number `n`). The texture's slot (kind 0x100's first slot +
//! `n`) is forgotten (`scene::forget_slot_texture`), `texture_changes` goes up by 0x10000, and
//! the texture (kind 0x100's first texture + `n`, kept in `mud_locked`; none: nothing more) is
//! locked (`d3d::lock_texture`; a failed lock: nothing more). Then:
//! - the throws: `mud_thrown_in[n]` (the wheels' thrown mud, a float) is rounded into
//!   `mud_throws`, which is then read back as a float and, when below 0.0, becomes 0.0's bits
//!   (negative counts are 0). Unless the texture's count `mud_thrown_counts[n]` is past 0x60000,
//!   the throws are held so that its low word plus the throws stays below 0x1000000, taken off
//!   `mud_thrown_in[n]`, and when above 0 that many `race::throw_mud_texel` run;
//! - the splashes: the same from `mud_splashes_in[n]` into `mud_splashes`, unless the count
//!   `mud_splash_counts[n]` has reached 0x1000 (also checked before each): `race::splash_mud`.
//!
//! Then the texture is unlocked (`d3d::unlock_texture`).
//!
//! See `race/mud.rs`. Called by `race::mud_texture_turn`. Refuses (before anything) a texture
//! past 3 and no mud, and its callees' refusals (the counts, the throws and the random state
//! are put back).

use crate::effects::cell_x87::CellX87;
use crate::errors::Refused;
use crate::math::x87_float::{int, to_int};
use crate::memory::session::{PlainDword, PlainFloat};
use crate::physics::flow::c0;
use crate::race::muddy_texel::texture_and_mud;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::splash_mud::{LAST_SPLASHED, splash_mud};
use crate::race::throw_mud_texel::{LAST_THROWN, throw_mud_texel};

/// `f` rounded to a whole number into `t`.
fn round_units(f: &PlainFloat, t: &PlainDword) {
    t.set(to_int(f.ld()) as u32);
}

/// `u` whole units taken off the float `a`.
fn subtract_units(a: &PlainFloat, u: &PlainDword) {
    a.st(a.ld() - int(u.get() as i32));
}

/// The units of a float count: `from` rounded into `to`, which is then read back as a float
/// and set to 0 when negative. Returns it.
fn whole_units(course: &Course, from: &PlainFloat, to: &PlainDword) -> i32 {
    round_units(from, to);
    let zero = course.numbers.get().zero.get();
    if c0(to.get(), zero) {
        to.set(zero.to_bits());
    }
    to.get() as i32
}

/// `units` taken off the float `at`.
fn take_units(at: &PlainFloat, units: &PlainDword) {
    subtract_units(at, units);
}

pub fn update_mud_texture(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let textures = &scene.textures;
    let frame = &race.frame;
    let n = texture_and_mud(&race.shared, &race.course.stage.mud)?;
    let k = n as usize;
    crate::scene::forget_slot_texture::forget_slot_texture(
        textures.kinds.get().kind_first_slots[256]
            .get()
            .wrapping_add(n),
        scene,
    )?;
    frame.panel_and_overlay.get().panel.texture_changes.set(
        frame
            .panel_and_overlay
            .get()
            .panel
            .texture_changes
            .get()
            .wrapping_add(0x1_0000),
    );
    let texture = textures.kinds.get().kind_first_textures[256]
        .get()
        .wrapping_add(n);
    race.shared.vars.get().mud_locked.set(texture);
    if texture == 0 {
        return Ok(());
    }
    if crate::d3d::lock_texture::lock_texture_in(gfx, texture)?.carry {
        return Ok(());
    }
    // The throws.
    let throws = whole_units(
        &race.course,
        &race.shared.vars.get().mud_thrown_in[k],
        &race.shared.vars.get().mud_throws,
    );
    let count: u32 = race.shared.vars.get().mud_thrown_counts[k].get();
    if count <= LAST_THROWN {
        let over = (count & 0xFFFF).wrapping_add(throws as u32);
        if (over.wrapping_sub(0x100_0000) as i32) >= 0 {
            let over = over.wrapping_sub(0x100_0000);
            race.shared
                .vars
                .get()
                .mud_throws
                .set((throws as u32).wrapping_sub(over));
        }
        take_units(
            &race.shared.vars.get().mud_thrown_in[k],
            &race.shared.vars.get().mud_throws,
        );
        let throws = race.shared.vars.get().mud_throws.get() as i32;
        if throws > 0 {
            let mut done = 0u32;
            loop {
                throw_mud_texel(race, scene, gfx, system)?;
                done = done.wrapping_add(1);
                if done == race.shared.vars.get().mud_throws.get() {
                    break;
                }
            }
        }
    }
    // The splashes.
    whole_units(
        &race.course,
        &race.shared.vars.get().mud_splashes_in[k],
        &race.shared.vars.get().mud_splashes,
    );
    if race.shared.vars.get().mud_splash_counts[k].get() < LAST_SPLASHED {
        take_units(
            &race.shared.vars.get().mud_splashes_in[k],
            &race.shared.vars.get().mud_splashes,
        );
        let splashes = race.shared.vars.get().mud_splashes.get() as i32;
        if splashes > 0 {
            let mut done = 0u32;
            loop {
                if race.shared.vars.get().mud_splash_counts[k].get() >= LAST_SPLASHED {
                    break;
                }
                splash_mud(&race.shared, &race.course.stage.mud, scene, gfx, system)?;
                done = done.wrapping_add(1);
                if done == race.shared.vars.get().mud_splashes.get() {
                    break;
                }
            }
        }
    }
    crate::d3d::unlock_texture::unlock_texture(gfx, race.shared.vars.get().mud_locked.get())?;
    Ok(())
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

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
                plain: |a| {
                    round_units(
                        crate::scene::x87_mem::view::<PlainFloat>(a[0], 0),
                        crate::scene::x87_mem::view::<PlainDword>(a[1], 0),
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
                plain: |a| {
                    subtract_units(
                        crate::scene::x87_mem::view::<PlainFloat>(a[0], 0),
                        crate::scene::x87_mem::view::<PlainDword>(a[1], 0),
                    );
                },
            },
        ];
        sparse("race.update_mud_texture", &blocks, 5000, 207);
    }
}
