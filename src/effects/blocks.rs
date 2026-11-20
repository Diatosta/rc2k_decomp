//! The steps of `effects::init_marks` that take its three blocks (counted on their lists,
//! not kept), scale the corner table and reset the eight settings blocks.

use crate::effects::blocks_store::Particle;
use crate::effects::particles_store::Particles;
use crate::effects::ring_store::Ring;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::memory::session::PlainFloat;
use crate::memory::session::write_bits;

const PLACES_LEN: u32 = 0x7024;
const RING_LEN: u32 = 0x4C000;
const LINKS_LEN: u32 = 0x71020;
pub const CORNERS: u32 = 0x100;

const SETTINGS_BLOCKS: u32 = 8;

/// `effects::allocate_places`, step 6: 0x7024 bytes on `PlacesBlock.list[0]` into
/// the places' block, its first three floats `PlacesBlock.start[0]` ... Nothing reads the
/// block, so it is only counted on its list (the memory figures) and not kept.
pub(super) fn allocate_places(effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    crate::memory::owned::count_on(&effects.blocks.places.list, PLACES_LEN)
}

/// Particle `p`, a particle of `Particles::ring` (taken by `allocate_ring`, given back by
/// `effects::tidy_marks`) named by its number + 1 (0 none), as in the links and in
/// `RingPool.ring` / `RingPool.free`, as it is now; refused for none or a number past the ring.
pub(super) fn read_particle(particles: &Particles, p: u32) -> Result<Particle, BadInput> {
    particles
        .ring
        .with(|r| r.get((p as usize).checked_sub(1)?).copied())?
        .ok_or(BadInput("a particle past the ring"))
}

/// Particle `p` changed by `change`; refused as `read_particle`, and then nothing is written.
pub(super) fn update_particle(
    particles: &Particles,
    p: u32,
    change: impl FnOnce(&mut Particle),
) -> Result<(), BadInput> {
    particles
        .ring
        .with(|r| {
            change(r.get_mut((p as usize).checked_sub(1)?)?);
            Some(())
        })?
        .ok_or(BadInput("a particle past the ring"))
}

/// `effects::allocate_ring`, step 13: 0x4C000 bytes on `RingPool.list[0]` for the ring and its
/// free-entry pointer, the used count 0, and the 0x1000 entries of 0x4C made a ring (the first two
/// dwords of each the particle before and the one after). The particles are `Particles::ring`,
/// counted on the list.
pub fn allocate_ring(particles: &Particles, ring: &Ring) -> Result<(), Refused> {
    // The logic is in `rc2k_core::effects::rings::neighbours`.
    let links: Vec<Particle> = (0..rc2k_core::effects::rings::ENTRIES)
        .map(|n| {
            let [before, after] = rc2k_core::effects::rings::neighbours(n);
            Particle {
                before: before + 1,
                after: after + 1,
                ..Particle::default()
            }
        })
        .collect();
    particles.ring.take(&ring.pool.list, RING_LEN, links)?;
    ring.pool.ring.set(1);
    ring.pool.free.set(1);
    ring.pool.used.set(0u32);
    Ok(())
}

/// `effects::allocate_links`, in step 3: 0x71020 bytes on `MarkLinks.list[0]`, its
/// last dword 0, and the 0x1000 entries' before and after links made a ring, then
/// `effects::scale_corners`. Nothing reads the block, so it is only counted on its list
/// (the memory figures) and not kept.
pub fn allocate_links(
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    crate::memory::owned::count_on(&effects.blocks.links.list, LINKS_LEN)?;
    // Set up on the game's thread, into its buffer.
    scale_corners(&scene.scratch.game_scratch().vertices, effects);
    Ok(())
}

/// The word pairs of a corner record, in the order they are made: the byte offsets of the pairs
/// in the record (the fourth pair before the third).
const CORNER_PAIRS: [u32; 4] = [0, 4, 0xC, 8];

/// `effects::scale_corners`: for the 0x100 corner records (0xFF first), the four word pairs
/// (in the order of `CORNER_PAIRS`) each times `DrawWork.uv_k` into the work and into the eight
/// `values` of the record in the corners' slopes; its `slope` there: |v of the third pair - v of
/// the first| over |u of the second - u of the first|
/// (sixteen-bit differences; `DrawWork.half[0]`, `DrawWork.half[1]`); 0 for a record with
/// both 0.
pub fn scale_corners(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    effects: &crate::effects::owner::Effects,
) {
    {
        let scale = effects.frame.draw_work.uv_k.get();
        for n in (0..CORNERS).rev() {
            let record = &effects.blocks.corners.corners[n as usize];
            let word = |o: u32| record.words[o as usize / 2].get();
            let mut values = [0u32; 8];
            for (k, &o) in CORNER_PAIRS.iter().enumerate() {
                for h in 0..2 {
                    let v = x87::int_times(word(o + 2 * h as u32) as i32, scale);
                    values[2 * k + h] = v.to_bits();
                    let vertex = &buffer.vertices[k];
                    if h == 0 {
                        vertex.u.set(v);
                    } else {
                        vertex.v.set(v);
                    }
                }
            }
            let side = |a: u32, b: u32| {
                let d = word(a).wrapping_sub(word(b));
                (if (d as i16) < 0 { d.wrapping_neg() } else { d }) as u32
            };
            let (across, along) = (side(4, 0), side(0xA, 2));
            write_bits(&effects.frame.draw_work.half, [across, along]);
            let slope = &effects.blocks.corner_slopes.slopes[n as usize];
            // A record of no size (both sides 0: the stage's unused corners) has no slope:
            // it is 0 (dividing 0 by 0 would give a NaN).
            let ratio = if along == 0 && across == 0 {
                0.0
            } else {
                x87::int_ratio(along as i32, across as i32)
            };
            slope.slope.set(ratio);
            write_bits(&slope.values, values);
        }
    }
}

/// `effects::reset_settings_blocks`, in step 3: the eight blocks of 0x4C8 in the marks'
/// settings (the last first): the `motion` and the first `steps` row the six floats from
/// `EmitterVars.settings_start[0]`, the last three again in each other `steps` row, and the
/// counter that holds the emitting back set to 8.
pub(super) fn reset_settings_blocks(
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let marks = effects.settings()?;
        for n in (0..SETTINGS_BLOCKS).rev() {
            let car = &marks.cars[n as usize];
            for (cell, v) in car
                .motion
                .iter()
                .chain(&car.steps[0])
                .zip(&effects.kinds.emitter.settings_start)
            {
                cell.set(x87::loaded(v));
            }
            let last = car.steps[0].each_ref().map(PlainFloat::get);
            for steps in &car.steps[1..] {
                for (cell, v) in steps.iter().zip(last) {
                    cell.set(v);
                }
            }
            car.reset_wait.set(8);
        }
    }
    Ok(())
}
