//! More of `effects::init_marks`' steps (IDA): the three blocks it takes
//! (Rust's since phase 12, counted on their lists), the corner table
//! scaled, and the eight settings blocks reset.

use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::session::write_bits;
use crate::session::{PlainDword, PlainFloat, PlainWord};

pub const PLACES_LEN: u32 = 0x7024;
pub const RING_LEN: u32 = 0x4C000;
pub const RING_ENTRY: u32 = rc2k_core::effects::rings::RING_ENTRY;
pub const LINKS_LEN: u32 = 0x71020;
pub const CORNERS: u32 = 0x100;

pub const SETTINGS_BLOCKS: u32 = 8;

/// `effects::allocate_places` (0x4C38B1), step 6: 0x7024 bytes on `PlacesBlock.list[0]` into
/// `g_marksPlaces`, its first three floats `PlacesBlock.start[0]` ... Only code no reference
/// reaches reads the block (0x4C3908, 0x4C396A, 0x4C39D5: `notes/reference.md`), so
/// since phase 12 it is only counted on its list (the memory figures) and not kept.
pub fn allocate_places(effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    crate::memory::owned::count_on(&effects.blocks.places.list, PLACES_LEN)
}

/// A particle of the ring (`g_marksRing`'s 0x4C-byte entries), its dwords by their
/// offsets: +0 the one before, +4 the one after (each a particle's number + 1, since phase
/// 12; the original's addresses), +8 the kind, +0xC .. +0x14 the place, +0x18 .. +0x20 the
/// last place, +0x24 the ground, +0x28 .. +0x34 the motion (+0x30 the lift), +0x38 the
/// size, +0x3C the colour, +0x40 the colour's step, +0x44 the life, +0x48 the colours'
/// place.
pub type Particle = [u32; (RING_ENTRY / 4) as usize];

