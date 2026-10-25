//! The hexagons' and squares' variables, out of the marks' block (phase 7, typed state):
//! `HexagonState` (IDA 0x814710 .. 0x814770) and `HexagonWork` (0x814772 .. 0x81518A; on an
//! address 2 more than a multiple of 4 in the image).

use crate::session::{Bytes, Dword, Float};

/// The hexagons' and squares' frame variables (`marks::hexagons_*`, `marks::squares_*`, `marks::draw_plane`) (IDA 0x814710 .. 0x814770, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct HexagonState {
    /// 0x814710: `dword_814710`: the live hexagon items counted in this frame (`marks::hexagons_frame`).
    pub live: Dword,
    /// 0x814714: `dword_814714`: the clock the items' steps are counted from (`marks::hexagons_step`, `marks::squares_step`).
    pub clock: Dword,
    /// 0x814718: `flt_814718`: the place's way dotted with the third axis, negative when it faces the third axis away (`marks::hexagons_top`, `marks::squares_frame`, `marks::draw_plane`).
    pub facing: Float,
    /// 0x81471C: `dword_81471C`, `dword_814720`: the cell (x, z) of the item being looked at (`marks::expire_hexagons`, `marks::expire_squares`).
    pub cell: [Dword; 2],
    /// 0x814724: `dword_814724`: the frames counted (`marks::hexagons_spawn`).
    pub frames: Dword,
    /// 0x814728: `flt_814728`: the hexagons' rate (`marks::hexagons_top`, `marks::hexagons_spawn`).
    pub rate: Float,
    /// 0x81472C: `flt_81472C`: the spawn rate worked out each frame (`marks::hexagons_spawn`).
    pub spawn_rate: Float,
    /// 0x814730: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_814730: Bytes<4>,
    /// 0x814734: `flt_814734`: the sprite's depth (`marks::hexagons_frame`).
    pub depth: Float,
    /// 0x814738: Not named: 56 bytes (a gap, or unreferenced data).
    pub _gap_814738: Bytes<56>,
}

pub static HEXAGON_STATE: HexagonState = HexagonState {
    live: Dword::new(0x0),
    clock: Dword::new(0x0),
    facing: Float::from_bits(0x0),
    cell: [Dword::new(0x0), Dword::new(0x0)],
    frames: Dword::new(0x0),
    rate: Float::from_bits(0x0),
    spawn_rate: Float::from_bits(0x3F900000),
    _gap_814730: Bytes::new([0; 4]),
    depth: Float::from_bits(0x0),
    _gap_814738: Bytes::new([
        0xC8, 0x00, 0x00, 0x00, 0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x3A, 0x5C, 0x72, 0x61, 0x6C, 0x6C, 0x79, 0x35, 0x5C,
        0x73, 0x72, 0x63, 0x5C, 0x6B, 0x61, 0x6B, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D,
        0x5C, 0x62, 0x61, 0x63, 0x6B, 0x62, 0x69, 0x67, 0x2E, 0x70, 0x63,
    ]),
};

impl HexagonState {
    pub const START: usize = 0x814710;
    pub const LEN: usize = 0x60;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        &HEXAGON_STATE as *const HexagonState as usize
    }
}

#[cfg(test)]
mod hexagon_state_tests {
    use super::*;

    #[test]
    fn hexagon_state_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<HexagonState>(), HexagonState::LEN);
        let p = &HEXAGON_STATE as *const HexagonState as *const u8;
        let ours: Vec<u8> = (0..size_of::<HexagonState>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(HexagonState::START, HexagonState::LEN),
            "HEXAGON_STATE starts differently from the image"
        );
        assert_eq!(
            offset_of!(HexagonState, live),
            0x814710 - HexagonState::START
        );
        assert_eq!(
            offset_of!(HexagonState, clock),
            0x814714 - HexagonState::START
        );
        assert_eq!(
            offset_of!(HexagonState, facing),
            0x814718 - HexagonState::START
        );
        assert_eq!(
            offset_of!(HexagonState, cell),
            0x81471C - HexagonState::START
        );
        assert_eq!(
            offset_of!(HexagonState, frames),
            0x814724 - HexagonState::START
        );
        assert_eq!(
            offset_of!(HexagonState, rate),
            0x814728 - HexagonState::START
        );
        assert_eq!(
            offset_of!(HexagonState, spawn_rate),
            0x81472C - HexagonState::START
        );
        assert_eq!(
            offset_of!(HexagonState, depth),
            0x814734 - HexagonState::START
        );
    }
}

