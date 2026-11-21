//! The wheel grit bursts' tuning.

impl BurstTuning {
    /// The half's (`n` 0 or 1) rate, count and entry randoms.
    pub fn randoms(&self, n: usize) -> &[u32; 3] {
        if n == 0 { &self.first } else { &self.second }
    }

    /// The half's (`n` 0 or 1) rate, count and entry randoms, to advance.
    pub fn randoms_mut(&mut self, n: usize) -> &mut [u32; 3] {
        if n == 0 {
            &mut self.first
        } else {
            &mut self.second
        }
    }
}

/// One burst emitter's factors and randoms (`emit_wheel_grit_bursts`' and `emit_bursts_kind_3`'s).
pub struct BurstTuning {
    /// The strength's factor
    pub strength_k: f32,
    /// The first half's rate, count and entry randoms
    pub first: [u32; 3],
    /// The blend's two factors
    pub blend_k: [f32; 2],
    /// The second half's rate, count and entry randoms
    pub second: [u32; 3],
}

impl BurstTuning {
    /// The values the game starts with.
    pub const fn first() -> Self {
        BurstTuning {
            strength_k: -0.25,
            first: [0x0; 3],
            blend_k: [-0.25, -0.05],
            second: [0x0; 3],
        }
    }
}
