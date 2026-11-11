//! The marks' view vectors: `ViewSources`
//! (IDA 0x764F90 .. 0x764FC8) and `KeptView` (0x769236 .. 0x769312). A reading: the
//! vectors the car view hands the marks, kept from frame to frame and turned.

#[cfg(test)]
use crate::effects::owner::effects;
use crate::session::{PlainBytes, PlainFloat};

/// A factor, a vector, and the vector scaled by the factor (0x1C bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct VectorSource {
    pub factor: PlainFloat,
    pub vector: [PlainFloat; 3],
    pub scaled: [PlainFloat; 3],
}

/// The marks' view vectors the frame starts from (IDA 0x764F90 .. 0x764FC8).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewSources {
    /// 0x764F90: `dword_764F90` (a factor), `dword_764F94` .. (a vector), `flt_764FA0` .. (the vector scaled by the factor; `effects::scale_vectors_by`), then the same at `dword_764FAC` ..: two direction vectors with a length factor (the image starts them as 5.0 with (0, -1, 0), and 0 with (0, 0, -1)); `effects::keep_vectors` copies them (with a third, `pushed`) into `KeptView` each frame, `effects::scale_kept_vectors` scales them, and `effects::aim_vector` / `view_step` read the scaled results (probably the directions the marks' view is aimed along).
    pub sources: [VectorSource; 2],
}

impl ViewSources {
    pub const fn new() -> Self {
        Self {
            sources: [
                VectorSource {
                    factor: PlainFloat::new(5.0),
                    vector: [
                        PlainFloat::new(0.0),
                        PlainFloat::new(-1.0),
                        PlainFloat::new(0.0),
                    ],
                    scaled: [
                        PlainFloat::new(0.0),
                        PlainFloat::new(-1.0),
                        PlainFloat::new(0.0),
                    ],
                },
                VectorSource {
                    factor: PlainFloat::new(0.0),
                    vector: [
                        PlainFloat::new(0.0),
                        PlainFloat::new(0.0),
                        PlainFloat::new(-1.0),
                    ],
                    scaled: [
                        PlainFloat::new(0.0),
                        PlainFloat::new(0.0),
                        PlainFloat::new(-1.0),
                    ],
                },
            ],
        }
    }
}

impl ViewSources {
    pub const START: usize = 0x764F90;
    pub const LEN: usize = 0x38;
}

/// The kept vectors of the marks' view (IDA 0x769236 .. 0x769312).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct KeptView {
    /// 0x769236: `flt_769236`, `flt_76923A`: the sine and cosine of `first_angle` (`effects::turn_kept_vectors`).
    pub first_sine_cosine: [PlainFloat; 2],
    /// 0x76923E: `flt_76923E`: the first angle the kept vectors are turned by (z and y); also read by the spawning of windscreen drops and squares.
    pub first_angle: PlainFloat,
    /// 0x769242: `flt_769242`, `flt_769246`: the sine and cosine of `second_angle`.
    pub second_sine_cosine: [PlainFloat; 2],
    /// 0x76924A: `flt_76924A`: the second angle (x and z).
    pub second_angle: PlainFloat,
    /// 0x76924E: `dword_76924E` (a factor, 16.0 at first), `dword_769252` .. (a vector): kept by `effects::keep_vectors` into `kept_a`.
    pub pushed: [PlainFloat; 4],
    /// 0x76925E: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_76925e: PlainBytes<12>,
    /// 0x76926A: `dword_76926A` (factor), `dword_76926E` .. (vector), `flt_76927A` .. (scaled); `effects::keep_vectors` fills the first two, `scale_kept_vectors` the third.
    pub kept_a: VectorSource,
    /// 0x769286: Not named: 16 bytes (a gap, or unreferenced data).
    pub _gap_769286: PlainBytes<16>,
    /// 0x769296: `flt_769296` ..: `kept_a`'s scaled vector at the last frame (`effects::view_step`).
    pub last: [PlainFloat; 3],
    /// 0x7692A2: `dword_7692A2` .. : as `kept_a`, from `sources[0]`.
    pub kept_b: VectorSource,
    /// 0x7692BE: `dword_7692BE` .. : as `kept_a`, from `sources[1]`.
    pub kept_c: VectorSource,
    /// 0x7692DA: Not named: 16 bytes (a gap, or unreferenced data).
    pub _gap_7692da: PlainBytes<16>,
    /// 0x7692EA: `flt_7692EA` ..: the view's step since the last frame, negated (`effects::view_step`).
    pub step: [PlainFloat; 3],
    /// 0x7692F6: `dword_7692F6`: the length of `vector` (`effects::aim_vector`).
    pub length: PlainFloat,
    /// 0x7692FA: `unk_7692FA` ..: `vector` made of length 1.
    pub unit: [PlainFloat; 3],
    /// 0x769306: `dword_769306` ..: the aim: minus `kept_b`'s scaled, less `kept_a`'s scaled, plus `kept_c`'s scaled, plus `step`.
    pub vector: [PlainFloat; 3],
}

impl KeptView {
    pub const fn new() -> Self {
        Self {
            first_sine_cosine: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            first_angle: PlainFloat::new(0.0),
            second_sine_cosine: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            second_angle: PlainFloat::new(0.0),
            pushed: [
                PlainFloat::new(16.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
            _gap_76925e: PlainBytes::new([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
            ]),
            kept_a: VectorSource {
                factor: PlainFloat::new(0.0),
                vector: [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                scaled: [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
            },
            _gap_769286: PlainBytes::new([
                0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00,
            ]),
            last: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            kept_b: VectorSource {
                factor: PlainFloat::new(1.0),
                vector: [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                scaled: [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
            },
            kept_c: VectorSource {
                factor: PlainFloat::new(1.0),
                vector: [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
                scaled: [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ],
            },
            _gap_7692da: PlainBytes::new([
                0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00,
            ]),
            step: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            length: PlainFloat::new(1.0),
            unit: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            vector: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
        }
    }
}

impl KeptView {
    pub const START: usize = 0x769236;
    pub const LEN: usize = 0xDC;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn the_vectors_start_as_the_games() {
        let frame = &effects().frame;
        assert_eq!(size_of::<VectorSource>(), 0x1C);
        assert_eq!(size_of::<ViewSources>(), ViewSources::LEN);
        assert_eq!(size_of::<KeptView>(), KeptView::LEN);
        let image = crate::moved::image_bytes;
        assert!(
            bytes(&frame.view_sources) == image(ViewSources::START, ViewSources::LEN),
            "SOURCES starts differently from the image"
        );
        assert!(
            bytes(&frame.kept_view) == image(KeptView::START, KeptView::LEN),
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
