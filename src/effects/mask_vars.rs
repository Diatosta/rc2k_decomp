//! The masks' variables.

use crate::session::{PlainBytes, PlainDword, PlainFloat, PlainWord};

/// One of the masks' shapes (IDA `g_marksShapesA` / `B`, 0x24 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Shape {
    /// 0x0: the step number of the shape's eased turn: `pick_shapes` sets it to the absolute value of the picks table's entry; `turn_all_shapes` passes it (and one more) to the cosine easing between the set's turn angle and the work step; `expire_windscreen_flakes` reads its low byte as the square-ends test input. Probable: indirect, from these uses.
    pub ease_step: PlainDword,
    /// 0x4: the shape's count of ring quads (0x80)
    pub count: PlainDword,
    /// 0x8: the offset in `run_table` of its runs of rows (`span_ring` puts the runs' place here; the original kept the address)
    pub rows: PlainDword,
    /// 0xC: its middle (x, y)
    pub middle: [PlainFloat; 2],
    /// 0x14: the ring's two angles' ends (times 0.0175)
    pub angle_ends: [PlainFloat; 2],
    /// 0x1C: the ring's two radii
    pub radii: [PlainFloat; 2],
}

/// The masks' variables (IDA 0x82E342 .. 0x844602): the work block's list, the corner tables, the shapes, the rasteriser's scratch, the runs, the masks and the ring angles.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MaskVars {
    /// 0x82E342: `g_marksSpansTotal`: the spans made
    pub spans_total: PlainDword,
    /// 0x82E346: `g_marksMaskList`: the work block's memory list
    pub list: PlainBytes<64>,
    /// 0x82E386: `g_marksMaskWork`: the 0x80000-byte work block (the game's address; since
    /// phase 12 the block is Rust's, `effects::masks::MASK_WORK`, and this stays 0)
    pub work: PlainDword,
    /// 0x82E38A: `dword_82E38A`: the running count `run_shapes` moves the shape index by
    pub count: PlainDword,
    /// 0x82E38E: `g_marksCornerScale`, `flt_82E392`: the corner tables' scales
    pub corner_scale: [PlainFloat; 2],
    /// 0x82E396: `g_marksLineTurn`: a 3 x 3 turn (the 2 x 2 turn is its elements 0, 1, 3, 4)
    pub turn: [PlainFloat; 9],
    /// 0x82E3BA: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_82e3ba: PlainBytes<12>,
    /// 0x82E3C6: `flt_82E3C6`, `flt_82E3CA`: the line's span and rise
    pub slope: [PlainFloat; 2],
    /// 0x82E3CE: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_82e3ce: PlainBytes<4>,
    /// 0x82E3D2: `dword_82E3D2`, `dword_82E3D6`: the pixel worked on
    pub work_pixel: [PlainDword; 2],
    /// 0x82E3DA: `g_marksLines`: two lines (x, y; x, y; x, y; 0x18 bytes each)
    pub lines: [PlainDword; 12],
    /// 0x82E40A: `flt_82E40A` .. `flt_82E41A`: the line's ends turned (the third is left)
    pub ends: [PlainFloat; 5],
    /// 0x82E41E: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_82e41e: PlainBytes<4>,
    /// 0x82E422: `g_marksRingCorners`: the corner table in use (its number, 0 for A and 1 for B; the original kept the address)
    pub ring_corners: PlainDword,
    /// 0x82E426: `g_marksCornersA`: 0x81 entries of two corners
    pub corners_a: PlainBytes<3096>,
    /// 0x82F03E: `g_marksCornersB`
    pub corners_b: PlainBytes<3096>,
    /// 0x82FC56: `g_marksShapesA`, `g_marksShapesB`
    pub shapes: [Shape; 2],
    /// 0x82FC9E: Not named: 72 bytes (a gap, or unreferenced data).
    pub _gap_82fc9e: PlainBytes<72>,
    /// 0x82FCE6: `g_marksControlWord`: the x87 control word the triangle came with
    pub control_word: PlainWord,
    /// 0x82FCE8: `word_82FCE8`: the compares' status word
    pub status_word: PlainWord,
    /// 0x82FCEA: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_82fcea: PlainBytes<4>,
    /// 0x82FCEE: `flt_82FCEE`
    pub height: PlainFloat,
    /// 0x82FCF2: `dword_82FCF2`
    pub row_int: PlainDword,
    /// 0x82FCF6: `flt_82FCF6`
    pub row: PlainFloat,
    /// 0x82FCFA: `flt_82FCFA`, `flt_82FCFE`
    pub slopes: [PlainFloat; 2],
    /// 0x82FD02: `flt_82FD02`, `flt_82FD06`
    pub xs: [PlainFloat; 2],
    /// 0x82FD0A: `flt_82FD0A`
    pub split: PlainFloat,
    /// 0x82FD0E: `g_marksMaskCursor` (since phase 12 a byte offset in `MASK_WORK`, as are
    /// the runs' span places)
    pub cursor: PlainDword,
    /// 0x82FD12: `g_marksMaskRuns`: where the next run goes (a byte offset in `run_table`; the original kept the address)
    pub runs: PlainDword,
    /// 0x82FD16: `g_marksEdge`: y0, y1, the left and right x at y0, at y1
    pub edge: [PlainDword; 6],
    /// 0x82FD2E: Not named: 8192 bytes (a gap, or unreferenced data).
    pub _gap_82fd2e: PlainBytes<8192>,
    /// 0x831D2E: `unk_831D2E`: the runs (16 bytes each)
    pub run_table: PlainBytes<8192>,
    /// 0x833D2E: `g_marksTriangle`: the sorted corners
    pub triangle: [PlainFloat; 6],
    /// 0x833D46: `g_marksQuadTriangle`: a quad's triangle
    pub quad_triangle: [PlainFloat; 6],
    /// 0x833D5E: `g_marksRingQuad`
    pub ring_quad: [PlainFloat; 8],
    /// 0x833D7E: `g_marksMaskRunCount`
    pub run_count: PlainDword,
    /// 0x833D82: `flt_833D82`, `flt_833D86`
    pub ring_ends: [PlainFloat; 2],
    /// 0x833D8A: `flt_833D8A`
    pub ring_middle: PlainFloat,
    /// 0x833D8E: `flt_833D8E`
    pub ring_half: PlainFloat,
    /// 0x833D92: `flt_833D92`, `flt_833D96`
    pub ring_angles: [PlainFloat; 2],
    /// 0x833D9A: `flt_833D9A` .. `flt_833DA6`
    pub ring_turns: [PlainFloat; 4],
    /// 0x833DAA: `dword_833DAA`
    pub ring_angle: PlainDword,
    /// 0x833DAE: `flt_833DAE`
    pub ring_step: PlainFloat,
    /// 0x833DB2: `g_marksPixel`, `dword_833DB6`: x, then y
    pub pixel: [PlainDword; 2],
    /// 0x833DBA: `g_marksMaskValue`: the shape's number
    pub mask_value: PlainDword,
    /// 0x833DBE: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_833dbe: PlainBytes<4>,
    /// 0x833DC2: `g_marksMaskA`: rows of 0x100
    pub mask_a: PlainBytes<32768>,
    /// 0x83BDC2: `g_marksMaskB`
    pub mask_b: PlainBytes<32768>,
    /// 0x843DC2: `g_marksRingAnglesA`
    pub ring_angles_a: PlainBytes<1032>,
    /// 0x8441CA: `g_marksRingAnglesB`
    pub ring_angles_b: PlainBytes<1032>,
    /// 0x8445D2: `g_marksPixelTable`: the table in use (rows of 0x400; its shape's number, `PICKS.pixel_table_a` or `_b`, where the original kept its address)
    pub pixel_table: PlainDword,
    /// 0x8445D6: `flt_8445D6`, `flt_8445DA`: the shapes' middle
    pub middle: [PlainFloat; 2],
    /// 0x8445DE: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8445de: PlainBytes<4>,
    /// 0x8445E2: `flt_8445E2`
    pub row_distance: PlainFloat,
    /// 0x8445E6: `g_marksPixelCorners`: the corner table kept (its shape's number; the original kept its address)
    pub pixel_corners: PlainDword,
    /// 0x8445EA: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8445ea: PlainBytes<24>,
}

