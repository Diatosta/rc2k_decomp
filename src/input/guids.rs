//! The DirectInput identifiers the input module names (the joystick device's interface, the
//! axes and the force-feedback effects), out of RAL.EXE's image (phase 5), typed (phase 7).
//! `#[repr(C)]` struct of byte buffers with the image's layout, first values the image's
//! (`tests`); `moved::MOVED` points the image's addresses at it: the effect templates
//! (`input::store::Devices`) point at the effects' GUIDs.

use crate::session::{Bytes, with_runs};

/// The DirectInput GUIDs (IDA 0x4F7E18 .. 0x4F7FD8, the image's layout).
#[repr(C, align(16))]
pub struct InputGuids {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: Bytes<8>,
    /// 0x4F7E18: `IID_IDirectInputDevice2A`: the joystick device's interface (`input::acquire_device`, `joystick_found`).
    pub device2: Bytes<16>,
    /// 0x4F7E28: Not named: 16 bytes (a gap, or data nothing reads here).
    pub _gap_4f7e28: Bytes<16>,
    /// 0x4F7E38: `GUID_XAxis` .. `GUID_RzAxis`: the six axes' GUIDs, 16 bytes each (`input::axis_found`).
    pub axes: Bytes<96>,
    /// 0x4F7E98: Not named: 128 bytes (a gap, or data nothing reads here).
    pub _gap_4f7e98: Bytes<128>,
    /// 0x4F7F18: `GUID_ConstantForce`: an effect.
    pub constant_force: Bytes<16>,
    /// 0x4F7F28: `GUID_RampForce`: an effect.
    pub ramp_force: Bytes<16>,
    /// 0x4F7F38: `GUID_Square`: an effect.
    pub square: Bytes<16>,
    /// 0x4F7F48: `GUID_Sine`: an effect.
    pub sine: Bytes<16>,
    /// 0x4F7F58: `GUID_Triangle`: an effect.
    pub triangle: Bytes<16>,
    /// 0x4F7F68: `GUID_SawtoothUp`: an effect.
    pub sawtooth_up: Bytes<16>,
    /// 0x4F7F78: `GUID_SawtoothDown`: an effect.
    pub sawtooth_down: Bytes<16>,
    /// 0x4F7F88: `GUID_Spring`: an effect.
    pub spring: Bytes<16>,
    /// 0x4F7F98: `GUID_Damper`: an effect.
    pub damper: Bytes<16>,
    /// 0x4F7FA8: `GUID_Inertia`: an effect.
    pub inertia: Bytes<16>,
    /// 0x4F7FB8: `GUID_Friction`: an effect.
    pub friction: Bytes<16>,
    /// 0x4F7FC8: Not named: 16 bytes (a gap, or data nothing reads here).
    pub _gap_4f7fc8: Bytes<16>,
}

pub static INPUT_GUIDS: InputGuids = InputGuids {
    _lead: Bytes::new([0; 8]),
    device2: Bytes::new(with_runs::<16>(&[(0, b"\x82\xe6DY.\xc9\xcf\x11\xbf\xc7DEST")])),
    _gap_4f7e28: Bytes::new(with_runs::<16>(&[(0, b"\xc0\xf7\xe1\xe7\xd2\x88\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    axes: Bytes::new(with_runs::<96>(&[(0, b"\xe0\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xe1\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xe2\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xf4\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xf5\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xe3\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST")])),
    _gap_4f7e98: Bytes::new(with_runs::<128>(&[(0, b"\xe4\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xf0\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00 \x82rU<\xd3\xcf\x11\xbf\xc7DEST\x00\x00\xf2\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xf3\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00`+\x1do\xa0\xd5\xcf\x11\xbf\xc7DEST\x00\x00a+\x1do\xa0\xd5\xcf\x11\xbf\xc7DEST\x00\x00p+\x1do\xa0\xd5\xcf\x11\xbf\xc7DEST")])),
    constant_force: Bytes::new(with_runs::<16>(&[(0, b" \x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    ramp_force: Bytes::new(with_runs::<16>(&[(0, b"!\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    square: Bytes::new(with_runs::<16>(&[(0, b"\"\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    sine: Bytes::new(with_runs::<16>(&[(0, b"#\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    triangle: Bytes::new(with_runs::<16>(&[(0, b"$\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    sawtooth_up: Bytes::new(with_runs::<16>(&[(0, b"%\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    sawtooth_down: Bytes::new(with_runs::<16>(&[(0, b"&\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    spring: Bytes::new(with_runs::<16>(&[(0, b"'\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    damper: Bytes::new(with_runs::<16>(&[(0, b"(\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    inertia: Bytes::new(with_runs::<16>(&[(0, b")\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    friction: Bytes::new(with_runs::<16>(&[(0, b"*\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
    _gap_4f7fc8: Bytes::new(with_runs::<16>(&[(0, b"+\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")])),
};

impl InputGuids {
    pub const START: usize = 0x4F7E18;
    pub const LEN: usize = 0x1C0;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(&INPUT_GUIDS as *const InputGuids) + 8) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inputguids_starts_as_the_games() {
        let size = size_of::<InputGuids>();
        assert!(8 + InputGuids::LEN <= size && size <= (8 + InputGuids::LEN).next_multiple_of(16));
        let image = crate::moved::image_bytes(InputGuids::START, InputGuids::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(InputGuids::home()) as *const u8,
                InputGuids::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
