//! The marks' ring of 0x400 entries of 0x94 bytes (`g_marksRingBlock`, 0x25000 bytes of the
//! game's heap on `stru_935620`, taken by `effects::set_up_surface`): Rust's since phase 12
//! (2026-10-27), counted on the list as the game's block was, given back by
//! `effects::tidy_marks`. An entry is named by its number + 1 (0 none), in `RING_WORK.cursor`
//! / `block` and in the entries' links (+0 the one before, +4 the one after) as the
//! original held their addresses. Each field keeps its offset in the original's record.

use crate::errors::BadInput;
use crate::math::x87_float::{self as bh, X};
use crate::memory::owned::Owned;

/// The entries.
pub const ENTRIES: u32 = 0x400;
/// An entry's bytes.
pub const ENTRY_LEN: u32 = 0x94;
/// The block's bytes, as counted.
pub const RING_BYTES: u32 = 0x2_5000;

/// Byte offsets of an entry's fields (the ones whose meaning is certain, used by the emitters and
/// the ring walks; the rest, and the links at 0 and 4, are read by number).
pub mod field {
    /// The particle's kind (1, 5, 6, 0xB, 0xD, 0x10 ..).
    pub const KIND: u32 = 0x0C;
    /// Its place (floats).
    pub const PLACE_X: u32 = 0x10;
    pub const PLACE_Y: u32 = 0x14;
    pub const PLACE_Z: u32 = 0x18;
    /// Its last place (the streaks), kept for the next frame's segment.
    pub const LAST_PLACE_X: u32 = 0x1C;
    pub const LAST_PLACE_Y: u32 = 0x20;
    pub const LAST_PLACE_Z: u32 = 0x24;
    /// The height of the ground under it: it dies when its y is below this.
    pub const GROUND_Y: u32 = 0x28;
    /// Its motion a frame (floats).
    pub const MOTION_X: u32 = 0x2C;
    pub const MOTION_Y: u32 = 0x30;
    pub const MOTION_Z: u32 = 0x34;
    /// The sprite's angle, and how fast it turns.
    pub const ANGLE: u32 = 0x50;
    pub const ANGLE_RATE: u32 = 0x54;
    /// The sprite's cell in its picture.
    pub const CELL: u32 = 0x58;
    /// The sprite's size, and the size it is pulled toward.
    pub const SIZE: u32 = 0x60;
    pub const SIZE_GOAL: u32 = 0x70;
    /// The car it was emitted for (a `marks_settings_store::car_handle`).
    pub const CAR: u32 = 0x68;
    /// The sprite's alpha, and how much of it goes a tick.
    pub const ALPHA: u32 = 0x74;
    pub const ALPHA_FADE: u32 = 0x78;
    /// The ticks of life left.
    pub const LIFE: u32 = 0x84;
}

pub type Entry = [u32; (ENTRY_LEN / 4) as usize];

pub static RING: Owned<Vec<Entry>> = Owned::new("no marks ring");

/// The ring made: each entry linked to its neighbours, counted on `list`.
pub fn take(list: crate::pointers::GamePtr) -> Result<(), crate::errors::Refused> {
    let ring = (0..ENTRIES)
        .map(|i| {
            let mut e = [0; (ENTRY_LEN / 4) as usize];
            e[0] = (i.wrapping_sub(1) & (ENTRIES - 1)) + 1;
            e[1] = ((i + 1) & (ENTRIES - 1)) + 1;
            e
        })
        .collect();
    RING.take(list, RING_BYTES, ring)
}

/// The dword at `offset` of entry `e`; refused for none, a number past the ring or an
/// offset past the entry.
pub fn get(e: u32, offset: u32) -> Result<u32, BadInput> {
    RING.with(|r| {
        let e = r.get((e as usize).checked_sub(1)?)?;
        offset
            .is_multiple_of(4)
            .then(|| e.get(offset as usize / 4).copied())?
    })?
    .ok_or(BadInput("a ring entry past the ring"))
}

/// The dword at `offset` of entry `e` set to `v`; refused as `get`.
pub fn put(e: u32, offset: u32, v: u32) -> Result<(), BadInput> {
    RING.with(|r| {
        let e = r.get_mut((e as usize).checked_sub(1)?)?;
        let d = offset
            .is_multiple_of(4)
            .then(|| e.get_mut(offset as usize / 4))??;
        *d = v;
        Some(())
    })?
    .ok_or(BadInput("a ring entry past the ring"))
}

/// `fld dword ptr [e + offset]` (the x87 blocks; a refusal ends the game, as their reads of
/// the game's memory did).
pub fn ld(e: u32, offset: u32) -> X {
    bh::ld(get(e, offset).unwrap_or_else(|r| crate::fatal::raise::end("a ring entry", r)))
}

/// `fstp dword ptr [e + offset]` (as `ld`).
pub fn st(e: u32, offset: u32, x: X) {
    if let Err(r) = put(e, offset, bh::st(x)) {
        crate::fatal::raise::end("a ring entry", r)
    }
}

/// `f` on entry 1 of a test ring holding the bytes `lo .. hi` of the game's memory at
/// `base` (the differential tests' operand), the bytes copied back after it.
#[cfg(all(test, target_arch = "x86"))]
pub fn with_entry_at<R>(base: usize, lo: u32, hi: u32, f: impl FnOnce(u32) -> R) -> R {
    use crate::pointers::GamePtr;
    let kept = RING.with(std::mem::take).ok();
    let mut e = [0u32; (ENTRY_LEN / 4) as usize];
    for o in (lo..hi).step_by(4) {
        e[o as usize / 4] = GamePtr::new(base as u32 + o).read("a test entry").unwrap();
    }
    RING.set(vec![e]);
    let out = f(1);
    let e = RING.with(|r| r[0]).unwrap();
    for o in (lo..hi).step_by(4) {
        GamePtr::new(base as u32 + o)
            .write(e[o as usize / 4], "a test entry")
            .unwrap();
    }
    match kept {
        Some(r) => RING.set(r),
        None => RING.give_back(),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring_is_linked_and_read_by_number() {
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let mut memory = vec![0u8; size_of::<crate::memory::MemoryList>()];
        let _region = crate::pointers::Lent::new(&memory);
        take(crate::pointers::GamePtr::new(
            crate::pointers::game_address(memory.as_mut_ptr()),
        ))
        .unwrap();
        assert_eq!(
            u32::from_le_bytes(memory[4..8].try_into().unwrap()),
            RING_BYTES
        );
        assert_eq!(get(1, 0).unwrap(), ENTRIES);
        assert_eq!(get(1, 4).unwrap(), 2);
        assert_eq!(get(ENTRIES, 4).unwrap(), 1);
        put(3, 0x90, 0x1234).unwrap();
        assert_eq!(get(3, 0x90).unwrap(), 0x1234);
        assert!(get(0, 0).is_err());
        assert!(get(ENTRIES + 1, 0).is_err());
        assert!(get(1, ENTRY_LEN).is_err());
        assert!(get(1, 2).is_err());
        RING.give_back();
        crate::memory::with_figures(|f| *f = figures);
    }
}
