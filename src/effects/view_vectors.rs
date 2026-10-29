//! The marks' view vectors, out of the marks' block (phase 7, typed state): `ViewSources`
//! (IDA 0x764F90 .. 0x764FC8) and `KeptView` (0x769236 .. 0x769312). A reading: the
//! vectors the car view hands the marks, kept from frame to frame and turned.

use crate::session::{Bytes, Dword, Float};

/// A factor, a vector, and the vector scaled by the factor (0x1C bytes).
#[repr(C)]
pub struct VectorSource {
    pub factor: Dword,
    pub vector: [Dword; 3],
    pub scaled: [Float; 3],
}

/// The marks' view vectors the frame starts from (IDA 0x764F90 .. 0x764FC8, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct ViewSources {
    /// 0x764F90: `dword_764F90` (a factor), `dword_764F94` .. (a vector), `flt_764FA0` .. (the vector scaled by the factor; `effects::scale_vectors_by`), then the same at `dword_764FAC` ..: what `effects::keep_vectors` keeps and the car view sets (a reading).
    pub sources: [VectorSource; 2],
}

pub static SOURCES: ViewSources = ViewSources {
    sources: [
        VectorSource {
            factor: Dword::new(0x40A00000),
            vector: [Dword::new(0x0), Dword::new(0xBF800000), Dword::new(0x0)],
            scaled: [
                Float::from_bits(0x0),
                Float::from_bits(0xBF800000),
                Float::from_bits(0x0),
            ],
        },
        VectorSource {
            factor: Dword::new(0x0),
            vector: [Dword::new(0x0), Dword::new(0x0), Dword::new(0xBF800000)],
            scaled: [
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0xBF800000),
            ],
        },
    ],
};

impl ViewSources {
    pub const START: usize = 0x764F90;
    pub const LEN: usize = 0x38;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&SOURCES as *const ViewSources) as usize
    }
}

/// The kept vectors of the marks' view (IDA 0x769236 .. 0x769312, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct KeptView {
    /// 0x769236: `flt_769236`, `flt_76923A`: the sine and cosine of `first_angle` (`effects::turn_kept_vectors`).
    pub first_sine_cosine: [Float; 2],
    /// 0x76923E: `flt_76923E`: the first angle the kept vectors are turned by (z and y); also read by the spawning of windscreen drops and squares.
    pub first_angle: Float,
    /// 0x769242: `flt_769242`, `flt_769246`: the sine and cosine of `second_angle`.
    pub second_sine_cosine: [Float; 2],
    /// 0x76924A: `flt_76924A`: the second angle (x and z).
    pub second_angle: Float,
    /// 0x76924E: `dword_76924E` (a factor, 16.0 at first), `dword_769252` .. (a vector): kept by `effects::keep_vectors` into `kept_a`.
    pub pushed: [Dword; 4],
    /// 0x76925E: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_76925e: Bytes<12>,
    /// 0x76926A: `dword_76926A` (factor), `dword_76926E` .. (vector), `flt_76927A` .. (scaled); `effects::keep_vectors` fills the first two, `scale_kept_vectors` the third.
    pub kept_a: VectorSource,
    /// 0x769286: Not named: 16 bytes (a gap, or unreferenced data).
    pub _gap_769286: Bytes<16>,
    /// 0x769296: `flt_769296` ..: `kept_a`'s scaled vector at the last frame (`effects::view_step`).
    pub last: [Float; 3],
    /// 0x7692A2: `dword_7692A2` .. : as `kept_a`, from `sources[0]`.
    pub kept_b: VectorSource,
    /// 0x7692BE: `dword_7692BE` .. : as `kept_a`, from `sources[1]`.
    pub kept_c: VectorSource,
    /// 0x7692DA: Not named: 16 bytes (a gap, or unreferenced data).
    pub _gap_7692da: Bytes<16>,
    /// 0x7692EA: `flt_7692EA` ..: the view's step since the last frame, negated (`effects::view_step`).
    pub step: [Float; 3],
    /// 0x7692F6: `dword_7692F6`: the length of `vector` (`effects::aim_vector`).
    pub length: Dword,
    /// 0x7692FA: `unk_7692FA` ..: `vector` made of length 1.
    pub unit: [Float; 3],
    /// 0x769306: `dword_769306` ..: the aim: minus `kept_b`'s scaled, less `kept_a`'s scaled, plus `kept_c`'s scaled, plus `step`.
    pub vector: [Float; 3],
}

pub static KEPT: KeptView = KeptView {
    first_sine_cosine: [Float::from_bits(0x0), Float::from_bits(0x0)],
    first_angle: Float::from_bits(0x0),
    second_sine_cosine: [Float::from_bits(0x0), Float::from_bits(0x0)],
    second_angle: Float::from_bits(0x0),
    pushed: [
        Dword::new(0x41800000),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x3F800000),
    ],
    _gap_76925e: Bytes::new([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
    ]),
    kept_a: VectorSource {
        factor: Dword::new(0x0),
        vector: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
        scaled: [
            Float::from_bits(0x0),
            Float::from_bits(0x0),
            Float::from_bits(0x0),
        ],
    },
    _gap_769286: Bytes::new([
        0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]),
    last: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    kept_b: VectorSource {
        factor: Dword::new(0x3F800000),
        vector: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
        scaled: [
            Float::from_bits(0x0),
            Float::from_bits(0x0),
            Float::from_bits(0x0),
        ],
    },
    kept_c: VectorSource {
        factor: Dword::new(0x3F800000),
        vector: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
        scaled: [
            Float::from_bits(0x0),
            Float::from_bits(0x0),
            Float::from_bits(0x0),
        ],
    },
    _gap_7692da: Bytes::new([
        0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]),
    step: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    length: Dword::new(0x3F800000),
    unit: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    vector: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
};

impl KeptView {
    pub const START: usize = 0x769236;
    pub const LEN: usize = 0xDC;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&KEPT as *const KeptView) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = v as *const T as *const u8;
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn the_vectors_start_as_the_games() {
        assert_eq!(size_of::<VectorSource>(), 0x1C);
        assert_eq!(size_of::<ViewSources>(), ViewSources::LEN);
        assert_eq!(size_of::<KeptView>(), KeptView::LEN);
        let image = crate::moved::image_bytes;
        assert!(
            bytes(&SOURCES) == image(ViewSources::START, ViewSources::LEN),
            "SOURCES starts differently from the image"
        );
        assert!(
            bytes(&KEPT) == image(KeptView::START, KeptView::LEN),
            "KEPT starts differently from the image"
        );
        let k = KeptView::START;
        assert_eq!(offset_of!(KeptView, first_angle), 0x76923E - k);
        assert_eq!(offset_of!(KeptView, pushed), 0x76924E - k);
        assert_eq!(offset_of!(KeptView, kept_a), 0x76926A - k);
        assert_eq!(offset_of!(KeptView, last), 0x769296 - k);
        assert_eq!(offset_of!(KeptView, kept_b), 0x7692A2 - k);
        assert_eq!(offset_of!(KeptView, kept_c), 0x7692BE - k);
        assert_eq!(offset_of!(KeptView, step), 0x7692EA - k);
        assert_eq!(offset_of!(KeptView, vector), 0x769306 - k);
        assert_eq!(
            offset_of!(ViewSources, sources) + 0x1C,
            0x764FAC - ViewSources::START
        );
    }
}
