//! More of `effects::init_marks`' steps (IDA, 2026-10-16): the three blocks it takes
//! (Rust's since phase 12, counted on their lists), the corner table
//! scaled, and the eight settings blocks reset.

#[cfg(test)]
use crate::effects::owner::effects;
use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::session::write_bits;
use crate::session::{Bytes, Dword, Float, Word};

pub const PLACES_LEN: u32 = 0x7024;
pub const RING_LEN: u32 = 0x4C000;
pub const RING_ENTRY: u32 = rc2k_core::effects::rings::RING_ENTRY;
pub const LINKS_LEN: u32 = 0x71020;
pub const CORNERS: u32 = 0x100;

pub const SETTINGS_BLOCKS: u32 = 8;

/// `effects::allocate_places` (0x4C38B1), step 6: 0x7024 bytes on `stru_8DB570` into
/// `g_marksPlaces`, its first three floats `flt_8DB5B4` ... Only code no reference
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

/// The particles (`Particles::ring`; `g_marksRing`, the game's 0x4C000-byte block on `stru_93A580`), Rust's
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

/// `effects::allocate_ring` (0x4CFAC5), step 13: 0x4C000 bytes on `stru_93A580` into
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

/// `effects::allocate_links` (0x4C5398), in step 3: 0x71020 bytes on `stru_9206F0` into
/// `g_marksLinks`, its +0x7100C 0, at +8 and +0x4008 the 0x1000 entries' before and after
/// (round), then `effects::scale_corners`. Only code no reference reaches reads the block
/// (0x4C5446 .. 0x4C5DEA, 0x4C6242: `notes/reference.md`), so since phase 12 it is
/// only counted on its list (the memory figures) and not kept.
pub fn allocate_links(effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    crate::memory::owned::count_on(&effects.blocks.links.list, LINKS_LEN)?;
    scale_corners(effects);
    Ok(())
}

/// The word pairs of a corner record, in the order they are made: +0, +4, +0xC, +8.
const CORNER_PAIRS: [u32; 4] = [0, 4, 0xC, 8];

/// `effects::scale_corners` (0x4C6420): for the 0x100 corner records (0xFF first), the four
/// word pairs (+0, +4, +0xC, +8) each times `flt_8EB969` into the work and into +4 .. +0x20
/// of the record's 0x24 at `g_marksCornerSlopes`; +0 there the slope: |+0xA - +2| over
/// |+4 - +0| (sixteen-bit differences; `dword_8EB8B5`, `dword_8EB8B9`).
pub fn scale_corners(effects: &crate::effects::owner::Effects) {
    {
        let scale: u32 = effects.frame.draw_work.uv_k.bits();
        for n in (0..CORNERS).rev() {
            let record = &effects.blocks.corners.corners[n as usize];
            let word = |o: u32| record.words[o as usize / 2].get();
            let mut values = [0u32; 8];
            for (k, &o) in CORNER_PAIRS.iter().enumerate() {
                for h in 0..2 {
                    let v = x87::int_times(word(o + 2 * h as u32) as i32, scale);
                    values[2 * k + h] = v;
                    let vertex = &crate::d3d::vertices_store::VERTICES.vertices[k];
                    if h == 0 {
                        vertex.u.set_bits(v);
                    } else {
                        vertex.v.set_bits(v);
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
            slope
                .slope
                .set_bits(x87::int_ratio(along as i32, across as i32));
            write_bits(&slope.values, values);
        }
    }
}

/// `effects::reset_settings_blocks` (0x4C9A6D), in step 3: the eight blocks of 0x4C8 at
/// `g_marksSettings` + 0x60 (the last first): +0x4C .. +0x60 the six floats from
/// `flt_9354C6`, the last three again at +0x64, +0x70, +0x7C, +0x30 8.
pub fn reset_settings_blocks(effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    {
        let marks = crate::effects::setup::marks()?;
        for n in (0..SETTINGS_BLOCKS).rev() {
            let car = &marks.cars[n as usize];
            for (cell, v) in car
                .motion
                .iter()
                .chain(&car.steps[0])
                .zip(&effects.kinds.emitter.settings_start)
            {
                cell.set_bits(x87::loaded(v.bits()));
            }
            let last = car.steps[0].each_ref().map(Float::bits);
            for steps in &car.steps[1..] {
                for (cell, v) in steps.iter().zip(last) {
                    cell.set_bits(v);
                }
            }
            car.reset_wait.set(8);
        }
    }
    Ok(())
}

/// The places' list, block and first floats (IDA 0x8DB570 .. 0x8DB5C0, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PlacesBlock {
    /// 0x8DB570: `stru_8DB570` (`g_marksPlacesList`): the list the places' block is taken on (16 bytes).
    pub list: [Dword; 4],
    /// 0x8DB580: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_8db580: Bytes<48>,
    /// 0x8DB5B0: `g_marksPlaces`: the places' block (0x7024 bytes; 0 since phase 12: only
    /// counted, `allocate_places`).
    pub places: Dword,
    /// 0x8DB5B4: `flt_8DB5B4` .. `flt_8DB5BC`: the places' block's first three floats (`effects::allocate_places`).
    pub start: [Float; 3],
}

impl PlacesBlock {
    pub const fn new() -> Self {
        Self {
            list: [
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
            ],
            _gap_8db580: Bytes::new([0; 48]),
            places: Dword::new(0x0),
            start: [
                Float::from_bits(0x0),
                Float::from_bits(0xC8800000),
                Float::from_bits(0x499C3E80),
            ],
        }
    }
}

impl PlacesBlock {
    pub const START: usize = 0x8DB570;
    pub const LEN: usize = 0x50;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const effects().blocks.places) as usize
    }
}

