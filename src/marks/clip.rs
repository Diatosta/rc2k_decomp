//! The clip's and the glow's variables, out of the marks' block (phase 7, typed state):
//! `ClipScreen` (IDA 0x7687E4 .. 0x7688B4) and `GlowState` (0x93CC70 .. 0x93CCB0).

use crate::session::{Bytes, Dword, Float};

/// The clip's screen edges and work vertices (IDA 0x7687E4 .. 0x7688B4, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct ClipScreen {
    /// 0x7687E4: `g_marksScreenMiddleX`, `g_marksScreenMiddleXLess`, `g_marksScreenMiddleY`, `g_marksScreenMiddleYLess`: the screen's middle and one less (`marks::init_marks` sets them; `marks::clip_quad` clips against the settings' edge).
    pub edges: [Float; 4],
    /// 0x7687F4: `g_marksClipVertices`: six `TLVertex`s (0x20 bytes each) `marks::clip_quad` builds the clipped quad in.
    pub vertices: [[Dword; 8]; 6],
}

pub static CLIP: ClipScreen = ClipScreen {
    edges: [
        Float::from_bits(0x43C80000),
        Float::from_bits(0x43C78000),
        Float::from_bits(0x43960000),
        Float::from_bits(0x43958000),
    ],
    vertices: [const { [const { Dword::new(0x0) }; 8] }; 6],
};

impl ClipScreen {
    pub const START: usize = 0x7687E4;
    pub const LEN: usize = 0xD0;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::host_number(&CLIP as *const ClipScreen)
    }
}

/// The glow's variables (and the clip's count) (IDA 0x93CC70 .. 0x93CCB0, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct GlowState {
    /// 0x93CC70: `dword_93CC70`: 1 = no glow (initially 1).
    pub off: Dword,
    /// 0x93CC74: `flt_93CC74`, `flt_93CC78`, `flt_93CC7C`: the glow's place (as the sprite's x, y, z are taken: z, y, x).
    pub place: [Float; 3],
    /// 0x93CC80: Not named: 20 bytes (a gap, or unreferenced data).
    pub _gap_93cc80: Bytes<20>,
    /// 0x93CC94: `dword_93CC94`: the vertex count `marks::clip_quad` leaves (nothing ported reads it; it sits in the glow's block).
    pub clip_count: Dword,
    /// 0x93CC98: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_93cc98: Bytes<12>,
    /// 0x93CCA4: `dword_93CCA4`: the glow sprite's angle.
    pub angle: Dword,
    /// 0x93CCA8: `flt_93CCA8`, `flt_93CCAC`: added to the place's z and y.
    pub offset: [Float; 2],
}

pub static GLOW: GlowState = GlowState {
    off: Dword::new(0x1),
    place: [
        Float::from_bits(0x48800000),
        Float::from_bits(0xC7800000),
        Float::from_bits(0xC7800000),
    ],
    _gap_93cc80: Bytes::new([0; 20]),
    clip_count: Dword::new(0x0),
    _gap_93cc98: Bytes::new([0; 12]),
    angle: Dword::new(0x0),
    offset: [Float::from_bits(0x0), Float::from_bits(0x0)],
};

impl GlowState {
    pub const START: usize = 0x93CC70;
    pub const LEN: usize = 0x40;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::host_number(&GLOW as *const GlowState)
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
    fn the_variables_start_as_the_games() {
        assert_eq!(size_of::<ClipScreen>(), ClipScreen::LEN);
        assert_eq!(size_of::<GlowState>(), GlowState::LEN);
        let image = crate::moved::image_bytes;
        assert!(
            bytes(&CLIP) == image(ClipScreen::START, ClipScreen::LEN),
            "CLIP starts differently from the image"
        );
        assert!(
            bytes(&GLOW) == image(GlowState::START, GlowState::LEN),
            "GLOW starts differently from the image"
        );
        assert_eq!(
            core::mem::offset_of!(ClipScreen, vertices),
            0x7687F4 - ClipScreen::START
        );
        assert_eq!(
            core::mem::offset_of!(GlowState, angle),
            0x93CCA4 - GlowState::START
        );
    }
}