impl MaskVars {
    pub const fn new() -> Self {
        MaskVars {
            spans_total: PlainDword::new(0x0),
            list: PlainBytes::new([0; 64]),
            work: PlainDword::new(0x0),
            count: PlainDword::new(0x0),
            corner_scale: [const { PlainFloat::new(0.0) }; 2],
            turn: [const { PlainFloat::new(0.0) }; 9],
            _gap_82e3ba: PlainBytes::new([0; 12]),
            slope: [const { PlainFloat::new(0.0) }; 2],
            _gap_82e3ce: PlainBytes::new([0; 4]),
            work_pixel: [const { PlainDword::new(0x0) }; 2],
            lines: [const { PlainDword::new(0x0) }; 12],
            ends: [const { PlainFloat::new(0.0) }; 5],
            _gap_82e41e: PlainBytes::new([0; 4]),
            ring_corners: PlainDword::new(0x0),
            corners_a: PlainBytes::new([0; 3096]),
            corners_b: PlainBytes::new([0; 3096]),
            shapes: [
                Shape {
                    ease_step: PlainDword::new(0x0),
                    count: PlainDword::new(0x80),
                    rows: PlainDword::new(0x0),
                    middle: [PlainFloat::new(18.0), PlainFloat::new(128.0)],
                    angle_ends: [PlainFloat::new(-5.0), PlainFloat::new(15.0)],
                    radii: [PlainFloat::new(110.0), PlainFloat::new(30.0)],
                },
                Shape {
                    ease_step: PlainDword::new(0x0),
                    count: PlainDword::new(0x80),
                    rows: PlainDword::new(0x0),
                    middle: [PlainFloat::new(98.0), PlainFloat::new(128.0)],
                    angle_ends: [PlainFloat::new(-35.0), PlainFloat::new(70.0)],
                    radii: [PlainFloat::new(110.0), PlainFloat::new(30.0)],
                },
            ],
            _gap_82fc9e: PlainBytes::new([
                0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0xC0, 0x42, 0x00, 0x00, 0x80, 0x42, 0x00, 0x00, 0x20, 0xC3, 0x00, 0x00, 0x20, 0x43,
                0x00, 0x00, 0x70, 0x42, 0x00, 0x00, 0xA0, 0x41, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x43, 0x00, 0x00, 0x0C, 0x43,
                0x00, 0x00, 0xA0, 0xC1, 0x00, 0x00, 0x0C, 0x42, 0x00, 0x00, 0xDC, 0x42, 0x00, 0x00,
                0x8C, 0x42,
            ]),
            control_word: PlainWord::new(0x0),
            status_word: PlainWord::new(0x0),
            _gap_82fcea: PlainBytes::new([0; 4]),
            height: PlainFloat::new(0.0),
            row_int: PlainDword::new(0x0),
            row: PlainFloat::new(0.0),
            slopes: [const { PlainFloat::new(0.0) }; 2],
            xs: [const { PlainFloat::new(0.0) }; 2],
            split: PlainFloat::new(0.0),
            cursor: PlainDword::new(0x0),
            runs: PlainDword::new(0x0),
            edge: [const { PlainDword::new(0x0) }; 6],
            _gap_82fd2e: PlainBytes::new([0; 8192]),
            run_table: PlainBytes::new([0; 8192]),
            triangle: [const { PlainFloat::new(0.0) }; 6],
            quad_triangle: [const { PlainFloat::new(0.0) }; 6],
            ring_quad: [const { PlainFloat::new(0.0) }; 8],
            run_count: PlainDword::new(0x0),
            ring_ends: [const { PlainFloat::new(0.0) }; 2],
            ring_middle: PlainFloat::new(0.0),
            ring_half: PlainFloat::new(0.0),
            ring_angles: [const { PlainFloat::new(0.0) }; 2],
            ring_turns: [const { PlainFloat::new(0.0) }; 4],
            ring_angle: PlainDword::new(0x0),
            ring_step: PlainFloat::new(0.0),
            pixel: [const { PlainDword::new(0x0) }; 2],
            mask_value: PlainDword::new(0x0),
            _gap_833dbe: PlainBytes::new([0; 4]),
            mask_a: PlainBytes::new([0; 32768]),
            mask_b: PlainBytes::new([0; 32768]),
            ring_angles_a: PlainBytes::new([0; 1032]),
            ring_angles_b: PlainBytes::new([0; 1032]),
            pixel_table: PlainDword::new(0x0),
            middle: [const { PlainFloat::new(0.0) }; 2],
            _gap_8445de: PlainBytes::new([0; 4]),
            row_distance: PlainFloat::new(0.0),
            pixel_corners: PlainDword::new(0x0),
            _gap_8445ea: PlainBytes::new([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x6F, 0x6C, 0x31, 0x2E, 0x64,
                0x61, 0x74, 0x00, 0x63, 0x6F, 0x6C, 0x32, 0x2E, 0x64, 0x61,
            ]),
        }
    }
}