/// The hexagons' work (`marks::hexagons_*`, `marks::shapes`, `marks::spawn_hexagon`): the drift, the curve, the radii, the axes, the sprite's corners and the hexagon points (IDA 0x814772 .. 0x81518A, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct HexagonWork {
    /// 0x814772: `flt_814772`, `flt_814776`: the drift at the item's cell (`marks::hexagons_frame`).
    pub drift: [Float; 2],
    /// 0x81477A: Not named: 72 bytes (a gap, or unreferenced data).
    pub _gap_81477a: Bytes<72>,
    /// 0x8147C2: `g_marksHexagonCurve`: a quarter-circle curve of 0x100 floats (`marks::shapes`).
    pub curve: [Float; 256],
    /// 0x814BC2: `flt_814BC2` ..: the circles' radii (eight kinds used).
    pub radii: [Float; 16],
    /// 0x814C02: `flt_814C02`: 1 over `flt_6241C8` (`marks::hexagons_setup`).
    pub inverse: Float,
    /// 0x814C06: `flt_814C06`, `flt_814C0A`: the axes' lengths' shares (`marks::hexagons_top`).
    pub shares: [Float; 2],
    /// 0x814C0E: `flt_814C0E` ..: the view's place (the settings' +0x2720 ..).
    pub place: [Float; 3],
    /// 0x814C1A: `flt_814C1A` ..: the three axes (rows of three).
    pub axes: [Float; 9],
    /// 0x814C3E: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_814c3e: Bytes<48>,
    /// 0x814C6E: `unk_814C6E`, `unk_814C7A`, `unk_814C86`, `unk_814C92`: the two axes scaled (`marks::hexagons_setup`).
    pub setup_axes: [Float; 12],
    /// 0x814C9E: `flt_814C9E`, `flt_814CA2`: the sprite's centre on the screen (`marks::hexagons_frame`).
    pub screen: [Float; 2],
    /// 0x814CA6: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_814ca6: Bytes<4>,
    /// 0x814CAA: `flt_814CAA`, `flt_814CAE`, `flt_814CB2`: the item's place turned, its depth last.
    pub cell: [Float; 3],
    /// 0x814CB6: `flt_814CB6` ..: the six corners (x, y, z).
    pub corners: [Float; 18],
    /// 0x814CFE: `flt_814CFE` ..: the 0x30 points scaled and turned onto the axes (`marks::hexagons_top`).
    pub scaled: [Float; 144],
    /// 0x814F3E: `g_marksHexagonPoints`: 0x30 points of 12 bytes (eight circles of six).
    pub points: [Float; 144],
    /// 0x81517E: `dword_81517E`: an offset into the items (`marks::spawn_hexagon`).
    pub cursor: Dword,
    /// 0x815182: `dword_815182`, `dword_815186`: the item's place as whole numbers (x, z).
    pub whole: [Dword; 2],
}

