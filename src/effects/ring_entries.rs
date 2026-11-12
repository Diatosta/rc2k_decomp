//! The effects' ring of 0x400 entries of 0x94 bytes (0x25000 bytes, taken by
//! `effects::set_up_surface`), counted on the ring list as the game's block was, given back
//! by `effects::tidy_marks`. An entry is named by its number + 1 (0 none), in `RING_WORK.cursor`
//! / `block` and in the entries' links (+0 the one before, +4 the one after) as the
//! original held their addresses. Each field keeps its offset in the original's record.

#[cfg(test)]
use crate::effects::owner::effects;
use crate::effects::ring_store::Ring;
use crate::errors::BadInput;
use crate::math::x87_float::{self as bh, X};
use std::mem::size_of;

/// The entries.
pub const ENTRIES: u32 = 0x400;
/// An entry's bytes.
pub const ENTRY_LEN: u32 = 0x94;
/// The block's bytes, as counted.
pub const RING_BYTES: u32 = 0x2_5000;

/// One ring entry: 37 dwords with the offsets of the original's record (the size, alpha and
/// pull fields stay dwords: kinds differ in whether they hold a float or a whole number). The three-axis fields are arrays x, y, z.
/// Fields no code reads or writes by name are `unknown_<offset>`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Entry {
    /// 0x00: The link to the entry before it in the ring (an entry number + 1).
    pub prev: u32,
    /// 0x04: The link to the entry after it.
    pub next: u32,
    /// 0x08: The emitter's tag (`EMITTER.tag`) at the time it was taken.
    pub emitter_tag: u32,
    /// 0x0C: The particle's kind (1, 5, 6, 0xB, 0xD, 0x10 ..).
    pub kind: u32,
    /// 0x10: Its place.
    pub place: [f32; 3],
    /// 0x1C: Its last place (the streaks), kept for the next frame's segment.
    pub last_place: [f32; 3],
    /// 0x28: The height of the ground under it: it dies when its y is below this.
    pub ground_y: f32,
    /// 0x2C: Its motion a frame. It lies 0x1C after the place, as the place plus
    /// the motion times the ticks is how it moves.
    pub motion: [f32; 3],
    /// 0x38: The burst particles' extra push (three floats; the car's +0x4C .. +0x54 times
    /// minus an eighth, x first).
    pub push: [f32; 3],
    /// 0x44: Three dwords no emitter sets (0 from the emitter that clears them).
    pub spare: [u32; 3],
    /// 0x50: The sprite's angle.
    pub angle: u32,
    /// 0x54: How fast the angle turns.
    pub angle_rate: u32,
    /// 0x58: The sprite's cell in its picture.
    pub cell: u32,
    /// 0x5C: How fast the sprite's cell advances with its life (read by the draw of kinds
    /// 0x10, 0x11; no emitter writes it, so it is 0 as the ring takes it).
    pub cell_rate: f32,
    /// 0x60: The sprite's size.
    pub size: u32,
    /// 0x64: Kind 1's size factor (a float): the size grows by it times `k1_size`; pulled
    /// toward 65536 while the entry is near the car and faces it.
    pub size_factor: f32,
    /// 0x68: The car it was emitted for (a `effects_settings_store::car_handle`).
    pub car: u32,
    /// 0x6C: The emitting car's +0x88 (its pace), kept by the emitters of kinds 6, 0xD, 0x15.
    pub car_pace: u32,
    /// 0x70: The size it is pulled toward.
    pub size_goal: u32,
    /// 0x74: The sprite's alpha.
    pub alpha: u32,
    /// 0x78: How much of the alpha goes a tick.
    pub alpha_fade: f32,
    /// 0x7C: The limits the object's box extents are tested against (kinds 6, 0x15, 0x10,
    /// 0x11); no emitter writes them, so they are 0 as the ring takes it.
    pub box_limit: [u32; 2],
    /// 0x84: The ticks of life left.
    pub life: u32,
    /// 0x88: no emitter writes it and no ring step reads it (only the tests' word copies touch it); an unused word.
    pub unused_88: u32,
    /// 0x8C: as 0x88, an unused word.
    pub unused_8c: u32,
    /// 0x90: The emitting car's tag (its object's +0x1C, 24 bits), for the draw lists.
    pub car_tag: u32,
}

const _: () = assert!(size_of::<Entry>() == ENTRY_LEN as usize);