impl MaskVars {
    pub const START: usize = 0x82E342;
    pub const LEN: usize = 0x162C0;
}

#[cfg(test)]
mod tests_masks {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn masks_starts_as_the_games() {
        assert_eq!(size_of::<MaskVars>(), MaskVars::LEN);
        assert_eq!(offset_of!(MaskVars, spans_total), 0x0);
        assert_eq!(offset_of!(MaskVars, list), 0x4);
        assert_eq!(offset_of!(MaskVars, work), 0x44);
        assert_eq!(offset_of!(MaskVars, count), 0x48);
        assert_eq!(offset_of!(MaskVars, corner_scale), 0x4c);
        assert_eq!(offset_of!(MaskVars, turn), 0x54);
        assert_eq!(offset_of!(MaskVars, slope), 0x84);
        assert_eq!(offset_of!(MaskVars, work_pixel), 0x90);
        assert_eq!(offset_of!(MaskVars, lines), 0x98);
        assert_eq!(offset_of!(MaskVars, ends), 0xc8);
        assert_eq!(offset_of!(MaskVars, ring_corners), 0xe0);
        assert_eq!(offset_of!(MaskVars, corners_a), 0xe4);
        assert_eq!(offset_of!(MaskVars, corners_b), 0xcfc);
        assert_eq!(offset_of!(MaskVars, shapes), 0x1914);
        assert_eq!(offset_of!(MaskVars, control_word), 0x19a4);
        assert_eq!(offset_of!(MaskVars, status_word), 0x19a6);
        assert_eq!(offset_of!(MaskVars, height), 0x19ac);
        assert_eq!(offset_of!(MaskVars, row_int), 0x19b0);
        assert_eq!(offset_of!(MaskVars, row), 0x19b4);
        assert_eq!(offset_of!(MaskVars, slopes), 0x19b8);
        assert_eq!(offset_of!(MaskVars, xs), 0x19c0);
        assert_eq!(offset_of!(MaskVars, split), 0x19c8);
        assert_eq!(offset_of!(MaskVars, cursor), 0x19cc);
        assert_eq!(offset_of!(MaskVars, runs), 0x19d0);
        assert_eq!(offset_of!(MaskVars, edge), 0x19d4);
        assert_eq!(offset_of!(MaskVars, run_table), 0x39ec);
        assert_eq!(offset_of!(MaskVars, triangle), 0x59ec);
        assert_eq!(offset_of!(MaskVars, quad_triangle), 0x5a04);
        assert_eq!(offset_of!(MaskVars, ring_quad), 0x5a1c);
        assert_eq!(offset_of!(MaskVars, run_count), 0x5a3c);
        assert_eq!(offset_of!(MaskVars, ring_ends), 0x5a40);
        assert_eq!(offset_of!(MaskVars, ring_middle), 0x5a48);
        assert_eq!(offset_of!(MaskVars, ring_half), 0x5a4c);
        assert_eq!(offset_of!(MaskVars, ring_angles), 0x5a50);
        assert_eq!(offset_of!(MaskVars, ring_turns), 0x5a58);
        assert_eq!(offset_of!(MaskVars, ring_angle), 0x5a68);
        assert_eq!(offset_of!(MaskVars, ring_step), 0x5a6c);
        assert_eq!(offset_of!(MaskVars, pixel), 0x5a70);
        assert_eq!(offset_of!(MaskVars, mask_value), 0x5a78);
        assert_eq!(offset_of!(MaskVars, mask_a), 0x5a80);
        assert_eq!(offset_of!(MaskVars, mask_b), 0xda80);
        assert_eq!(offset_of!(MaskVars, ring_angles_a), 0x15a80);
        assert_eq!(offset_of!(MaskVars, ring_angles_b), 0x15e88);
        assert_eq!(offset_of!(MaskVars, pixel_table), 0x16290);
        assert_eq!(offset_of!(MaskVars, middle), 0x16294);
        assert_eq!(offset_of!(MaskVars, row_distance), 0x162a0);
        assert_eq!(offset_of!(MaskVars, pixel_corners), 0x162a4);
        let p = core::ptr::addr_of!(crate::effects::owner::effects().masks.vars).cast::<u8>();
        let now: Vec<u8> = (0..MaskVars::LEN)
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            crate::test_support::first_values::holds(
                "effects.owner.EFFECTS.masks.vars/90816",
                &now
            ),
            "MASKS starts differently from the image"
        );
    }
}
