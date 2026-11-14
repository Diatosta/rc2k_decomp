//! The masks' variables.

use crate::session::{PlainBytes, PlainDword, PlainFloat, PlainWord};

/// One of the masks' shapes (0x24 bytes).
#[derive(rc2k_derive::Cells)]
pub struct Shape {
    /// 0x0: the step number of the shape's eased turn: `pick_shapes` sets it to the absolute value
    /// of the picks table's entry; `turn_all_shapes` passes it (and one more) to the cosine easing
    /// between the set's turn angle and the work step; `expire_windscreen_flakes` reads its low
    /// byte as the square-ends test input (a reading from these uses).
    pub ease_step: PlainDword,
    /// 0x4: the shape's count of ring quads (0x80)
    pub count: PlainDword,
    /// 0x8: the offset in `run_table` of its runs of rows (`span_ring` puts the runs' place here)
    pub rows: PlainDword,
    /// 0xC: its middle (x, y)
    pub middle: [PlainFloat; 2],
    /// 0x14: the ring's two angles' ends (times 0.0175)
    pub angle_ends: [PlainFloat; 2],
    /// 0x1C: the ring's two radii
    pub radii: [PlainFloat; 2],
}

/// The masks' variables: the work block's list, the corner tables, the shapes, the rasteriser's
/// scratch, the runs, the masks and the ring angles.
#[derive(rc2k_derive::Cells)]
pub struct MaskVars {
    /// The spans made
    pub spans_total: PlainDword,
    /// The work block's memory list
    pub list: crate::memory::ListCells,
    /// The 48 bytes after the list (no code reads or writes them).
    pub list_rest: PlainBytes<48>,
    /// The 0x80000-byte work block (Rust's, `Masks::work`; this stays 0)
    pub work: PlainDword,
    /// The running count `run_shapes` moves the shape index by
    pub count: PlainDword,
    /// The corner tables' scales
    pub corner_scale: [PlainFloat; 2],
    /// A 3 x 3 turn (the 2 x 2 turn is its elements 0, 1, 3, 4)
    pub turn: [PlainFloat; 9],
    /// The line's span and rise
    pub slope: [PlainFloat; 2],
    /// The pixel worked on
    pub work_pixel: [PlainDword; 2],
    /// Two lines (x, y; x, y; x, y; 0x18 bytes each)
    pub lines: [PlainDword; 12],
    /// The line's ends turned (the third is left)
    pub ends: [PlainFloat; 5],
    /// The corner table in use (its number, 0 for A and 1 for B)
    pub ring_corners: PlainDword,
    /// 0x81 entries of two corners
    pub corners_a: PlainBytes<3096>,
    /// The second corner table (the same layout)
    pub corners_b: PlainBytes<3096>,
    /// The shapes A and B
    pub shapes: [Shape; 2],
    /// The x87 control word the triangle came with
    pub control_word: PlainWord,
    /// The compares' status word
    pub status_word: PlainWord,
    pub height: PlainFloat,
    pub row_int: PlainDword,
    pub row: PlainFloat,
    pub slopes: [PlainFloat; 2],
    pub xs: [PlainFloat; 2],
    pub split: PlainFloat,
    /// The work block's cursor (a byte offset in `Masks::work`, as are the runs' span places)
    pub cursor: PlainDword,
    /// Where the next run goes (a byte offset in `run_table`)
    pub runs: PlainDword,
    /// Y0, y1, the left and right x at y0, at y1
    pub edge: [PlainDword; 6],
    /// The runs (16 bytes each)
    pub run_table: PlainBytes<8192>,
    /// The sorted corners
    pub triangle: [PlainFloat; 6],
    /// A quad's triangle
    pub quad_triangle: [PlainFloat; 6],
    /// The ring's quad
    pub ring_quad: [PlainFloat; 8],
    /// The runs made
    pub run_count: PlainDword,
    pub ring_ends: [PlainFloat; 2],
    pub ring_middle: PlainFloat,
    pub ring_half: PlainFloat,
    pub ring_angles: [PlainFloat; 2],
    pub ring_turns: [PlainFloat; 4],
    pub ring_angle: PlainDword,
    pub ring_step: PlainFloat,
    /// The pixel's x, then y
    pub pixel: [PlainDword; 2],
    /// The shape's number
    pub mask_value: PlainDword,
    /// Rows of 0x100
    pub mask_a: PlainBytes<32768>,
    /// The second mask (the same layout)
    pub mask_b: PlainBytes<32768>,
    /// The ring angles, table A
    pub ring_angles_a: PlainBytes<1032>,
    /// The ring angles, table B
    pub ring_angles_b: PlainBytes<1032>,
    /// The table in use (rows of 0x400; its shape's number, `MaskPicks.pixel_table_a` or `_b`)
    pub pixel_table: PlainDword,
    /// The shapes' middle
    pub middle: [PlainFloat; 2],
    pub row_distance: PlainFloat,
    /// The corner table kept (its shape's number)
    pub pixel_corners: PlainDword,
}

impl MaskVars {
    pub const fn new() -> Self {
        MaskVars {
            spans_total: PlainDword::new(0x0),
            list: crate::memory::ListCells::empty(),
            list_rest: PlainBytes::new([0; 48]),
            work: PlainDword::new(0x0),
            count: PlainDword::new(0x0),
            corner_scale: [const { PlainFloat::new(0.0) }; 2],
            turn: [const { PlainFloat::new(0.0) }; 9],
            slope: [const { PlainFloat::new(0.0) }; 2],
            work_pixel: [const { PlainDword::new(0x0) }; 2],
            lines: [const { PlainDword::new(0x0) }; 12],
            ends: [const { PlainFloat::new(0.0) }; 5],
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
            control_word: PlainWord::new(0x0),
            status_word: PlainWord::new(0x0),
            height: PlainFloat::new(0.0),
            row_int: PlainDword::new(0x0),
            row: PlainFloat::new(0.0),
            slopes: [const { PlainFloat::new(0.0) }; 2],
            xs: [const { PlainFloat::new(0.0) }; 2],
            split: PlainFloat::new(0.0),
            cursor: PlainDword::new(0x0),
            runs: PlainDword::new(0x0),
            edge: [const { PlainDword::new(0x0) }; 6],
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
            mask_a: PlainBytes::new([0; 32768]),
            mask_b: PlainBytes::new([0; 32768]),
            ring_angles_a: PlainBytes::new([0; 1032]),
            ring_angles_b: PlainBytes::new([0; 1032]),
            pixel_table: PlainDword::new(0x0),
            middle: [const { PlainFloat::new(0.0) }; 2],
            row_distance: PlainFloat::new(0.0),
            pixel_corners: PlainDword::new(0x0),
        }
    }
}
