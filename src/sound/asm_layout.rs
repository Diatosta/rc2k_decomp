//! The 1999 layout of the sound stores, as the differential tests of the kept x87 assembly need it.
//! The stores hold their fields in a natural order with nothing between them; the assembly was
//! recorded against the 1999 program's own bytes, where unnamed dwords sit between some of the fields. Here
//! each store's runs of fields are listed with the offset (`at`) the assembly reads them at, so a
//! test copies the fields in from a buffer laid out as the 1999 variables was, runs the Rust
//! version on them, and copies them back. The bytes between the runs stay as the buffer had them:
//! nothing typed reads them. Test code only; it goes with the recorded vectors.

use crate::sound::sets_store::SoundRequest;
use crate::sound::store::{
    EngineVoice, SoundObject, SoundObjectCell, SoundSlot, SurfaceConstants, SurfaceSounds,
    SurfaceWork,
};
use rc2k_core::sound::cars::{Cue, CueQueue};
use std::cell::RefCell;

/// A run of fields that were next to each other in the 1999 layout.
pub struct Run {
    /// Its offset from the object's first byte in the 1999 layout.
    pub at: usize,
    /// The first field of the run, named by the value it holds when each word of the object holds
    /// its own number (see `typed_start`).
    pub first: fn(&SoundObject) -> u32,
    /// Its length in bytes.
    pub len: usize,
}

/// The first `len` bytes of the test buffer at `base`.
///
/// # Safety
/// `base` is the address of a buffer of `Operand::Sized { len, .. }` of the
/// vectors' harness, which stays alive and is touched by nothing else while the block runs.
unsafe fn buffer<'a>(base: usize, len: usize) -> &'a mut [u8] {
    // SAFETY: as the contract says; the buffer is `len` bytes at least.
    unsafe { core::slice::from_raw_parts_mut(base as *mut u8, len) }
}

/// A plain value as words, in field order.
trait Words {
    fn put(&self, out: &mut Vec<u32>);
    fn take(&mut self, from: &mut impl Iterator<Item = u32>);
}

impl Words for u32 {
    fn put(&self, out: &mut Vec<u32>) {
        out.push(*self);
    }
    fn take(&mut self, from: &mut impl Iterator<Item = u32>) {
        *self = from.next().expect("a word");
    }
}

impl Words for f32 {
    fn put(&self, out: &mut Vec<u32>) {
        out.push(self.to_bits());
    }
    fn take(&mut self, from: &mut impl Iterator<Item = u32>) {
        *self = f32::from_bits(from.next().expect("a word"));
    }
}

impl<T: Words, const N: usize> Words for [T; N] {
    fn put(&self, out: &mut Vec<u32>) {
        self.iter().for_each(|x| x.put(out));
    }
    fn take(&mut self, from: &mut impl Iterator<Item = u32>) {
        self.iter_mut().for_each(|x| x.take(from));
    }
}

macro_rules! words_of {
    ($t:ty { $($f:ident),* $(,)? }) => {
        impl Words for $t {
            fn put(&self, out: &mut Vec<u32>) {
                $(self.$f.put(out);)*
            }
            fn take(&mut self, from: &mut impl Iterator<Item = u32>) {
                $(self.$f.take(from);)*
            }
        }
    };
}

words_of!(EngineVoice {
    share_a,
    share_b,
    share_c,
    surface
});
words_of!(SoundSlot {
    sound,
    start,
    playing,
    ends,
    volume,
    rate
});
words_of!(Cue {
    first,
    count,
    sounds
});
words_of!(CueQueue {
    first,
    place,
    due,
    count,
    cues
});
words_of!(SoundObject {
    sound_offset,
    kind,
    in_use,
    view_flag,
    new_revs,
    revs,
    throttle,
    speed,
    gear_event,
    no_gear,
    place,
    motion,
    voices,
    reading_times,
    reading_revs,
    reserved_a4,
    flag_ac,
    effect_in,
    knock,
    knock_in,
    steer_trim_copy,
    effect_request_b,
    knock_count,
    gear,
    last_gear,
    gear_notice,
    countdown,
    drop_latched,
    last_direction,
    revs_input,
    throttle_input,
    speed_input,
    state,
    last_state,
    smooth_throttle,
    throttle_lead,
    turns_up,
    direction,
    turns_down,
    distance,
    approach,
    pitch_factor,
    volume_factor,
    eights_sound,
    fours_sound,
    slots,
    backfire_wait,
    gear_slot,
    gear_pitch,
    place_b,
    slot_b,
    hit_sound,
    hit_volume,
    hit_place,
    cue_sound,
    cue_volume,
    cue_place,
    slot_c,
    landed,
    last_forward_speed,
    reserved_280,
    sum_a,
    sum_b,
    sum_c,
    surface,
    cue_queue
});

