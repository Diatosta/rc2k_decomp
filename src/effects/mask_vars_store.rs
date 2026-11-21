//! The masks' variables.

/// One of the masks' shapes (0x24 bytes).
#[derive(Clone, Copy)]
pub struct Shape {
    /// 0x0: the step number of the shape's eased turn: `pick_shapes` sets it to the absolute value
    /// of the picks table's entry; `turn_all_shapes` passes it (and one more) to the cosine easing
    /// between the set's turn angle and the work step; `expire_windscreen_flakes` reads its low
    /// byte as the square-ends test input (a reading from these uses).
    pub ease_step: u32,
    /// 0x4: the shape's count of ring quads (0x80)
    pub count: u32,
    /// 0x8: the offset in `run_table` of its runs of rows (`span_ring` puts the runs' place here)
    pub rows: u32,
    /// 0xC: its middle (x, y)
    pub middle: [f32; 2],
    /// 0x14: the ring's two angles' ends (times 0.0175)
    pub angle_ends: [f32; 2],
    /// 0x1C: the ring's two radii
    pub radii: [f32; 2],
}

/// The masks' variables: the work block's list, the corner tables, the shapes, the rasteriser's
/// scratch, the runs, the masks and the ring angles.
pub struct MaskVars {
    /// The spans made
    pub spans_total: u32,
    /// The work block's memory list
    pub list: crate::memory::PlainList,
    /// The running count `run_shapes` moves the shape index by
    pub count: u32,
    /// The corner tables' scales
    pub corner_scale: [f32; 2],
    /// A 3 x 3 turn (the 2 x 2 turn is its elements 0, 1, 3, 4)
    pub turn: [f32; 9],
    /// The line's span and rise
    pub slope: [f32; 2],
    /// The pixel worked on
    pub work_pixel: [u32; 2],
    /// Two lines (x, y; x, y; x, y; 0x18 bytes each)
    pub lines: [u32; 12],
    /// The line's ends turned (the third is left)
    pub ends: [f32; 5],
    /// The corner table in use (its number, 0 for A and 1 for B)
    pub ring_corners: u32,
    /// The corner tables A and B: 0x81 entries of two corners each
    pub corners: [[u8; 3096]; 2],
    /// The shapes A and B
    pub shapes: [Shape; 2],
    /// The x87 control word the triangle came with
    pub control_word: u16,
    /// The compares' status word
    pub status_word: u16,
    pub height: f32,
    pub row_int: u32,
    pub row: f32,
    pub slopes: [f32; 2],
    pub xs: [f32; 2],
    pub split: f32,
    /// The work block's cursor (a byte offset in `Masks::work`, as are the runs' span places)
    pub cursor: u32,
    /// Where the next run goes (a byte offset in `run_table`)
    pub runs: u32,
    /// Y0, y1, the left and right x at y0, at y1
    pub edge: [u32; 6],
    /// The runs (16 bytes each)
    pub run_table: [u8; 8192],
    /// The sorted corners
    pub triangle: [f32; 6],
    /// A quad's triangle
    pub quad_triangle: [f32; 6],
    /// The ring's quad
    pub ring_quad: [f32; 8],
    /// The runs made
    pub run_count: u32,
    pub ring_ends: [f32; 2],
    pub ring_middle: f32,
    pub ring_half: f32,
    pub ring_angles: [f32; 2],
    pub ring_turns: [f32; 4],
    pub ring_angle: u32,
    pub ring_step: f32,
    /// The pixel's x, then y
    pub pixel: [u32; 2],
    /// The shape's number
    pub mask_value: u32,
    /// The masks A and B: rows of 0x100
    pub masks: [[u8; 32768]; 2],
    /// The ring angle tables A and B
    pub ring_angle_tables: [[u8; 1032]; 2],
    /// The table in use (rows of 0x400; its shape's number, `MaskPicks.pixel_table_a` or `_b`)
    pub pixel_table: u32,
    /// The shapes' middle
    pub middle: [f32; 2],
    pub row_distance: f32,
    /// The corner table kept (its shape's number)
    pub pixel_corners: u32,
}

impl MaskVars {
    pub const fn new() -> Self {
        MaskVars {
            spans_total: 0,
            list: crate::memory::PlainList::empty(),
            count: 0,
            corner_scale: [0.0; 2],
            turn: [0.0; 9],
            slope: [0.0; 2],
            work_pixel: [0; 2],
            lines: [0; 12],
            ends: [0.0; 5],
            ring_corners: 0,
            corners: [[0; 3096]; 2],
            shapes: [
                Shape {
                    ease_step: 0,
                    count: 0x80,
                    rows: 0,
                    middle: [18.0, 128.0],
                    angle_ends: [-5.0, 15.0],
                    radii: [110.0, 30.0],
                },
                Shape {
                    ease_step: 0,
                    count: 0x80,
                    rows: 0,
                    middle: [98.0, 128.0],
                    angle_ends: [-35.0, 70.0],
                    radii: [110.0, 30.0],
                },
            ],
            control_word: 0,
            status_word: 0,
            height: 0.0,
            row_int: 0,
            row: 0.0,
            slopes: [0.0; 2],
            xs: [0.0; 2],
            split: 0.0,
            cursor: 0,
            runs: 0,
            edge: [0; 6],
            run_table: [0; 8192],
            triangle: [0.0; 6],
            quad_triangle: [0.0; 6],
            ring_quad: [0.0; 8],
            run_count: 0,
            ring_ends: [0.0; 2],
            ring_middle: 0.0,
            ring_half: 0.0,
            ring_angles: [0.0; 2],
            ring_turns: [0.0; 4],
            ring_angle: 0,
            ring_step: 0.0,
            pixel: [0; 2],
            mask_value: 0,
            masks: [[0; 32768]; 2],
            ring_angle_tables: [[0; 1032]; 2],
            pixel_table: 0,
            middle: [0.0; 2],
            row_distance: 0.0,
            pixel_corners: 0,
        }
    }
}
