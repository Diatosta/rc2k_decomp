//! The 1999 layout of the sound stores, as the differential tests of the kept x87 assembly
//! need it. The stores hold their fields in a natural order with nothing between them; the
//! assembly was recorded against RAL.EXE's own bytes, where unnamed dwords sit between some
//! of the fields. Here each store's runs of fields are listed with the offset (`at`) the
//! assembly reads them at, so a test copies the fields in from a buffer laid out as the image
//! was, runs the Rust version on them, and copies them back. The bytes between the runs stay
//! as the buffer had them: nothing typed reads them. Test code only; it goes with the
//! recorded vectors.

use crate::cells::{ByteView, load_cells, store_cells};
use crate::sound::store::{EngineVoice, SoundObject, SurfaceSounds};
use core::mem::{offset_of, size_of};

/// A run of fields that were next to each other in the 1999 layout.
pub struct Run {
    /// Its offset from the store's first byte in the 1999 layout.
    pub at: usize,
    /// Its offset in the typed store.
    pub to: usize,
    /// Its length.
    pub len: usize,
}

/// A store with a 1999 layout.
pub trait Laid: ByteView + Sized {
    /// The store's length in the 1999 layout, gaps included.
    const EXTENT: usize;
    /// The runs of fields, in order.
    const RUNS: &'static [Run];
    /// A store with first values, every field of which the runs overwrite.
    fn fresh() -> Box<Self>;
}

/// The first `T::EXTENT` bytes of the test buffer at `base`.
///
/// # Safety
/// `base` is the address of a buffer of `Operand::Sized { len: T::EXTENT, .. }` of the
/// vectors' harness, which stays alive and is touched by nothing else while the block runs.
unsafe fn buffer<'a, T: Laid>(base: usize) -> &'a mut [u8] {
    // SAFETY: as the contract says; the buffer is `T::EXTENT` bytes at least.
    unsafe { core::slice::from_raw_parts_mut(base as *mut u8, T::EXTENT) }
}

/// Runs `f` on a store `T` made from the bytes the test buffer at `base` holds at the 1999
/// places of its fields (the operand is `Operand::Sized { len: T::EXTENT, .. }`), and puts the
/// store's fields back into the buffer afterwards.
pub fn with_laid<T: Laid, R>(base: usize, f: impl FnOnce(&T) -> R) -> R {
    // SAFETY: the operand is a sized buffer of the vectors' harness (see `buffer`).
    let bytes = unsafe { buffer::<T>(base) };
    let local = T::fresh();
    for r in T::RUNS {
        store_cells(&*local, r.to, &bytes[r.at..r.at + r.len]).expect("a run inside the store");
    }
    let out = f(&local);
    for r in T::RUNS {
        load_cells(&*local, r.to, &mut bytes[r.at..r.at + r.len]).expect("a run inside the store");
    }
    out
}

/// The offset in the 1999 layout of the byte at `typed` in the store `T`.
pub fn image_offset<T: Laid>(typed: usize) -> usize {
    let r = T::RUNS
        .iter()
        .find(|r| (r.to..r.to + r.len).contains(&typed))
        .expect("a byte of a field");
    r.at + (typed - r.to)
}

impl Laid for SurfaceSounds {
    const EXTENT: usize = 0x44;
    const RUNS: &'static [Run] = &[
        Run {
            at: 0x0,
            to: 0,
            len: 0x38,
        },
        // 0x38: one dword nothing reads or writes.
        Run {
            at: 0x3C,
            to: offset_of!(SurfaceSounds, along),
            len: 0x8,
        },
    ];
    fn fresh() -> Box<Self> {
        Box::new(SurfaceSounds::new())
    }
}

/// One wheel record's runs: its first dword, then (after a dword nothing reads) the other four.
const fn voice(n: usize) -> [Run; 2] {
    let at = 0x44 + 0x14 * n;
    let to = offset_of!(SoundObject, voices) + size_of::<EngineVoice>() * n;
    [
        Run { at, to, len: 0x4 },
        Run {
            at: at + 0x8,
            to: to + offset_of!(EngineVoice, share_b),
            len: 0xC,
        },
    ]
}

