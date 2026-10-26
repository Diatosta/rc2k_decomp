//! The masks' variables out of the marks' block (phase 7).

use crate::session::{Bytes, Dword, Float, Word};

/// One of the masks' shapes (IDA `g_marksShapesA` / `B`, 0x24 bytes).
#[repr(C)]
pub struct Shape {
    /// 0x0: the shape picked (`pick_shapes` sets it) and the eased step count `turn_all_shapes` reads (a reading)
    pub step: Dword,
    /// 0x4: the shape's count of ring quads (0x80)
    pub count: Dword,
    /// 0x8: the address of its runs of rows (`span_ring` puts the runs' place here)
    pub rows: Dword,
    /// 0xC: its middle (x, y)
    pub middle: [Float; 2],
    /// 0x14: the ring's two angles' ends (times 0.0175)
    pub angle_ends: [Float; 2],
    /// 0x1C: the ring's two radii
    pub radii: [Float; 2],
}

/// The masks' variables (IDA 0x82E342 .. 0x844602, the image's layout; phase 7, typed state): the work block's list, the corner tables, the shapes, the rasteriser's scratch, the runs, the masks and the ring angles.
#[repr(C)]
pub struct MaskVars {
    /// 0x82E342: `g_marksSpansTotal`: the spans made
    pub spans_total: Dword,
    /// 0x82E346: `g_marksMaskList`: the work block's memory list
    pub list: Bytes<64>,
    /// 0x82E386: `g_marksMaskWork`: the 0x80000-byte work block
    pub work: Dword,
    /// 0x82E38A: `dword_82E38A`: the running count `run_shapes` moves the shape index by
    pub count: Dword,
    /// 0x82E38E: `g_marksCornerScale`, `flt_82E392`: the corner tables' scales
    pub corner_scale: [Float; 2],
    /// 0x82E396: `g_marksLineTurn`: a 3 x 3 turn (the 2 x 2 turn is its elements 0, 1, 3, 4)
    pub turn: [Float; 9],
    /// 0x82E3BA: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_82e3ba: Bytes<12>,
    /// 0x82E3C6: `flt_82E3C6`, `flt_82E3CA`: the line's span and rise
    pub slope: [Float; 2],
    /// 0x82E3CE: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_82e3ce: Bytes<4>,
    /// 0x82E3D2: `dword_82E3D2`, `dword_82E3D6`: the pixel worked on
    pub work_pixel: [Dword; 2],
    /// 0x82E3DA: `g_marksLines`: two lines (x, y; x, y; x, y; 0x18 bytes each)
    pub lines: [Dword; 12],
    /// 0x82E40A: `flt_82E40A` .. `flt_82E41A`: the line's ends turned (the third is left)
    pub ends: [Float; 5],
    /// 0x82E41E: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_82e41e: Bytes<4>,
    /// 0x82E422: `g_marksRingCorners`: the corner table in use
    pub ring_corners: Dword,
    /// 0x82E426: `g_marksCornersA`: 0x81 entries of two corners
    pub corners_a: Bytes<3096>,
    /// 0x82F03E: `g_marksCornersB`
    pub corners_b: Bytes<3096>,
    /// 0x82FC56: `g_marksShapesA`, `g_marksShapesB`
    pub shapes: [Shape; 2],
    /// 0x82FC9E: Not named: 72 bytes (a gap, or unreferenced data).
    pub _gap_82fc9e: Bytes<72>,
    /// 0x82FCE6: `g_marksControlWord`: the x87 control word the triangle came with
    pub control_word: Word,
    /// 0x82FCE8: `word_82FCE8`: the compares' status word
    pub status_word: Word,
    /// 0x82FCEA: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_82fcea: Bytes<4>,
    /// 0x82FCEE: `flt_82FCEE`
    pub height: Float,
    /// 0x82FCF2: `dword_82FCF2`
    pub row_int: Dword,
    /// 0x82FCF6: `flt_82FCF6`
    pub row: Float,
    /// 0x82FCFA: `flt_82FCFA`, `flt_82FCFE`
    pub slopes: [Float; 2],
    /// 0x82FD02: `flt_82FD02`, `flt_82FD06`
    pub xs: [Float; 2],
    /// 0x82FD0A: `flt_82FD0A`
    pub split: Float,
    /// 0x82FD0E: `g_marksMaskCursor`
    pub cursor: Dword,
    /// 0x82FD12: `g_marksMaskRuns`: where the next run goes
    pub runs: Dword,
    /// 0x82FD16: `g_marksEdge`: y0, y1, the left and right x at y0, at y1
    pub edge: [Dword; 6],
    /// 0x82FD2E: Not named: 8192 bytes (a gap, or unreferenced data).
    pub _gap_82fd2e: Bytes<8192>,
    /// 0x831D2E: `unk_831D2E`: the runs (16 bytes each)
    pub run_table: Bytes<8192>,
    /// 0x833D2E: `g_marksTriangle`: the sorted corners
    pub triangle: [Float; 6],
    /// 0x833D46: `g_marksQuadTriangle`: a quad's triangle
    pub quad_triangle: [Float; 6],
    /// 0x833D5E: `g_marksRingQuad`
    pub ring_quad: [Float; 8],
    /// 0x833D7E: `g_marksMaskRunCount`
    pub run_count: Dword,
    /// 0x833D82: `flt_833D82`, `flt_833D86`
    pub ring_ends: [Float; 2],
    /// 0x833D8A: `flt_833D8A`
    pub ring_middle: Float,
    /// 0x833D8E: `flt_833D8E`
    pub ring_half: Float,
    /// 0x833D92: `flt_833D92`, `flt_833D96`
    pub ring_angles: [Float; 2],
    /// 0x833D9A: `flt_833D9A` .. `flt_833DA6`
    pub ring_turns: [Float; 4],
    /// 0x833DAA: `dword_833DAA`
    pub ring_angle: Dword,
    /// 0x833DAE: `flt_833DAE`
    pub ring_step: Float,
    /// 0x833DB2: `g_marksPixel`, `dword_833DB6`: x, then y
    pub pixel: [Dword; 2],
    /// 0x833DBA: `g_marksMaskValue`: the shape's number
    pub mask_value: Dword,
    /// 0x833DBE: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_833dbe: Bytes<4>,
    /// 0x833DC2: `g_marksMaskA`: rows of 0x100
    pub mask_a: Bytes<32768>,
    /// 0x83BDC2: `g_marksMaskB`
    pub mask_b: Bytes<32768>,
    /// 0x843DC2: `g_marksRingAnglesA`
    pub ring_angles_a: Bytes<1032>,
    /// 0x8441CA: `g_marksRingAnglesB`
    pub ring_angles_b: Bytes<1032>,
    /// 0x8445D2: `g_marksPixelTable`: the table in use (rows of 0x400)
    pub pixel_table: Dword,
    /// 0x8445D6: `flt_8445D6`, `flt_8445DA`: the shapes' middle
    pub middle: [Float; 2],
    /// 0x8445DE: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8445de: Bytes<4>,
    /// 0x8445E2: `flt_8445E2`
    pub row_distance: Float,
    /// 0x8445E6: `g_marksPixelCorners`: the corner table kept
    pub pixel_corners: Dword,
    /// 0x8445EA: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8445ea: Bytes<24>,
}