/// The links' list and block (IDA 0x9206F0 .. 0x920734, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarkLinks {
    /// 0x9206F0: `stru_9206F0` (`g_marksLinksList`): the list the links' block is taken on (16 bytes).
    pub list: [Dword; 4],
    /// 0x920700: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_920700: Bytes<48>,
    /// 0x920730: `g_marksLinks`: the links' block (0x71020 bytes; 0 since phase 12: only
    /// counted, `allocate_links`).
    pub links: Dword,
}

impl MarkLinks {
    pub const fn new() -> Self {
        Self {
            list: [
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
            ],
            _gap_920700: Bytes::new([0; 48]),
            links: Dword::new(0x0),
        }
    }
}

impl MarkLinks {
    pub const START: usize = 0x9206F0;
    pub const LEN: usize = 0x44;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const effects().blocks.links) as usize
    }
}

/// A corner's slope record (0x24 bytes): the slope of its sides and its eight scaled values
/// (`effects::scale_corners`; `effects::draw_cell_sprite` scales a sprite by the slope, and
/// `effects::set_quad_uvs` reads record 0x58's eight values as four texture places).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Slope {
    /// +0: |+0xA - +2| over |+4 - +0| of the corner record, as a float.
    pub slope: Float,
    /// +4: the corner's four word pairs times `DrawWork::uv_k`.
    pub values: [Float; 8],
}

impl Slope {
    #[cfg(test)]
    pub const LEN: usize = 0x24;
    pub const fn new() -> Self {
        Slope {
            slope: Float::from_bits(0),
            values: [const { Float::from_bits(0) }; 8],
        }
    }
}

/// The corners' slopes (`g_marksCornerSlopes`: 0x100 records of 0x24 bytes) (IDA 0x920734 .. 0x922B34, the image's layout; phase 7, typed state).
#[repr(C)]
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

impl CornerSlopes {
    pub const START: usize = 0x920734;
    pub const LEN: usize = 0x2400;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const effects().blocks.corner_slopes) as usize
    }
}

/// A corner record of `g_marksCorners` (0x80 bytes of 16-bit words: four (u, v) pairs of a
/// cell's texture places, and what `effects::scale_corners` scales).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Corner {
    pub words: [Word; 0x40],
}

impl Corner {
    #[cfg(test)]
    pub const LEN: usize = 0x80;
    pub const fn new() -> Self {
        Corner {
            words: [const { Word::new(0) }; 0x40],
        }
    }
}

/// The marks' corner table (`g_marksCorners`: 0x100 records of 0x80 bytes) (IDA 0x8EB971 .. 0x8F3971, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
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

impl MarkCorners {
    pub const START: usize = 0x8EB971;
    pub const LEN: usize = 0x8000;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const effects().blocks.corners) as usize
    }
}

/// The particle ring's pool and the burst code's scratch (IDA 0x93A580 .. 0x93A5F0, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RingPool {
    /// 0x93A580: `stru_93A580` (`g_marksRingList`): a `memory::List` (head, total, cursor, remaining)
    pub list: [Dword; 4],
    /// 0x93A590: Not named: 48 bytes (a gap, or unreferenced data)
    pub _gap_93a590: Bytes<48>,
    /// 0x93A5C0: `g_marksRingFree`: the first entry of the ring in use
    pub free: Dword,
    /// 0x93A5C4: Not named: 4 bytes (a gap, or unreferenced data)
    pub _gap_93a5c4: Bytes<4>,
    /// 0x93A5C8: `g_marksRing`: the block taken for the ring
    pub ring: Dword,
    /// 0x93A5CC: Not named: 4 bytes (a gap, or unreferenced data)
    pub _gap_93a5cc: Bytes<4>,
    /// 0x93A5D0: `dword_93A5D0`: a burst particle's +8 (kind plus 8 bits)
    pub nine: Dword,
    /// 0x93A5D4: `dword_93A5D4`: a burst particle's kind times 0x400
    pub kind: Dword,
    /// 0x93A5D8: Not named: 12 bytes (a gap, or unreferenced data)
    pub _gap_93a5d8: Bytes<12>,
    /// 0x93A5E4: `g_marksBurstRadius`: the burst's radius (`flt_93A5E4`)
    pub burst_radius: Float,
    /// 0x93A5E8: `dword_93A5E8`: the colour index of the particle being drawn
    pub index: Dword,
    /// 0x93A5EC: `g_marksRingUsed`: the entries taken
    pub used: Dword,
}

