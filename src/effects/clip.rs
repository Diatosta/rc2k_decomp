//! The clip's and the glow's variables:
//! `ClipScreen` (IDA 0x7687E4 .. 0x7688B4) and `GlowState` (0x93CC70 .. 0x93CCB0).

use crate::d3d::vertices_store::Vertex;
#[cfg(test)]
use crate::effects::owner::effects;
use crate::session::{Dword, Float, PlainBytes, PlainDword, PlainFloat};

/// The clip's screen edges and work vertices (IDA 0x7687E4 .. 0x7688B4).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ClipScreen {
    /// 0x7687E4: `g_marksScreenMiddleX`, `g_marksScreenMiddleXLess`, `g_marksScreenMiddleY`, `g_marksScreenMiddleYLess`: the screen's middle and one less (`effects::init_marks` sets them; `effects::clip_quad` clips against the settings' edge).
    pub edges: [PlainFloat; 4],
    /// 0x7687F4: `g_marksClipVertices`: six `TLVertex`s (0x20 bytes each) `effects::clip_quad` builds the clipped quad in.
    pub vertices: [Vertex; 6],
}

const fn zero_vertex() -> Vertex {
    Vertex {
        x: Float::new(0.0),
        y: Float::new(0.0),
        z: Float::new(0.0),
        rhw: Float::new(0.0),
        colour: Dword::new(0),
        specular: Dword::new(0),
        u: Float::new(0.0),
        v: Float::new(0.0),
    }
}

impl ClipScreen {
    pub const fn new() -> Self {
        Self {
            edges: [
                PlainFloat::new(400.0),
                PlainFloat::new(399.0),
                PlainFloat::new(300.0),
                PlainFloat::new(299.0),
            ],
            vertices: [const { zero_vertex() }; 6],
        }
    }
}

impl ClipScreen {
    pub const START: usize = 0x7687E4;
    pub const LEN: usize = 0xD0;
}

/// The glow's variables (and the clip's count) (IDA 0x93CC70 .. 0x93CCB0).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct GlowState {
    /// 0x93CC70: `dword_93CC70`: 1 = no glow (initially 1).
    pub off: PlainDword,
    /// 0x93CC74: `flt_93CC74`, `flt_93CC78`, `flt_93CC7C`: the glow's place (as the sprite's x, y, z are taken: z, y, x).
    pub place: [PlainFloat; 3],
    /// 0x93CC80: Not named: 20 bytes (a gap, or unreferenced data).
    pub _gap_93cc80: PlainBytes<20>,
    /// 0x93CC94: `dword_93CC94`: the vertex count `effects::clip_quad` leaves (nothing ported reads it; it sits in the glow's block).
    pub clip_count: PlainDword,
    /// 0x93CC98: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_93cc98: PlainBytes<12>,
    /// 0x93CCA4: `dword_93CCA4`: the glow sprite's angle.
    pub angle: PlainDword,
    /// 0x93CCA8: `flt_93CCA8`, `flt_93CCAC`: added to the place's z and y.
    pub offset: [PlainFloat; 2],
}

impl GlowState {
    pub const fn new() -> Self {
        Self {
            off: PlainDword::new(0x1),
            place: [
                PlainFloat::new(262144.0),
                PlainFloat::new(-65536.0),
                PlainFloat::new(-65536.0),
            ],
            _gap_93cc80: PlainBytes::new([0; 20]),
            clip_count: PlainDword::new(0x0),
            _gap_93cc98: PlainBytes::new([0; 12]),
            angle: PlainDword::new(0x0),
            offset: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
        }
    }
}

impl GlowState {
    pub const START: usize = 0x93CC70;
    pub const LEN: usize = 0x40;
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
    fn the_variables_start_as_the_games() {
        let clipping = &effects().clipping;
        assert_eq!(size_of::<ClipScreen>(), ClipScreen::LEN);
        assert_eq!(size_of::<GlowState>(), GlowState::LEN);
        let image = crate::moved::image_bytes;
        assert!(
            bytes(&clipping.clip) == image(ClipScreen::START, ClipScreen::LEN),
            "CLIP starts differently from the image"
        );
        assert!(
            bytes(&clipping.glow) == image(GlowState::START, GlowState::LEN),
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
