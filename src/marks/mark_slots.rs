//! The mark slots' variables, out of the marks' block (phase 7, typed state): `MarkSlots`
//! (IDA 0x934F7C .. 0x935300: the 0x20 slots and the work places) and `SlotTuning`
//! (0x9353A6 .. 0x935412: constants and randoms; on an address 2 more than a multiple of 4
//! in the image).

use crate::session::{Bytes, Dword, Float};

/// One mark slot (IDA `g_markSlots`, 0x14 bytes): a sprite that lives a number of ticks.
#[repr(C)]
pub struct MarkSlot {
    /// +0: the ticks left (0 or less: free; `marks::add_mark_slot` sets 0x3C).
    pub life: Dword,
    /// +4: which point of `g_marksPoints` (12 bytes each) the slot sits at.
    pub point: Dword,
    /// +8: the address of the matrix it is turned by.
    pub matrix: Dword,
    /// +0xC: 0.
    pub spare: Dword,
    /// +0x10: a float of the slot's own random (`flt_93540E` plus the random's low word).
    pub size: Float,
}

/// The mark slots (IDA 0x934F7C .. 0x935300, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct MarkSlots {
    /// 0x934F7C: `g_markSlots`: 0x20 slots of 0x14 bytes.
    pub slots: [MarkSlot; 32],
    /// 0x9351FC: Not named: 248 bytes (a gap, or unreferenced data).
    pub _gap_9351fc: Bytes<248>,
    /// 0x9352F4: `dword_9352F4` ..: the slot's point turned, then its projected place (z, y, x; then x, y).
    pub work: [Dword; 3],
}

pub static SLOTS: MarkSlots = MarkSlots {
    slots: [const {
        MarkSlot {
            life: Dword::new(0),
            point: Dword::new(0),
            matrix: Dword::new(0),
            spare: Dword::new(0),
            size: Float::from_bits(0x46800000),
        }
    }; 32],
    _gap_9351fc: Bytes::new([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0xF5, 0x47, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x80, 0xBF, 0x00, 0x00, 0x80, 0xBF, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xBF, 0x00, 0x00, 0x80,
        0xBF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x47, 0x00, 0x00, 0x80, 0x47, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]),
    work: [Dword::new(0), Dword::new(0), Dword::new(0)],
};

impl MarkSlots {
    pub const START: usize = 0x934F7C;
    pub const LEN: usize = 0x384;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::host_number(&SLOTS as *const MarkSlots)
    }
}

/// The mark slots' constants and randoms (IDA 0x9353A6 .. 0x935412, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SlotTuning {
    /// 0x9353A6: `flt_9353A6`: half the slot square's size on the screen.
    pub half: Float,
    /// 0x9353AA: Not named: 68 bytes (a gap, or unreferenced data).
    pub _gap_9353aa: Bytes<68>,
    /// 0x9353EE: `flt_9353EE`, `flt_9353F2`, `flt_9353F6`, `flt_9353FA`: what a slot's point is scaled by: z times the first, y less the second times the third, x times the fourth.
    pub place_k: [Float; 4],
    /// 0x9353FE: `dword_9353FE`: `marks::draw_mark_slots`' random (times 0xC5 plus 0x5D) choosing a corner picture.
    pub draw_random: Dword,
    /// 0x935402: `flt_935402`: what one over the depth is multiplied by (the vertices' rhw).
    pub rhw_k: Float,
    /// 0x935406: `flt_935406`: `marks::add_mark_slot` first takes it, then the random (nothing keeps it).
    pub first_float: Float,
    /// 0x93540A: `g_marksSlotRandom`: `marks::add_mark_slot`' random (times 0xC5 plus 0x5D).
    pub slot_random: Dword,
    /// 0x93540E: `flt_93540E`: added to a new slot's random (the slot's size).
    pub base: Float,
}

pub static TUNING: SlotTuning = SlotTuning {
    half: Float::from_bits(0x0),
    _gap_9353aa: Bytes::new([0; 68]),
    place_k: [
        Float::from_bits(0x4283126F),
        Float::from_bits(0x442F0000),
        Float::from_bits(0x4283126F),
        Float::from_bits(0x4283126F),
    ],
    draw_random: Dword::new(0x0),
    rhw_k: Float::from_bits(0x43800000),
    first_float: Float::from_bits(0x47000000),
    slot_random: Dword::new(0x0),
    base: Float::from_bits(0x46800000),
};

impl SlotTuning {
    pub const START: usize = 0x9353A6;
    pub const LEN: usize = 0x6C;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::host_number(&TUNING as *const SlotTuning)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = v as *const T as *const u8;
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn the_slots_start_as_the_games() {
        assert_eq!(size_of::<MarkSlot>(), 0x14);
        assert_eq!(size_of::<MarkSlots>(), MarkSlots::LEN);
        assert_eq!(size_of::<SlotTuning>(), SlotTuning::LEN);
        let image = crate::moved::image_bytes;
        assert!(
            bytes(&SLOTS) == image(MarkSlots::START, MarkSlots::LEN),
            "SLOTS starts differently from the image"
        );
        assert!(
            bytes(&TUNING) == image(SlotTuning::START, SlotTuning::LEN),
            "TUNING starts differently from the image"
        );
        use core::mem::offset_of;
        assert_eq!(offset_of!(MarkSlots, work), 0x9352F4 - MarkSlots::START);
        assert_eq!(
            offset_of!(SlotTuning, place_k),
            0x9353EE - SlotTuning::START
        );
        assert_eq!(
            offset_of!(SlotTuning, slot_random),
            0x93540A - SlotTuning::START
        );
    }
}