#[cfg(test)]
impl Entry {
    /// The entry's dwords in order of offset.
    pub fn words(&self) -> [u32; (ENTRY_LEN / 4) as usize] {
        let mut w = [0; (ENTRY_LEN / 4) as usize];
        let mut n = 0;
        let mut push = |v: &[u32]| {
            w[n..n + v.len()].copy_from_slice(v);
            n += v.len();
        };
        push(&[self.prev, self.next, self.emitter_tag, self.kind]);
        push(&self.place.map(f32::to_bits));
        push(&self.last_place.map(f32::to_bits));
        push(&[self.ground_y.to_bits()]);
        push(&self.motion.map(f32::to_bits));
        push(&self.push.map(f32::to_bits));
        push(&self.spare);
        push(&[
            self.angle,
            self.angle_rate,
            self.cell,
            self.cell_rate.to_bits(),
            self.size,
            self.size_factor.to_bits(),
            self.car,
            self.car_pace,
            self.size_goal,
            self.alpha,
            self.alpha_fade.to_bits(),
        ]);
        push(&self.box_limit);
        push(&[self.life, self.unused_88, self.unused_8c, self.car_tag]);
        assert_eq!(n, w.len());
        w
    }

    /// An entry from its dwords in order of offset.
    pub fn from_words(w: [u32; (ENTRY_LEN / 4) as usize]) -> Entry {
        Entry {
            prev: w[0],
            next: w[1],
            emitter_tag: w[2],
            kind: w[3],
            place: [w[4], w[5], w[6]].map(f32::from_bits),
            last_place: [w[7], w[8], w[9]].map(f32::from_bits),
            ground_y: f32::from_bits(w[10]),
            motion: [w[11], w[12], w[13]].map(f32::from_bits),
            push: [w[14], w[15], w[16]].map(f32::from_bits),
            spare: [w[17], w[18], w[19]],
            angle: w[20],
            angle_rate: w[21],
            cell: w[22],
            cell_rate: f32::from_bits(w[23]),
            size: w[24],
            size_factor: f32::from_bits(w[25]),
            car: w[26],
            car_pace: w[27],
            size_goal: w[28],
            alpha: w[29],
            alpha_fade: f32::from_bits(w[30]),
            box_limit: [w[31], w[32]],
            life: w[33],
            unused_88: w[34],
            unused_8c: w[35],
            car_tag: w[36],
        }
    }
}

/// The ring made: each entry linked to its neighbours, counted on `list`.
pub fn take(ring: &Ring, list: &dyn crate::cells::Store) -> Result<(), crate::errors::Refused> {
    let entries = (0..ENTRIES)
        .map(|i| Entry {
            prev: (i.wrapping_sub(1) & (ENTRIES - 1)) + 1,
            next: ((i + 1) & (ENTRIES - 1)) + 1,
            ..Entry::default()
        })
        .collect();
    ring.entries.take(list, RING_BYTES, entries)
}

/// Writes into one entry, refusing as `set`.
pub struct Slot<'a> {
    ring: &'a Ring,
    e: u32,
}

impl<'a> Slot<'a> {
    pub fn new(ring: &'a Ring, e: u32) -> Self {
        Slot { ring, e }
    }

    /// The field `field` selects set to `v`.
    pub fn put<T>(&self, field: impl FnOnce(&mut Entry) -> &mut T, v: T) -> Result<(), BadInput> {
        set(self.ring, self.e, v, field)
    }
}

/// A copy of entry `e`; refused for none or a number past the ring.
pub fn entry(ring: &Ring, e: u32) -> Result<Entry, BadInput> {
    ring.entries
        .with(|r| r.get((e as usize).checked_sub(1)?).copied())?
        .ok_or(BadInput("a ring entry past the ring"))
}

/// What `field` reads of entry `e`; refused as `entry`.
pub fn get<T>(ring: &Ring, e: u32, field: impl FnOnce(&Entry) -> T) -> Result<T, BadInput> {
    Ok(field(&entry(ring, e)?))
}

/// The field `field` selects of entry `e` set to `v`; refused as `entry`.
pub fn set<T>(
    ring: &Ring,
    e: u32,
    v: T,
    field: impl FnOnce(&mut Entry) -> &mut T,
) -> Result<(), BadInput> {
    ring.entries
        .with(|r| {
            *field(r.get_mut((e as usize).checked_sub(1)?)?) = v;
            Some(())
        })?
        .ok_or(BadInput("a ring entry past the ring"))
}

/// A field loaded on the x87 stack (a refusal ends the game, as the reads of the game's
/// memory did).
pub fn ld<T: bh::Loadable>(ring: &Ring, e: u32, field: impl FnOnce(&Entry) -> T) -> X {
    bh::ld(get(ring, e, field).unwrap_or_else(|r| crate::fatal::raise::end("a ring entry", r)))
}

