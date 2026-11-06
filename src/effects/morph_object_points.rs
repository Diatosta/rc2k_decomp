//! `effects::morph_object_points` (0x4D08DD, reached through the jumps at 0x4D074D and
//! 0x4D0815): for each car (the settings' +0x5C, the last first; its block at +0x29C of
//! its 0x4C8 bytes) the block's +0xC its +8; when its eight factors (+0x90 ..) differ from
//! the kept ones (+0xB0 ..): +0xC -1, the factors kept (also in `dword_93C6D4` ..), and
//! for each of its first object's (+0) points the matching point of the second object
//! (+4) blended toward from the base list (+0x8C) by `effects::blend_corner`, where the
//! base and the second object's point differ. Then `effects::measure_lines`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4D14B4`. Refuses (before anything) null settings, more than 8 cars, and for
//! a car whose factors changed: an object past 0xFF, a null base list or one (`lines::
//! BLOCK_POINTS`, Rust's since phase 12) shorter than the points, points past the
//! object points' room, a point count below 0 (the loop runs on); and `measure_lines`'
//! refusals (all it may write is put back first).

use crate::effects::blend_corner::blend_corner;
use crate::effects::lines::measure_lines;
use crate::effects::paths::Paths;
use crate::errors::{BadInput, Refused};
use crate::race::marks_settings_store::MarkCar;
use crate::race::owner::race;
use crate::scene::object_slots_store::{OBJECT_SLOTS, ObjectTables};
use crate::scene::objects::SLOTS;
use crate::session::{Bytes, Dword};

const BLOCK: u32 = crate::race::marks_settings_store::CAR_BLOCK;
const MOST_CARS: i32 = 8;

/// The cars' blocks, the last first (none for a count of 0 or less).
fn blocks() -> Result<Vec<&'static MarkCar>, Refused> {
    let marks = crate::effects::setup::marks()?;
    Ok({
        let count = marks.cars_copy.get() as i32;
        (0..count.clamp(0, MOST_CARS) as u32)
            .rev()
            .map(|n| &marks.cars[n as usize])
            .collect()
    })
}

/// Whether a block's factors differ from its kept ones.
fn changed(b: &MarkCar) -> Result<bool, BadInput> {
    for k in 0..8 {
        if b.word(BLOCK + 0x90 + 4 * k)? != b.word(BLOCK + 0xB0 + 4 * k)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// A block's objects' first points (in `g_objectPoints`) and their count.
fn points(b: &MarkCar) -> Result<(u32, u32, i32), Refused> {
    Ok({
        let base: u32 = race().course.lists.object_points.bits();
        let first = |id: u32| {
            base.wrapping_add(12 * ObjectTables::slot_value(&OBJECT_SLOTS.first_point, id as usize))
        };
        let (a, c) = (b.word(BLOCK)?, b.word(BLOCK + 4)?);
        (
            first(a),
            first(c),
            ObjectTables::slot_value(&OBJECT_SLOTS.point_counts, a as usize) as i32,
        )
    })
}

/// Why the Rust version refuses, if it does (its own checks), worked out without writing.
fn refusal(paths: &Paths) -> Result<Option<&'static str>, Refused> {
    let course = &race().course;
    let marks = crate::effects::setup::marks()?;
    Ok({
        if marks.cars_copy.get() as i32 > MOST_CARS {
            return Ok(Some("more than 8 cars"));
        }
        let base: u32 = course.lists.object_points.bits();
        let room: u32 = course.lists.object_points_room.bits();
        for b in blocks()? {
            if !changed(b)? {
                continue;
            }
            if b.word(BLOCK)? >= SLOTS || b.word(BLOCK + 4)? >= SLOTS {
                return Ok(Some("an object past 0xFF"));
            }
            if b.word(BLOCK + 0x8C)? == 0 {
                return Ok(Some("a null base point list"));
            }
            let (a, c, n) = points(b)?;
            if n < 0 {
                return Ok(Some("a negative point count: the loop runs on"));
            }
            if n > 0
                && crate::effects::lines::block_point(paths, b.word(BLOCK + 0x8C)?, n as u32 - 1)
                    .is_err()
            {
                return Ok(Some("a base point list shorter than the object's points"));
            }
            let end = base.wrapping_add(room);
            for p in [a, c] {
                if p < base || p.wrapping_add(12 * n as u32) > end {
                    return Ok(Some("points past the object points' room"));
                }
            }
        }
        None
    })
}

/// The Rust version (see the header).
pub fn morph_object_points(paths: &Paths) -> Result<(), Refused> {
    if let Some(why) = refusal(paths)? {
        return Err(BadInput(why).into());
    }
    {
        for b in blocks()? {
            b.put(BLOCK + 0xC, b.word(BLOCK + 8)?)?;
            if crate::effects::try_all(0..8, |k| {
                Ok(b.word(BLOCK + 0x90 + 4 * k)? == b.word(BLOCK + 0xB0 + 4 * k)?)
            })? {
                continue;
            }
            b.put(BLOCK + 0xC, 0xFFFF_FFFF)?;
            for k in 0..8 {
                let v = b.word(BLOCK + 0x90 + 4 * k)?;
                b.put(BLOCK + 0xB0 + 4 * k, v)?;
                paths.morph_work.kept[k as usize].set(v);
            }
            let (a, c, n) = points(b)?;
            let points = b.word(BLOCK + 0x8C)?;
            for k in 0..n.max(0) as u32 {
                let (to, from) = (a + 12 * k, c + 12 * k);
                let base = crate::effects::lines::block_point(paths, points, k)?;
                if crate::effects::try_any(0..3, |i| {
                    Ok(base[i as usize] != crate::scene::object_points::word(from + 4 * i)?)
                })? {
                    blend_corner(b, to, from, base)?;
                }
            }
        }
        measure_lines(paths)?;
    }
    Ok(())
}

/// The object morph's work (IDA 0x93C6CC .. 0x93C6F4, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MorphWork {
    /// 0x93C6CC: `g_marksTaken`: the block last taken for a settings block's points (`lines::set_up_blocks`)
    pub taken: Dword,
    /// 0x93C6D0: Not named: 4 bytes (a gap, or unreferenced data)
    pub _gap_93c6d0: Bytes<4>,
    /// 0x93C6D4: `dword_93C6D4` .. `dword_93C6F0`: the factors kept
    pub kept: [Dword; 8],
}

impl MorphWork {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        MorphWork {
            taken: Dword::new(0x0),
            _gap_93c6d0: Bytes::new([0; 4]),
            kept: [
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
            ],
        }
    }
    pub const START: usize = 0x93C6CC;
    pub const LEN: usize = 0x28;
}

#[cfg(test)]
mod morph_work_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn morph_work_starts_as_the_games() {
        assert_eq!(size_of::<MorphWork>(), MorphWork::LEN);
        assert!(
            bytes(&crate::effects::owner::EFFECTS.paths.morph_work)
                == crate::moved::image_bytes(MorphWork::START, MorphWork::LEN),
            "MORPH_WORK starts differently from the image"
        );
        assert_eq!(offset_of!(MorphWork, taken), 0x93C6CC - MorphWork::START);
        assert_eq!(offset_of!(MorphWork, kept), 0x93C6D4 - MorphWork::START);
    }
}
