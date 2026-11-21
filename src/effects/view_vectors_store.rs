//! The marks' view vectors: `ViewSources` and `KeptView`. A reading: the vectors the car view hands
//! the marks, kept from frame to frame and turned.

/// A factor, a vector, and the vector scaled by the factor (0x1C bytes).
#[derive(Clone, Copy)]
pub struct VectorSource {
    pub factor: f32,
    pub vector: [f32; 3],
    pub scaled: [f32; 3],
}

/// The marks' view vectors the frame starts from.
#[derive(Clone, Copy)]
pub struct ViewSources {
    /// Two direction vectors with a length factor each (a factor, a vector, the vector scaled by
    /// the factor; at first 5.0 with (0, -1, 0), and 0 with (0, 0, -1)); `effects::keep_vectors`
    /// copies them (with a third, `pushed`) into `KeptView` each frame,
    /// `effects::scale_kept_vectors` scales them, and `effects::aim_vector` / `view_step` read the
    /// scaled results (probably the directions the marks' view is aimed along).
    pub sources: [VectorSource; 2],
}

impl ViewSources {
    pub const fn new() -> Self {
        Self {
            sources: [
                VectorSource {
                    factor: 5.0,
                    vector: [0.0, -1.0, 0.0],
                    scaled: [0.0, -1.0, 0.0],
                },
                VectorSource {
                    factor: 0.0,
                    vector: [0.0, 0.0, -1.0],
                    scaled: [0.0, 0.0, -1.0],
                },
            ],
        }
    }
}

/// The kept vectors of the marks' view.
#[derive(Clone, Copy)]
pub struct KeptView {
    /// The sine and cosine of `first_angle` (`effects::turn_kept_vectors`).
    pub first_sine_cosine: [f32; 2],
    /// The first angle the kept vectors are turned by (z and y); also read by the spawning of
    /// windscreen drops and squares.
    pub first_angle: f32,
    /// The sine and cosine of `second_angle`.
    pub second_sine_cosine: [f32; 2],
    /// The second angle (x and z).
    pub second_angle: f32,
    /// A factor (16.0 at first) and a vector: kept by `effects::keep_vectors` into `kept_a`.
    pub pushed: [f32; 4],
    /// A factor, a vector and the scaled vector; `effects::keep_vectors` fills the first two,
    /// `scale_kept_vectors` the third.
    pub kept_a: VectorSource,
    /// `kept_a`'s scaled vector at the last frame (`effects::view_step`).
    pub last: [f32; 3],
    /// As `kept_a`, from `sources[0]`.
    pub kept_b: VectorSource,
    /// As `kept_a`, from `sources[1]`.
    pub kept_c: VectorSource,
    /// The view's step since the last frame, negated (`effects::view_step`).
    pub step: [f32; 3],
    /// The length of `vector` (`effects::aim_vector`).
    pub length: f32,
    /// `vector` made of length 1.
    pub unit: [f32; 3],
    /// The aim: minus `kept_b`'s scaled, less `kept_a`'s scaled, plus `kept_c`'s scaled, plus
    /// `step`.
    pub vector: [f32; 3],
}

impl KeptView {
    pub const fn new() -> Self {
        Self {
            first_sine_cosine: [0.0, 0.0],
            first_angle: 0.0,
            second_sine_cosine: [0.0, 0.0],
            second_angle: 0.0,
            pushed: [16.0, 0.0, 0.0, 1.0],
            kept_a: VectorSource {
                factor: 0.0,
                vector: [0.0, 0.0, 0.0],
                scaled: [0.0, 0.0, 0.0],
            },
            last: [0.0, 0.0, 0.0],
            kept_b: VectorSource {
                factor: 1.0,
                vector: [0.0, 0.0, 0.0],
                scaled: [0.0, 0.0, 0.0],
            },
            kept_c: VectorSource {
                factor: 1.0,
                vector: [0.0, 0.0, 0.0],
                scaled: [0.0, 0.0, 0.0],
            },
            step: [0.0, 0.0, 0.0],
            length: 1.0,
            unit: [0.0, 0.0, 0.0],
            vector: [0.0, 0.0, 0.0],
        }
    }
}
