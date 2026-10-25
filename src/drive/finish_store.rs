//! The finish routine's variables (IDA 0x64BB18 .. 0x64BB88; phase 7, typed): the
//! "no place" switch, the stage numbers of the finish, the finish lengths and limits, the
//! clock at the finish's start, the time, the eased time and the share. `#[repr(C)]` with the
//! image's layout, over the race block's bytes (`FINISH`). Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
pub struct Finish {
    /// +0x0 (0x64BB18): `dword_64BB18`: set while the race has no place or panel to draw (`race::car_starts`, `race::draw_mirror_view`, `race::draw_race_scene`, `race::loadlevel`, `race::stage_loading`, `physics::passed_split_point`, `drive::finish_race`).
    pub no_place: Dword,
    /// +0x4 (0x64BB1C): `dword_64BB1C`: the stage number for each finish stage index (`drive::finish_race`).
    pub finish_stages: [Dword; 7],
    /// +0x20 (0x64BB38): `flt_64BB38`: not used by the Rust side as a variable (an x87 run reads it by RVA).
    pub unknown_20: Float,
    /// +0x24 (0x64BB3C): `flt_64BB3C`: the time the finish lasts, by `finished` 0 or 1 (`drive::finish_run`).
    pub finish_lengths: [Float; 2],
    /// +0x2c (0x64BB44): `flt_64BB44`: the first limit of the eased time (`drive::finish_timing`).
    pub limit_a: Float,
    /// +0x30 (0x64BB48): `flt_64BB48`: the second limit.
    pub limit_b: Float,
    /// +0x34 (0x64BB4C): `flt_64BB4C` .. `flt_64BB5C`: not worked out.
    pub unknown_2c: [Float; 5],
    /// +0x48 (0x64BB60): `dword_64BB60`: the scene clock at the finish's start (`drive::finish_run`, `drive::finish_timing`).
    pub start_clock: Dword,
    /// +0x4c (0x64BB64): `flt_64BB64`: the time since the start (`drive::finish_timing`, `drive::finish_run`).
    pub time: Float,
    /// +0x50 (0x64BB68): `flt_64BB68`: the eased time.
    pub eased: Float,
    /// +0x54 (0x64BB6C): `flt_64BB6C`: the share of the last stretch, 0 to 1.
    pub share: Float,
    /// +0x58 (0x64BB70): `flt_64BB70`: the share's first limit.
    pub limit_c: Float,
    /// +0x5c (0x64BB74): `flt_64BB74`: the share's second limit.
    pub limit_d: Float,
    /// +0x60 (0x64BB78): `flt_64BB78`: not worked out.
    pub unknown_60: Float,
    /// +0x64 (0x64BB7C): `flt_64BB7C`: not worked out.
    pub unknown_64: Float,
    /// +0x68 (0x64BB80): `dword_64BB80`, `dword_64BB84`: not worked out.
    pub unknown_68: [Dword; 2],
}

/// The piece: the race block's bytes at IDA 0x64BB18 seen as `Finish`.
pub struct InBlock;

impl core::ops::Deref for InBlock {
    type Target = Finish;
    fn deref(&self) -> &Finish {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(piece_addr() as *const Finish) }
    }
}

pub static FINISH: InBlock = InBlock;

/// IDA 0x64BB18, the first byte of the piece.
pub const START: usize = 0x64BB18;
/// IDA 0x64BB88, the end of the piece.
pub const END: usize = 0x64BB88;

const _: () = assert!(END - START == core::mem::size_of::<Finish>());

/// Where IDA 0x64BB18 is now.
pub fn piece_addr() -> usize {
    crate::moved::place(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );
        assert_eq!(offset_of!(Finish, finish_stages), 0x64BB1C - START);
        assert_eq!(offset_of!(Finish, finish_lengths), 0x64BB3C - START);
        assert_eq!(offset_of!(Finish, limit_a), 0x64BB44 - START);
        assert_eq!(offset_of!(Finish, start_clock), 0x64BB60 - START);
        assert_eq!(offset_of!(Finish, time), 0x64BB64 - START);
        assert_eq!(offset_of!(Finish, share), 0x64BB6C - START);
        assert_eq!(offset_of!(Finish, limit_d), 0x64BB74 - START);
        assert_eq!(offset_of!(Finish, unknown_68), 0x64BB80 - START);
    }
}