pub static HEXAGON_WORK: HexagonWork = HexagonWork {
    drift: [Float::from_bits(0x0), Float::from_bits(0x0)],
    _gap_81477a: Bytes::new([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
    ]),
    curve: [const { Float::from_bits(0x0) }; 256],
    radii: [
        Float::from_bits(0x40E00000),
        Float::from_bits(0x40C00000),
        Float::from_bits(0x40A00000),
        Float::from_bits(0x40800000),
        Float::from_bits(0x40400000),
        Float::from_bits(0x40000000),
        Float::from_bits(0x40800000),
        Float::from_bits(0x40A00000),
        Float::from_bits(0x40000000),
        Float::from_bits(0x3FE66666),
        Float::from_bits(0x3FCCCCCD),
        Float::from_bits(0x3FB33333),
        Float::from_bits(0x3F99999A),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x3F99999A),
        Float::from_bits(0x3FB33333),
    ],
    inverse: Float::from_bits(0x0),
    shares: [Float::from_bits(0x42666666), Float::from_bits(0x42666666)],
    place: [
        Float::from_bits(0xC7666666),
        Float::from_bits(0xC6E66666),
        Float::from_bits(0x47800000),
    ],
    axes: [
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    _gap_814c3e: Bytes::new([0; 48]),
    setup_axes: [
        Float::from_bits(0x44D00000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0xC4948000),
        Float::from_bits(0x44948000),
        Float::from_bits(0x44D00000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0xC4948000),
        Float::from_bits(0x44948000),
    ],
    screen: [Float::from_bits(0x0), Float::from_bits(0x0)],
    _gap_814ca6: Bytes::new([0; 4]),
    cell: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    corners: [const { Float::from_bits(0x0) }; 18],
    scaled: [const { Float::from_bits(0x0) }; 144],
    points: [const { Float::from_bits(0x0) }; 144],
    cursor: Dword::new(0x0),
    whole: [Dword::new(0x0), Dword::new(0x0)],
};

impl HexagonWork {
    pub const START: usize = 0x814772;
    pub const LEN: usize = 0xA18;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        &HEXAGON_WORK as *const HexagonWork as usize
    }
}

#[cfg(test)]
mod hexagon_work_tests {
    use super::*;

    #[test]
    fn hexagon_work_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<HexagonWork>(), HexagonWork::LEN);
        let p = &HEXAGON_WORK as *const HexagonWork as *const u8;
        let ours: Vec<u8> = (0..size_of::<HexagonWork>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(HexagonWork::START, HexagonWork::LEN),
            "HEXAGON_WORK starts differently from the image"
        );
        assert_eq!(
            offset_of!(HexagonWork, drift),
            0x814772 - HexagonWork::START
        );
        assert_eq!(
            offset_of!(HexagonWork, curve),
            0x8147C2 - HexagonWork::START
        );
        assert_eq!(
            offset_of!(HexagonWork, radii),
            0x814BC2 - HexagonWork::START
        );
        assert_eq!(
            offset_of!(HexagonWork, inverse),
            0x814C02 - HexagonWork::START
        );
        assert_eq!(
            offset_of!(HexagonWork, shares),
            0x814C06 - HexagonWork::START
        );
        assert_eq!(
            offset_of!(HexagonWork, place),
            0x814C0E - HexagonWork::START
        );
        assert_eq!(offset_of!(HexagonWork, axes), 0x814C1A - HexagonWork::START);
        assert_eq!(
            offset_of!(HexagonWork, setup_axes),
            0x814C6E - HexagonWork::START
        );
        assert_eq!(
            offset_of!(HexagonWork, screen),
            0x814C9E - HexagonWork::START
        );
        assert_eq!(offset_of!(HexagonWork, cell), 0x814CAA - HexagonWork::START);
        assert_eq!(
            offset_of!(HexagonWork, corners),
            0x814CB6 - HexagonWork::START
        );
        assert_eq!(
            offset_of!(HexagonWork, scaled),
            0x814CFE - HexagonWork::START
        );
        assert_eq!(
            offset_of!(HexagonWork, points),
            0x814F3E - HexagonWork::START
        );
        assert_eq!(
            offset_of!(HexagonWork, cursor),
            0x81517E - HexagonWork::START
        );
        assert_eq!(
            offset_of!(HexagonWork, whole),
            0x815182 - HexagonWork::START
        );
    }
}