/// RingPool's first values.
impl RingPool {
    pub const fn new() -> Self {
        RingPool {
            list: [
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
            ],
            _gap_93a590: Bytes::new([0; 48]),
            free: Dword::new(0x0),
            _gap_93a5c4: Bytes::new([0; 4]),
            ring: Dword::new(0x0),
            _gap_93a5cc: Bytes::new([0; 4]),
            nine: Dword::new(0x0),
            kind: Dword::new(0x0),
            _gap_93a5d8: Bytes::new([
                0x0D, 0x00, 0x80, 0x39, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]),
            burst_radius: Float::from_bits(0x0),
            index: Dword::new(0x0),
            used: Dword::new(0x0),
        }
    }
}

impl RingPool {
    pub const START: usize = 0x93A580;
    pub const LEN: usize = 0x70;
}

#[cfg(test)]
mod typed_tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn places_block_starts_as_the_games() {
        assert_eq!(size_of::<PlacesBlock>(), PlacesBlock::LEN);
        let image = crate::moved::image_bytes(PlacesBlock::START, PlacesBlock::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(PlacesBlock::home()).cast_const(),
                PlacesBlock::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "PLACES_BLOCK starts differently from the image"
        );
        assert_eq!(
            offset_of!(PlacesBlock, places),
            0x8DB5B0 - PlacesBlock::START
        );
        assert_eq!(
            offset_of!(PlacesBlock, start),
            0x8DB5B4 - PlacesBlock::START
        );
    }

    #[test]
    fn mark_links_starts_as_the_games() {
        assert_eq!(size_of::<MarkLinks>(), MarkLinks::LEN);
        let image = crate::moved::image_bytes(MarkLinks::START, MarkLinks::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(MarkLinks::home()).cast_const(),
                MarkLinks::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "MARK_LINKS starts differently from the image"
        );
        assert_eq!(offset_of!(MarkLinks, links), 0x920730 - MarkLinks::START);
    }

    #[test]
    fn corner_slopes_starts_as_the_games() {
        assert_eq!(size_of::<CornerSlopes>(), CornerSlopes::LEN);
        let image = crate::moved::image_bytes(CornerSlopes::START, CornerSlopes::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(CornerSlopes::home()).cast_const(),
                CornerSlopes::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "CORNER_SLOPES starts differently from the image"
        );
        assert_eq!(size_of::<Slope>(), Slope::LEN);
    }

    #[test]
    fn mark_corners_starts_as_the_games() {
        assert_eq!(size_of::<MarkCorners>(), MarkCorners::LEN);
        let image = crate::moved::image_bytes(MarkCorners::START, MarkCorners::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(MarkCorners::home()).cast_const(),
                MarkCorners::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "MARK_CORNERS starts differently from the image"
        );
        assert_eq!(size_of::<Corner>(), Corner::LEN);
    }
}

#[cfg(test)]
mod ring_pool_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn ring_pool_starts_as_the_games() {
        assert_eq!(size_of::<RingPool>(), RingPool::LEN);
        assert!(
            bytes(&effects().ring.pool)
                == crate::moved::image_bytes(RingPool::START, RingPool::LEN),
            "RING_POOL starts differently from the image"
        );
        assert_eq!(offset_of!(RingPool, list), 0x93A580 - RingPool::START);
        assert_eq!(offset_of!(RingPool, free), 0x93A5C0 - RingPool::START);
        assert_eq!(offset_of!(RingPool, ring), 0x93A5C8 - RingPool::START);
        assert_eq!(offset_of!(RingPool, nine), 0x93A5D0 - RingPool::START);
        assert_eq!(offset_of!(RingPool, kind), 0x93A5D4 - RingPool::START);
        assert_eq!(
            offset_of!(RingPool, burst_radius),
            0x93A5E4 - RingPool::START
        );
        assert_eq!(offset_of!(RingPool, index), 0x93A5E8 - RingPool::START);
        assert_eq!(offset_of!(RingPool, used), 0x93A5EC - RingPool::START);
    }
}