/// The particles (`Particles::ring`; `g_marksRing`, the game's 0x4C000-byte block on `RingPool.list[0]`), Rust's
/// since phase 12: taken by `allocate_ring`, given back by `effects::tidy_marks`. A
/// particle is named by its number + 1 (0 none), in the links and in `RING_POOL.ring` /
/// `free`.
/// The dword at `offset` of particle `p` (its number + 1); refused for none, a number past
/// the ring or an offset past the entry.
pub fn particle(particles: &Particles, p: u32, offset: u32) -> Result<u32, BadInput> {
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
pub fn set_particle(particles: &Particles, p: u32, offset: u32, v: u32) -> Result<(), BadInput> {
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

/// `effects::allocate_ring` (0x4CFAC5), step 13: 0x4C000 bytes on `RingPool.list[0]` into
/// `g_marksRing` and `g_marksRingFree`, `g_marksRingUsed` 0, and the 0x1000 entries of
/// 0x4C made a ring (+0 the one before, +4 the one after). The particles are Rust's
/// (`Particles::ring`), counted on the list as the game's block was.
pub fn allocate_ring(particles: &Particles, ring: &Ring) -> Result<(), Refused> {
    // Converted: `rc2k_core::effects::rings::neighbours`.
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

/// `effects::allocate_links` (0x4C5398), in step 3: 0x71020 bytes on `MarkLinks.list[0]` into
/// `g_marksLinks`, its +0x7100C 0, at +8 and +0x4008 the 0x1000 entries' before and after
/// (round), then `effects::scale_corners`. Only code no reference reaches reads the block
/// (0x4C5446 .. 0x4C5DEA, 0x4C6242: `notes/reference.md`), so since phase 12 it is
/// only counted on its list (the memory figures) and not kept.
pub fn allocate_links(effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    crate::memory::owned::count_on(&effects.blocks.links.list, LINKS_LEN)?;
    // Set up on the game's thread, into its buffer.
    scale_corners(&crate::workers::game_scratch().vertices, effects);
    Ok(())
}

/// The word pairs of a corner record, in the order they are made: +0, +4, +0xC, +8.
const CORNER_PAIRS: [u32; 4] = [0, 4, 0xC, 8];

/// `effects::scale_corners` (0x4C6420): for the 0x100 corner records (0xFF first), the four
/// word pairs (+0, +4, +0xC, +8) each times `DrawWork.uv_k` into the work and into +4 .. +0x20
/// of the record's 0x24 at `g_marksCornerSlopes`; +0 there the slope: |+0xA - +2| over
/// |+4 - +0| (sixteen-bit differences; `DrawWork.half[0]`, `DrawWork.half[1]`); 0 for a record
/// with both 0 (the original's NaN, section 16).
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
            // A record of no size (both sides 0: the stage's unused corners) has no slope.
            // The original divided 0 by 0 all the same and kept a NaN (a 1999 slip; in play
            // only `draw_cell_sprite`'s misplaced read could reach it, and none of the runs
            // did). Here it is 0.
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

/// `effects::reset_settings_blocks` (0x4C9A6D), in step 3: the eight blocks of 0x4C8 at
/// `g_marksSettings` + 0x60 (the last first): +0x4C .. +0x60 the six floats from
/// `EmitterVars.settings_start[0]`, the last three again at +0x64, +0x70, +0x7C, +0x30 8.
pub fn reset_settings_blocks(effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
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

/// The places' list, block and first floats (IDA 0x8DB570 .. 0x8DB5C0).
#[derive(rc2k_derive::Cells)]
pub struct PlacesBlock {
    /// 0x8DB570: `stru_8DB570` (`g_marksPlacesList`): the list the places' block is taken on (16 bytes).
    pub list: crate::memory::ListCells,
    /// 0x8DB5B0: `g_marksPlaces`: the places' block (0x7024 bytes; 0 since phase 12: only
    /// counted, `allocate_places`).
    pub places: PlainDword,
    /// 0x8DB5B4: `flt_8DB5B4` .. `flt_8DB5BC`: the places' block's first three floats (`effects::allocate_places`).
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

/// The links' list and block (IDA 0x9206F0 .. 0x920734).
#[derive(rc2k_derive::Cells)]
pub struct MarkLinks {
    /// 0x9206F0: `stru_9206F0` (`g_marksLinksList`): the list the links' block is taken on (16 bytes).
    pub list: crate::memory::ListCells,
    /// 0x920730: `g_marksLinks`: the links' block (0x71020 bytes; 0 since phase 12: only
    /// counted, `allocate_links`).
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
    /// +0: |+0xA - +2| over |+4 - +0| of the corner record, as a float.
    pub slope: PlainFloat,
    /// +4: the corner's four word pairs times `DrawWork::uv_k`.
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

/// The corners' slopes (`g_marksCornerSlopes`: 0x100 records of 0x24 bytes) (IDA 0x920734 .. 0x922B34).
#[derive(rc2k_derive::Cells)]
pub struct CornerSlopes {
    /// 0x920734: `g_marksCornerSlopes`: one record for each corner record.
    pub slopes: [Slope; 0x100],
}

impl CornerSlopes {
    pub const fn new() -> Self {
        Self {
            slopes: [const { Slope::new() }; 0x100],
        }
    }
}

/// A corner record of `g_marksCorners` (0x80 bytes of 16-bit words: four (u, v) pairs of a
/// cell's texture places, and what `effects::scale_corners` scales).
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

/// The marks' corner table (`g_marksCorners`: 0x100 records of 0x80 bytes) (IDA 0x8EB971 .. 0x8F3971).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct MarkCorners {
    /// 0x8EB971: `g_marksCorners`: the corner records (the course's 0x8000 bytes of `markinfo` are copied here; `effects::scale_corners`).
    pub corners: [Corner; 0x100],
}

impl MarkCorners {
    pub const fn new() -> Self {
        Self {
            corners: [const { Corner::new() }; 0x100],
        }
    }
}

/// The particle ring's pool and the burst code's scratch (IDA 0x93A580 .. 0x93A5F0).
#[derive(rc2k_derive::Cells)]
pub struct RingPool {
    /// 0x93A580: `stru_93A580` (`g_marksRingList`): a `memory::List` (head, total, cursor, remaining)
    pub list: crate::memory::ListCells,
    /// 0x93A5C0: `g_marksRingFree`: the first entry of the ring in use
    pub free: PlainDword,
    /// 0x93A5C8: `g_marksRing`: the block taken for the ring
    pub ring: PlainDword,
    /// 0x93A5CC: `g_fireworksRun`: a developer's toggle (the original flips it with a key held in the race; the debug page's "fworks run"). Nothing ported reads or writes it.
    pub fireworks_run: PlainDword,
    /// 0x93A5D0: `dword_93A5D0`: a burst particle's +8 (kind plus 8 bits)
    pub nine: PlainDword,
    /// 0x93A5D4: `dword_93A5D4`: a burst particle's kind times 0x400
    pub kind: PlainDword,
    /// 0x93A5E4: `g_marksBurstRadius`: the burst's radius (`flt_93A5E4`)
    pub burst_radius: PlainFloat,
    /// 0x93A5E8: `dword_93A5E8`: the colour index of the particle being drawn
    pub index: PlainDword,
    /// 0x93A5EC: `g_marksRingUsed`: the entries taken
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
