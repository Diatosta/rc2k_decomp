//! The camera's places and the lights' data read from the stage's file.

/// The lights' data in bytes (0x22 records of 0x90 bytes).
pub const LIGHTS_BYTES: usize = 0x1320;

/// The camera's two sets of seven figures (`physics::place_camera`): constants.
pub struct LightsData {
    /// Seven figures of the camera's place (`physics::place_camera`, the usual mode).
    pub place_usual: [u32; 7],
    /// Seven figures of the camera's place (`physics::place_camera`, the second record mode).
    pub place_second: [u32; 7],
}

impl LightsData {
    pub const fn new() -> Self {
        LightsData {
            place_usual: [
                0x3FA0_624E,
                0xC0B8_0000,
                0x400F_AE14,
                0xC0EC_0000,
                0x404F_3B64,
                0xC110_0000,
                0x3C80_0000,
            ],
            place_second: [
                0x3E80_0000,
                0xC0D8_0000,
                0x3EEC_CCCD,
                0xC106_0000,
                0x3F2C_CCCD,
                0xC120_0000,
                0x3CBC_6A7F,
            ],
        }
    }
}

/// The lights' data (`race::load_lights_data`, `race::copy_lights_data`).
pub struct LightsTable {
    /// 0x22 records of 0x90 bytes.
    pub lights: [u8; LIGHTS_BYTES],
}

impl LightsTable {
    pub const fn new() -> Self {
        LightsTable {
            lights: [0; LIGHTS_BYTES],
        }
    }

    /// The little-endian dword at byte `offset` (any alignment); past the table ends the program.
    pub fn get_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(self.lights[offset..offset + 4].try_into().unwrap())
    }

    pub fn set_u32(&mut self, offset: usize, value: u32) {
        self.lights[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}