const fn object_runs() -> [Run; 15] {
    let [v0a, v0b] = voice(0);
    let [v1a, v1b] = voice(1);
    let [v2a, v2b] = voice(2);
    let [v3a, v3b] = voice(3);
    [
        Run {
            at: 0x0,
            to: 0,
            len: 0x28,
        },
        // 0x28: one dword nothing reads or writes.
        Run {
            at: 0x2C,
            to: offset_of!(SoundObject, place),
            len: 0x18,
        },
        v0a,
        v0b,
        v1a,
        v1b,
        v2a,
        v2b,
        v3a,
        v3b,
        Run {
            at: 0x94,
            to: offset_of!(SoundObject, reading_times),
            len: 0xE0 - 0x94,
        },
        // 0xE0: two dwords nothing reads or writes.
        Run {
            at: 0xE8,
            to: offset_of!(SoundObject, gear),
            len: 0x114 - 0xE8,
        },
        // 0x114: one dword nothing reads or writes.
        Run {
            at: 0x118,
            to: offset_of!(SoundObject, smooth_throttle),
            len: 0x8,
        },
        // 0x120: two dwords nothing reads or writes.
        Run {
            at: 0x128,
            to: offset_of!(SoundObject, turns_up),
            len: 0x288 - 0x128,
        },
        // 0x288: one dword nothing reads or writes.
        Run {
            at: 0x28C,
            to: offset_of!(SoundObject, sum_b),
            len: 0x4A8 - 0x28C,
        },
    ]
}

impl Laid for SoundObject {
    const EXTENT: usize = 0x4A8;
    const RUNS: &'static [Run] = &object_runs();
    fn fresh() -> Box<Self> {
        Box::new(crate::sound::store::initial_object(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The runs are the whole store, in order, with nothing between them, and lie in the
    /// 1999 extent without overlap.
    fn covered<T: Laid>() {
        let mut next = 0;
        let mut image = 0;
        for r in T::RUNS.iter().filter(|r| r.len != 0) {
            assert_eq!(r.to, next, "{}", std::any::type_name::<T>());
            assert!(r.at >= image, "{}", std::any::type_name::<T>());
            next += r.len;
            image = r.at + r.len;
        }
        assert_eq!(next, size_of::<T>(), "{}", std::any::type_name::<T>());
        assert!(image <= T::EXTENT);
    }

    #[test]
    fn runs_cover_the_stores() {
        covered::<SoundObject>();
        covered::<SurfaceSounds>();
    }

    /// The engine's dwords sit where the core's offsets had them in the sound object.
    #[test]
    fn the_engine_words_sit_where_the_offsets_were() {
        let o = SoundObject::of(crate::sound::owner::sound(), 0).unwrap();
        let base = core::ptr::from_ref(o).addr();
        let at = |cell: &crate::session::PlainDword| {
            image_offset::<SoundObject>(core::ptr::from_ref(cell).addr() - base)
        };
        assert_eq!(
            [
                at(&o.new_revs),
                at(&o.revs),
                at(&o.throttle),
                at(&o.gear_event),
                at(&o.no_gear),
                at(&o.reading_times[0]),
                at(&o.reading_times[1]),
                at(&o.reading_revs[0]),
                at(&o.reading_revs[1]),
                at(&o.gear),
                at(&o.last_gear),
                at(&o.gear_notice),
                at(&o.countdown),
                at(&o.drop_latched),
                at(&o.last_direction),
                at(&o.state),
                at(&o.last_state),
                at(&o.smooth_throttle),
                at(&o.throttle_lead),
                at(&o.turns_up),
                at(&o.direction),
                at(&o.turns_down),
                at(&o.slots[5].sound),
                at(&o.slots[5].start),
                at(&o.slots[6].start),
                at(&o.slots[6].playing),
                at(&o.slots[6].volume),
                at(&o.backfire_wait),
                at(&o.gear_slot.start),
                at(&o.voices[2].surface),
                at(&o.surface),
            ],
            [
                0x10,
                0x14,
                0x18,
                0x20,
                0x24,
                0x94,
                0x98,
                0x9C,
                0xA0,
                0xE8,
                0xEC,
                0xF0,
                0xF4,
                0xF8,
                0xFC,
                0x10C,
                0x110,
                0x118,
                0x11C,
                0x128,
                0x12C,
                0x130,
                0x1C4,
                0x1C8,
                0x1E0,
                0x1E4,
                0x1EC,
                0x1F4,
                0x1FC,
                0x44 + 2 * 0x14 + 0x10,
                0x294
            ]
        );
    }
}