/// The object's words in field order.
fn to_words(o: &SoundObject) -> Vec<u32> {
    let mut words = Vec::new();
    o.put(&mut words);
    words
}

/// An object made from `words`, in field order.
fn from_words(words: &[u32]) -> SoundObject {
    let mut o = crate::sound::store::initial_object(0);
    let mut it = words.iter().copied();
    o.take(&mut it);
    assert!(it.next().is_none(), "words left over");
    o
}

/// How many words the object has.
fn word_count() -> usize {
    to_words(&crate::sound::store::initial_object(0)).len()
}

/// The place in the object's words (in field order) of the field `first` names.
pub fn typed_start(first: fn(&SoundObject) -> u32) -> usize {
    let probe: Vec<u32> = (0..word_count() as u32).collect();
    first(&from_words(&probe)) as usize
}

/// The length of the 1999 object, gaps included.
pub const OBJECT_EXTENT: usize = 0x4A8;

/// Runs `f` on a sound object made from the bytes the test buffer at `base` holds at the 1999
/// places of its fields (the operand is `Operand::Sized { len: OBJECT_EXTENT, .. }`), and puts the
/// object's fields back into the buffer afterwards.
pub fn with_object<R>(base: usize, f: impl FnOnce(&SoundObjectCell) -> R) -> R {
    // SAFETY: the operand is a sized buffer of the vectors' harness (see `buffer`).
    let bytes = unsafe { buffer(base, OBJECT_EXTENT) };
    let mut words = to_words(&crate::sound::store::initial_object(0));
    for r in RUNS {
        let start = typed_start(r.first);
        for k in 0..r.len / 4 {
            words[start + k] = word(bytes, r.at + 4 * k);
        }
    }
    let local = RefCell::new(from_words(&words));
    let out = f(&local);
    let words = to_words(&local.borrow());
    for r in RUNS {
        let start = typed_start(r.first);
        for k in 0..r.len / 4 {
            bytes[r.at + 4 * k..r.at + 4 * k + 4].copy_from_slice(&words[start + k].to_le_bytes());
        }
    }
    out
}

/// The offset in the 1999 layout of the word the field `first` names.
pub fn image_offset(first: fn(&SoundObject) -> u32) -> usize {
    let typed = typed_start(first);
    let mut from = 0;
    for r in RUNS {
        if typed < from + r.len / 4 {
            return r.at + 4 * (typed - from);
        }
        from += r.len / 4;
    }
    panic!("the field lies in no run");
}

/// The length of the 1999 surface-sounds variables, gaps included.
pub const SURFACE_EXTENT: usize = 0x44;

fn word(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().expect("four bytes"))
}

/// Runs `f` on a `SurfaceSounds` made from the bytes the test buffer at `base` holds at the
/// 1999 places of its fields (the operand is `Operand::Sized { len: SURFACE_EXTENT, .. }`), and
/// puts the work fields back into the buffer afterwards.
pub fn with_surface<R>(base: usize, f: impl FnOnce(&SurfaceSounds) -> R) -> R {
    // SAFETY: the operand is a sized buffer of the vectors' harness (see `buffer`).
    let bytes = unsafe { buffer(base, SURFACE_EXTENT) };
    let float = |at| f32::from_bits(word(bytes, at));
    let local = SurfaceSounds {
        constants: SurfaceConstants {
            volume_scale: float(0x4),
            pitch_blend: float(0x8),
            across_scale: float(0xC),
            along_scale: float(0x10),
            speed_scale: float(0x18),
            speed_gain: float(0x24),
            revs_gain: float(0x28),
        },
        work: RefCell::new(SurfaceWork {
            tables_done: word(bytes, 0),
            across: float(0x34),
            along: float(0x3C),
            volume: float(0x40),
        }),
    };
    let out = f(&local);
    let work = local.work.borrow();
    bytes[0..4].copy_from_slice(&work.tables_done.to_le_bytes());
    bytes[0x34..0x38].copy_from_slice(&work.across.to_bits().to_le_bytes());
    bytes[0x3C..0x40].copy_from_slice(&work.along.to_bits().to_le_bytes());
    bytes[0x40..0x44].copy_from_slice(&work.volume.to_bits().to_le_bytes());
    out
}

/// The length of the 1999 request variables.
pub const REQUEST_EXTENT: usize = 40;

