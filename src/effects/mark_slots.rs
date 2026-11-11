//! The mark slots' variables: `MarkSlots`
//! (IDA 0x934F7C .. 0x935300: the 0x20 slots and the work places) and `SlotTuning`
//! (0x9353A6 .. 0x935412: constants and randoms; on an address 2 more than a multiple of 4
//! in the image).

#[cfg(test)]
use crate::effects::owner::effects;
use crate::race::owner_frame::Frame;
use crate::session::{PlainBytes, PlainDword, PlainFloat};

/// One mark slot (IDA `g_markSlots`, 0x14 bytes): a sprite that lives a number of ticks.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarkSlot {
    /// +0: the ticks left (0 or less: free; `effects::add_mark_slot` sets 0x3C).
    pub life: PlainDword,
    /// +4: which point of `g_marksPoints` (12 bytes each) the slot sits at.
    pub point: PlainDword,
    /// +8: the matrix it is turned by, as a handle (`VIEW_PLACE_MATRIX`; the original kept
    /// the matrix's address); 0 for none.
    pub matrix: PlainDword,
    /// +0xC: 0.
    pub spare: PlainDword,
    /// +0x10: a float of the slot's own random (`flt_93540E` plus the random's low word).
    pub size: PlainFloat,
}

/// A slot's matrix handle for the view place matrix (`g_viewPlaceMatrix`, `MATRICES.view_place`),
/// the only one `effects::add_mark_slot` is given.
pub const VIEW_PLACE_MATRIX: u32 = 0xCD00_0001;

/// The matrix a slot's handle names; refused for any other value (the original would have
/// turned by whatever lay at that address).
pub fn slot_matrix(
    frame: &Frame,
    handle: u32,
) -> Result<&[crate::session::Float; 16], crate::errors::BadInput> {
    match handle {
        VIEW_PLACE_MATRIX => Ok(&frame.matrices.view_place),
        _ => Err(crate::errors::BadInput(
            "a mark slot's matrix handle that names none",
        )),
    }
}

/// The mark slots (IDA 0x934F7C .. 0x935300).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarkSlots {
    /// 0x934F7C: `g_markSlots`: 0x20 slots of 0x14 bytes.
    pub slots: [MarkSlot; 32],
    /// 0x9351FC: Not named: 248 bytes (a gap, or unreferenced data).
    pub _gap_9351fc: PlainBytes<248>,
    /// 0x9352F4: `dword_9352F4` ..: the slot's point turned, then its projected place (z, y, x; then x, y).
    pub work: [PlainDword; 3],
}

impl MarkSlots {
    pub const fn new() -> Self {
        Self {
            slots: [const {
                MarkSlot {
                    life: PlainDword::new(0),
                    point: PlainDword::new(0),
                    matrix: PlainDword::new(0),
                    spare: PlainDword::new(0),
                    size: PlainFloat::new(16384.0),
                }
            }; 32],
            _gap_9351fc: PlainBytes::new([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0xF5, 0x47, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xBF, 0x00, 0x00,
                0x80, 0xBF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x80, 0xBF, 0x00, 0x00, 0x80, 0xBF, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x47, 0x00, 0x00, 0x80, 0x47, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]),
            work: [PlainDword::new(0), PlainDword::new(0), PlainDword::new(0)],
        }
    }
}

impl MarkSlots {
    pub const START: usize = 0x934F7C;
    pub const LEN: usize = 0x384;
}

/// The mark slots' constants and randoms (IDA 0x9353A6 .. 0x935412).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SlotTuning {
    /// 0x9353A6: `flt_9353A6`: half the slot square's size on the screen.
    pub half: PlainFloat,
    /// 0x9353AA: Not named: 68 bytes (a gap, or unreferenced data).
    pub _gap_9353aa: PlainBytes<68>,
    /// 0x9353EE: `flt_9353EE`, `flt_9353F2`, `flt_9353F6`, `flt_9353FA`: what a slot's point is scaled by: z times the first, y less the second times the third, x times the fourth.
    pub place_k: [PlainFloat; 4],
    /// 0x9353FE: `dword_9353FE`: `effects::draw_mark_slots`' random (times 0xC5 plus 0x5D) choosing a corner picture.
    pub draw_random: PlainDword,
    /// 0x935402: `flt_935402`: what one over the depth is multiplied by (the vertices' rhw).
    pub rhw_k: PlainFloat,
    /// 0x935406: `flt_935406`: `effects::add_mark_slot` first takes it, then the random (nothing keeps it).
    pub first_float: PlainFloat,
    /// 0x93540A: `g_marksSlotRandom`: `effects::add_mark_slot`' random (times 0xC5 plus 0x5D).
    pub slot_random: PlainDword,
    /// 0x93540E: `flt_93540E`: added to a new slot's random (the slot's size).
    pub base: PlainFloat,
}

impl SlotTuning {
    pub const fn new() -> Self {
        Self {
            half: PlainFloat::new(0.0),
            _gap_9353aa: PlainBytes::new([0; 68]),
            place_k: [
                PlainFloat::new(65.536),
                PlainFloat::new(700.0),
                PlainFloat::new(65.536),
                PlainFloat::new(65.536),
            ],
            draw_random: PlainDword::new(0x0),
            rhw_k: PlainFloat::new(256.0),
            first_float: PlainFloat::new(32768.0),
            slot_random: PlainDword::new(0x0),
            base: PlainFloat::new(16384.0),
        }
    }
}

impl SlotTuning {
    pub const START: usize = 0x9353A6;
    pub const LEN: usize = 0x6C;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn the_slots_start_as_the_games() {
        let blocks = &effects().blocks;
        assert_eq!(size_of::<MarkSlot>(), 0x14);
        assert_eq!(size_of::<MarkSlots>(), MarkSlots::LEN);
        assert_eq!(size_of::<SlotTuning>(), SlotTuning::LEN);
        let image = crate::moved::image_bytes;
        assert!(
            bytes(&blocks.slots) == image(MarkSlots::START, MarkSlots::LEN),
            "SLOTS starts differently from the image"
        );
        assert!(
            bytes(&blocks.slot_tuning) == image(SlotTuning::START, SlotTuning::LEN),
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