/// A field stored from the x87 stack (as `ld`).
pub fn st<T: bh::Stored>(ring: &Ring, e: u32, x: X, field: impl FnOnce(&mut Entry) -> &mut T) {
    if let Err(r) = set(ring, e, T::stored(x), field) {
        crate::fatal::raise::end("a ring entry", r)
    }
}

/// `f` on entry 1 of a test ring holding the bytes `lo .. hi` of the test buffer `base`
/// (the differential tests' operand), the bytes copied back after it.
#[cfg(test)]
pub fn with_entry_at<R>(
    ring: &Ring,
    base: crate::effects::asm_test::Operand,
    lo: u32,
    hi: u32,
    f: impl FnOnce(u32) -> R,
) -> R {
    use crate::scene::x87_mem::Base;
    let kept = ring.entries.with(std::mem::take).ok();
    let mut w = [0u32; (ENTRY_LEN / 4) as usize];
    for o in (lo..hi).step_by(4) {
        w[o as usize / 4] = base.dword(o as usize);
    }
    ring.entries.set(vec![Entry::from_words(w)]);
    let out = f(1);
    let w = ring.entries.with(|r| r[0].words()).unwrap();
    for o in (lo..hi).step_by(4) {
        base.set_dword(o as usize, w[o as usize / 4]);
    }
    match kept {
        Some(r) => ring.entries.set(r),
        None => ring.entries.give_back(),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring_is_linked_and_read_by_number() {
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let memory = crate::memory::ListCells::empty();
        let ring = &effects().ring;
        take(ring, &memory).unwrap();
        assert_eq!(memory.get().total, RING_BYTES);
        assert_eq!(entry(ring, 1).unwrap().prev, ENTRIES);
        assert_eq!(entry(ring, 1).unwrap().next, 2);
        assert_eq!(entry(ring, ENTRIES).unwrap().next, 1);
        set(ring, 3, 0x1234, |e| &mut e.car_tag).unwrap();
        assert_eq!(get(ring, 3, |e| e.car_tag).unwrap(), 0x1234);
        set(ring, 3, 7.0, |e| &mut e.motion[2]).unwrap();
        assert_eq!(entry(ring, 3).unwrap().motion, [0.0, 0.0, 7.0]);
        assert!(entry(ring, 0).is_err());
        assert!(entry(ring, ENTRIES + 1).is_err());
        assert!(set(ring, 0, 1, |e| &mut e.life).is_err());
        assert!(set(ring, ENTRIES + 1, 1, |e| &mut e.life).is_err());
        ring.entries.give_back();
        crate::memory::with_figures(|f| *f = figures);
    }

    #[test]
    fn fields_sit_at_the_originals_offsets() {
        use std::mem::offset_of;
        for (offset, at) in [
            (0x00, offset_of!(Entry, prev)),
            (0x04, offset_of!(Entry, next)),
            (0x08, offset_of!(Entry, emitter_tag)),
            (0x0C, offset_of!(Entry, kind)),
            (0x10, offset_of!(Entry, place)),
            (0x1C, offset_of!(Entry, last_place)),
            (0x28, offset_of!(Entry, ground_y)),
            (0x2C, offset_of!(Entry, motion)),
            (0x38, offset_of!(Entry, push)),
            (0x44, offset_of!(Entry, spare)),
            (0x50, offset_of!(Entry, angle)),
            (0x54, offset_of!(Entry, angle_rate)),
            (0x58, offset_of!(Entry, cell)),
            (0x5C, offset_of!(Entry, cell_rate)),
            (0x60, offset_of!(Entry, size)),
            (0x64, offset_of!(Entry, size_factor)),
            (0x68, offset_of!(Entry, car)),
            (0x6C, offset_of!(Entry, car_pace)),
            (0x70, offset_of!(Entry, size_goal)),
            (0x74, offset_of!(Entry, alpha)),
            (0x78, offset_of!(Entry, alpha_fade)),
            (0x7C, offset_of!(Entry, box_limit)),
            (0x84, offset_of!(Entry, life)),
            (0x90, offset_of!(Entry, car_tag)),
        ] {
            assert_eq!(at, offset);
        }
        assert_eq!(size_of::<Entry>(), 0x94);
        let words: [u32; 37] = std::array::from_fn(|n| n as u32 + 1);
        let e = Entry::from_words(words);
        assert_eq!(e.words(), words);
        assert_eq!(e.life, 0x84 / 4 + 1);
        assert_eq!(e.motion.map(f32::to_bits), [12, 13, 14]);
        assert_eq!(e.box_limit, [32, 33]);
    }
}