pub static MASKS: MaskVars = MaskVars {
    spans_total: Dword::new(0x0),
    list: Bytes::new([0; 64]),
    work: Dword::new(0x0),
    count: Dword::new(0x0),
    corner_scale: [const { Float::from_bits(0x0) }; 2],
    turn: [const { Float::from_bits(0x0) }; 9],
    _gap_82e3ba: Bytes::new([0; 12]),
    slope: [const { Float::from_bits(0x0) }; 2],
    _gap_82e3ce: Bytes::new([0; 4]),
    work_pixel: [const { Dword::new(0x0) }; 2],
    lines: [const { Dword::new(0x0) }; 12],
    ends: [const { Float::from_bits(0x0) }; 5],
    _gap_82e41e: Bytes::new([0; 4]),
    ring_corners: Dword::new(0x0),
    corners_a: Bytes::new([0; 3096]),
    corners_b: Bytes::new([0; 3096]),
    shapes: [
        Shape {
            step: Dword::new(0x0),
            count: Dword::new(0x80),
            rows: Dword::new(0x0),
            middle: [Float::from_bits(0x41900000), Float::from_bits(0x43000000)],
            angle_ends: [Float::from_bits(0xC0A00000), Float::from_bits(0x41700000)],
            radii: [Float::from_bits(0x42DC0000), Float::from_bits(0x41F00000)],
        },
        Shape {
            step: Dword::new(0x0),
            count: Dword::new(0x80),
            rows: Dword::new(0x0),
            middle: [Float::from_bits(0x42C40000), Float::from_bits(0x43000000)],
            angle_ends: [Float::from_bits(0xC20C0000), Float::from_bits(0x428C0000)],
            radii: [Float::from_bits(0x42DC0000), Float::from_bits(0x41F00000)],
        },
    ],
    _gap_82fc9e: Bytes::new([
        0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0,
        0x42, 0x00, 0x00, 0x80, 0x42, 0x00, 0x00, 0x20, 0xC3, 0x00, 0x00, 0x20, 0x43, 0x00, 0x00,
        0x70, 0x42, 0x00, 0x00, 0xA0, 0x41, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x43, 0x00, 0x00, 0x0C, 0x43, 0x00, 0x00, 0xA0, 0xC1,
        0x00, 0x00, 0x0C, 0x42, 0x00, 0x00, 0xDC, 0x42, 0x00, 0x00, 0x8C, 0x42,
    ]),
    control_word: Word::new(0x0),
    status_word: Word::new(0x0),
    _gap_82fcea: Bytes::new([0; 4]),
    height: Float::from_bits(0x0),
    row_int: Dword::new(0x0),
    row: Float::from_bits(0x0),
    slopes: [const { Float::from_bits(0x0) }; 2],
    xs: [const { Float::from_bits(0x0) }; 2],
    split: Float::from_bits(0x0),
    cursor: Dword::new(0x0),
    runs: Dword::new(0x0),
    edge: [const { Dword::new(0x0) }; 6],
    _gap_82fd2e: Bytes::new([0; 8192]),
    run_table: Bytes::new([0; 8192]),
    triangle: [const { Float::from_bits(0x0) }; 6],
    quad_triangle: [const { Float::from_bits(0x0) }; 6],
    ring_quad: [const { Float::from_bits(0x0) }; 8],
    run_count: Dword::new(0x0),
    ring_ends: [const { Float::from_bits(0x0) }; 2],
    ring_middle: Float::from_bits(0x0),
    ring_half: Float::from_bits(0x0),
    ring_angles: [const { Float::from_bits(0x0) }; 2],
    ring_turns: [const { Float::from_bits(0x0) }; 4],
    ring_angle: Dword::new(0x0),
    ring_step: Float::from_bits(0x0),
    pixel: [const { Dword::new(0x0) }; 2],
    mask_value: Dword::new(0x0),
    _gap_833dbe: Bytes::new([0; 4]),
    mask_a: Bytes::new([0; 32768]),
    mask_b: Bytes::new([0; 32768]),
    ring_angles_a: Bytes::new([0; 1032]),
    ring_angles_b: Bytes::new([0; 1032]),
    pixel_table: Dword::new(0x0),
    middle: [const { Float::from_bits(0x0) }; 2],
    _gap_8445de: Bytes::new([0; 4]),
    row_distance: Float::from_bits(0x0),
    pixel_corners: Dword::new(0x0),
    _gap_8445ea: Bytes::new([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x6F, 0x6C, 0x31, 0x2E, 0x64, 0x61,
        0x74, 0x00, 0x63, 0x6F, 0x6C, 0x32, 0x2E, 0x64, 0x61,
    ]),
};