/// Runs `f` on a `SoundRequest` made from the ten dwords of the test buffer at `base` (the operand
/// is `Operand::Sized { len: REQUEST_EXTENT, .. }`), and puts the request's dwords back
/// afterwards.
pub fn with_request<R>(base: usize, f: impl FnOnce(&mut SoundRequest) -> R) -> R {
    // SAFETY: the operand is a sized buffer of the vectors' harness (see `buffer`).
    let bytes = unsafe { buffer(base, REQUEST_EXTENT) };
    let mut request = SoundRequest::new();
    for n in 0..10 {
        let bits = word(bytes, 4 * n);
        request.set_word(n, bits).expect("one of the ten words");
    }
    let out = f(&mut request);
    let words = [
        request.sound,
        request.pitch,
        request.volume,
        request.place[0].to_bits(),
        request.place[1].to_bits(),
        request.place[2].to_bits(),
        request.angle,
        request.pan,
        request.pitch_factor.to_bits(),
        request.volume_factor.to_bits(),
    ];
    for (n, w) in words.into_iter().enumerate() {
        bytes[4 * n..4 * n + 4].copy_from_slice(&w.to_le_bytes());
    }
    out
}

/// The runs of the object's fields in the 1999 layout; the gaps are words nothing reads or writes.
const RUNS: &[Run] = &[
    Run {
        at: 0x0,
        first: |o| o.sound_offset,
        len: 0x28,
    },
    Run {
        at: 0x2C,
        first: |o| o.place[0].to_bits(),
        len: 0x18,
    },
    Run {
        at: 0x44,
        first: |o| o.voices[0].share_a.to_bits(),
        len: 0x4,
    },
    Run {
        at: 0x4C,
        first: |o| o.voices[0].share_b.to_bits(),
        len: 0xC,
    },
    Run {
        at: 0x58,
        first: |o| o.voices[1].share_a.to_bits(),
        len: 0x4,
    },
    Run {
        at: 0x60,
        first: |o| o.voices[1].share_b.to_bits(),
        len: 0xC,
    },
    Run {
        at: 0x6C,
        first: |o| o.voices[2].share_a.to_bits(),
        len: 0x4,
    },
    Run {
        at: 0x74,
        first: |o| o.voices[2].share_b.to_bits(),
        len: 0xC,
    },
    Run {
        at: 0x80,
        first: |o| o.voices[3].share_a.to_bits(),
        len: 0x4,
    },
    Run {
        at: 0x88,
        first: |o| o.voices[3].share_b.to_bits(),
        len: 0xC,
    },
    Run {
        at: 0x94,
        first: |o| o.reading_times[0],
        len: 0xE0 - 0x94,
    },
    // 0xE0: two dwords nothing reads or writes.
    Run {
        at: 0xE8,
        first: |o| o.gear,
        len: 0x114 - 0xE8,
    },
    // 0x114: one dword nothing reads or writes.
    Run {
        at: 0x118,
        first: |o| o.smooth_throttle,
        len: 0x8,
    },
    // 0x120: two dwords nothing reads or writes.
    Run {
        at: 0x128,
        first: |o| o.turns_up,
        len: 0x288 - 0x128,
    },
    // 0x288: one dword nothing reads or writes.
    Run {
        at: 0x28C,
        first: |o| o.sum_b,
        len: 0x4A8 - 0x28C,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The runs are the whole object, in order, with nothing between them, and lie in the 1999
    /// extent without overlap.
    #[test]
    fn runs_cover_the_object() {
        let mut next = 0;
        let mut image = 0;
        for r in RUNS {
            assert_eq!(typed_start(r.first), next);
            assert!(r.at >= image);
            next += r.len / 4;
            image = r.at + r.len;
        }
        assert_eq!(next, word_count());
        assert!(image <= OBJECT_EXTENT);
    }

    /// The engine's dwords sit where the core's offsets had them in the sound object.
    #[test]
    fn the_engine_words_sit_where_the_offsets_were() {
        let at = image_offset;
        assert_eq!(
            [
                at(|o| o.new_revs),
                at(|o| o.revs),
                at(|o| o.throttle),
                at(|o| o.gear_event),
                at(|o| o.no_gear),
                at(|o| o.reading_times[0]),
                at(|o| o.reading_times[1]),
                at(|o| o.reading_revs[0]),
                at(|o| o.reading_revs[1]),
                at(|o| o.gear),
                at(|o| o.last_gear),
                at(|o| o.gear_notice),
                at(|o| o.countdown),
                at(|o| o.drop_latched),
                at(|o| o.last_direction),
                at(|o| o.state),
                at(|o| o.last_state),
                at(|o| o.smooth_throttle),
                at(|o| o.throttle_lead),
                at(|o| o.turns_up),
                at(|o| o.direction),
                at(|o| o.turns_down),
                at(|o| o.slots[5].sound),
                at(|o| o.slots[5].start),
                at(|o| o.slots[6].start),
                at(|o| o.slots[6].playing),
                at(|o| o.slots[6].volume),
                at(|o| o.backfire_wait),
                at(|o| o.gear_slot.start),
                at(|o| o.voices[2].surface),
                at(|o| o.surface),
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
