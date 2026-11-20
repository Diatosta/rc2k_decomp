//! The steps of `effects::init_marks` that take its three blocks (counted on their lists,
//! not kept), scale the corner table and reset the eight settings blocks.

use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::memory::session::write_bits;
use crate::memory::session::{PlainDword, PlainFloat, PlainWord};

const PLACES_LEN: u32 = 0x7024;
const RING_LEN: u32 = 0x4C000;
pub const RING_ENTRY: u32 = rc2k_core::effects::rings::RING_ENTRY;
const LINKS_LEN: u32 = 0x71020;
pub const CORNERS: u32 = 0x100;

const SETTINGS_BLOCKS: u32 = 8;

/// `effects::allocate_places`, step 6: 0x7024 bytes on `PlacesBlock.list[0]` into
/// the places' block, its first three floats `PlacesBlock.start[0]` ... Nothing reads the
/// block, so it is only counted on its list (the memory figures) and not kept.
pub(super) fn allocate_places(effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    crate::memory::owned::count_on(&effects.blocks.places.list, PLACES_LEN)
}

/// A particle of the ring (0x4C-byte entries), its dwords in order: the particle before and the
/// one after (each a particle's number + 1), the kind, the place (three), the last place
/// (three), the ground, the motion (four dwords, the third the lift), the size, the colour,
/// the colour's step, the life, the colours' place.
pub(super) type Particle = [u32; (RING_ENTRY / 4) as usize];

/// The dword at `offset` of particle `p`, a particle of `Particles::ring` (taken by
/// `allocate_ring`, given back by `effects::tidy_marks`) named by its number + 1 (0 none), as
/// in the links and in `RingPool.ring` / `RingPool.free`; refused for none, a number past the ring
/// or an offset past the entry.
pub(super) fn particle(particles: &Particles, p: u32, offset: u32) -> Result<u32, BadInput> {
    particles
        .ring
        .with(|r| {
            let e = r.get((p as usize).checked_sub(1)?)?;
            offset
                .is_multiple_of(4)
                .then(|| e.get(offset as usize / 4).copied())?
        })?
        .ok_or(BadInput("a particle past the ring"))
}

/// The dword at `offset` of particle `p` set to `v`; refused as `particle`.
pub(super) fn set_particle(
    particles: &Particles,
    p: u32,
    offset: u32,
    v: u32,
) -> Result<(), BadInput> {
    particles
        .ring
        .with(|r| {
            let e = r.get_mut((p as usize).checked_sub(1)?)?;
            let d = offset
                .is_multiple_of(4)
                .then(|| e.get_mut(offset as usize / 4))??;
            *d = v;
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
            let mut p = [0; (RING_ENTRY / 4) as usize];
            let [before, after] = rc2k_core::effects::rings::neighbours(n);
            p[0] = before + 1;
            p[1] = after + 1;
            p
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

/// The places' list, block and first floats.
#[derive(rc2k_derive::Cells)]
pub struct PlacesBlock {
    /// The list the places' block is taken on (16 bytes).
    pub list: crate::memory::ListCells,
    /// The places' block (0x7024 bytes; 0: only counted, `allocate_places`).
    pub places: PlainDword,
    /// The places' block's first three floats (`effects::allocate_places`).
    pub start: [PlainFloat; 3],
}

impl PlacesBlock {
    pub const fn new() -> Self {
        Self {
            list: crate::memory::ListCells::empty(),
            places: PlainDword::new(0x0),
            start: [
                PlainFloat::new(0.0),
                PlainFloat::new(-262144.0),
                PlainFloat::new(1279952.0),
            ],
        }
    }
}

/// The links' list and block.
#[derive(rc2k_derive::Cells)]
pub struct MarkLinks {
    /// The list the links' block is taken on (16 bytes).
    pub list: crate::memory::ListCells,
    /// The links' block (0x71020 bytes; 0: only counted, `allocate_links`).
    pub links: PlainDword,
}

impl MarkLinks {
    pub const fn new() -> Self {
        Self {
            list: crate::memory::ListCells::empty(),
            links: PlainDword::new(0x0),
        }
    }
}

/// A corner's slope record (0x24 bytes): the slope of its sides and its eight scaled values
/// (`effects::scale_corners`; `effects::draw_cell_sprite` scales a sprite by the slope, and
/// `effects::set_quad_uvs` reads record 0x58's eight values as four texture places).
#[derive(rc2k_derive::Cells)]
pub struct Slope {
    /// |v of the third pair - v of the first| over |u of the second - u of the first| of the
    /// corner record, as a float.
    pub slope: PlainFloat,
    /// The corner's four word pairs times `DrawWork::uv_k`.
    pub values: [PlainFloat; 8],
}

impl Slope {
    pub const fn new() -> Self {
        Slope {
            slope: PlainFloat::new(0.0),
            values: [const { PlainFloat::new(0.0) }; 8],
        }
    }
}

/// The corners' slopes (0x100 records of 0x24 bytes).
#[derive(rc2k_derive::Cells)]
pub struct CornerSlopes {
    /// One record for each corner record.
    pub slopes: [Slope; 0x100],
}

impl CornerSlopes {
    pub const fn new() -> Self {
        Self {
            slopes: [const { Slope::new() }; 0x100],
        }
    }
}

/// A corner record (0x80 bytes of 16-bit words: four (u, v) pairs of a cell's texture
/// places, and what `effects::scale_corners` scales).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct Corner {
    pub words: [PlainWord; 0x40],
}

impl Corner {
    pub const fn new() -> Self {
        Corner {
            words: [const { PlainWord::new(0) }; 0x40],
        }
    }
}

/// The marks' corner table (0x100 records of 0x80 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct MarkCorners {
    /// The corner records (the course's 0x8000 bytes of `markinfo` are copied here).
    pub corners: [Corner; 0x100],
}

impl MarkCorners {
    pub const fn new() -> Self {
        Self {
            corners: [const { Corner::new() }; 0x100],
        }
    }
}

/// The particle ring's pool and the burst code's scratch.
#[derive(rc2k_derive::Cells)]
pub struct RingPool {
    /// A `memory::List` (head, total, cursor, remaining).
    pub list: crate::memory::ListCells,
    /// The first entry of the ring in use.
    pub free: PlainDword,
    /// The block taken for the ring.
    pub ring: PlainDword,
    /// A developer's toggle (the debug page's "fworks run"). Nothing reads or writes it.
    pub fireworks_run: PlainDword,
    /// A burst particle's kind plus 8 bits.
    pub nine: PlainDword,
    /// A burst particle's kind times 0x400.
    pub kind: PlainDword,
    /// The burst's radius.
    pub burst_radius: PlainFloat,
    /// The colour index of the particle being drawn.
    pub index: PlainDword,
    /// The entries taken.
    pub used: PlainDword,
}

/// RingPool's first values.
impl RingPool {
    pub const fn new() -> Self {
        RingPool {
            list: crate::memory::ListCells::empty(),
            free: PlainDword::new(0x0),
            ring: PlainDword::new(0x0),
            fireworks_run: PlainDword::new(0x0),
            nine: PlainDword::new(0x0),
            kind: PlainDword::new(0x0),
            burst_radius: PlainFloat::new(0.0),
            index: PlainDword::new(0x0),
            used: PlainDword::new(0x0),
        }
    }
}