impl MaskVars {
    pub const START: usize = 0x82E342;
    pub const LEN: usize = 0x162C0;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(core::ptr::addr_of!(MASKS))) as usize
    }
}

#[cfg(test)]
mod tests_masks {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn masks_starts_as_the_games() {
        assert_eq!(size_of::<MaskVars>(), MaskVars::LEN);
        assert_eq!(
            offset_of!(MaskVars, spans_total),
            0x82E342 - MaskVars::START
        );
        assert_eq!(offset_of!(MaskVars, list), 0x82E346 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, work), 0x82E386 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, count), 0x82E38A - MaskVars::START);
        assert_eq!(
            offset_of!(MaskVars, corner_scale),
            0x82E38E - MaskVars::START
        );
        assert_eq!(offset_of!(MaskVars, turn), 0x82E396 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, slope), 0x82E3C6 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, work_pixel), 0x82E3D2 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, lines), 0x82E3DA - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, ends), 0x82E40A - MaskVars::START);
        assert_eq!(
            offset_of!(MaskVars, ring_corners),
            0x82E422 - MaskVars::START
        );
        assert_eq!(offset_of!(MaskVars, corners_a), 0x82E426 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, corners_b), 0x82F03E - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, shapes), 0x82FC56 - MaskVars::START);
        assert_eq!(
            offset_of!(MaskVars, control_word),
            0x82FCE6 - MaskVars::START
        );
        assert_eq!(
            offset_of!(MaskVars, status_word),
            0x82FCE8 - MaskVars::START
        );
        assert_eq!(offset_of!(MaskVars, height), 0x82FCEE - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, row_int), 0x82FCF2 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, row), 0x82FCF6 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, slopes), 0x82FCFA - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, xs), 0x82FD02 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, split), 0x82FD0A - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, cursor), 0x82FD0E - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, runs), 0x82FD12 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, edge), 0x82FD16 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, run_table), 0x831D2E - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, triangle), 0x833D2E - MaskVars::START);
        assert_eq!(
            offset_of!(MaskVars, quad_triangle),
            0x833D46 - MaskVars::START
        );
        assert_eq!(offset_of!(MaskVars, ring_quad), 0x833D5E - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, run_count), 0x833D7E - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, ring_ends), 0x833D82 - MaskVars::START);
        assert_eq!(
            offset_of!(MaskVars, ring_middle),
            0x833D8A - MaskVars::START
        );
        assert_eq!(offset_of!(MaskVars, ring_half), 0x833D8E - MaskVars::START);
        assert_eq!(
            offset_of!(MaskVars, ring_angles),
            0x833D92 - MaskVars::START
        );
        assert_eq!(offset_of!(MaskVars, ring_turns), 0x833D9A - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, ring_angle), 0x833DAA - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, ring_step), 0x833DAE - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, pixel), 0x833DB2 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, mask_value), 0x833DBA - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, mask_a), 0x833DC2 - MaskVars::START);
        assert_eq!(offset_of!(MaskVars, mask_b), 0x83BDC2 - MaskVars::START);
        assert_eq!(
            offset_of!(MaskVars, ring_angles_a),
            0x843DC2 - MaskVars::START
        );
        assert_eq!(
            offset_of!(MaskVars, ring_angles_b),
            0x8441CA - MaskVars::START
        );
        assert_eq!(
            offset_of!(MaskVars, pixel_table),
            0x8445D2 - MaskVars::START
        );
        assert_eq!(offset_of!(MaskVars, middle), 0x8445D6 - MaskVars::START);
        assert_eq!(
            offset_of!(MaskVars, row_distance),
            0x8445E2 - MaskVars::START
        );
        assert_eq!(
            offset_of!(MaskVars, pixel_corners),
            0x8445E6 - MaskVars::START
        );
        let p = core::ptr::addr_of!(MASKS) as *const u8;
        let now: Vec<u8> = (0..MaskVars::LEN)
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            now == crate::moved::image_bytes(MaskVars::START, MaskVars::LEN),
            "MASKS starts differently from the image"
        );
    }
}
