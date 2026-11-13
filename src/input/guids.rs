//! The DirectInput identifiers the input module names (the joystick device's interface, the
//! axes and the force-feedback effects), as a `#[repr(C)]` struct of byte buffers. The effect
//! templates (`input::store::Devices`) name the effects' GUIDs by handle (`EffectGuid`).

use crate::session::{Bytes, with_runs};

/// The DirectInput GUIDs.
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct InputGuids {
    /// Pads the struct so every field keeps the alignment it has in the game's layout (the struct
    /// is 16-byte aligned).
    pub _lead: Bytes<8>,
    /// `IID_IDirectInputDevice2A`: the joystick device's interface (`input::acquire_device`,
    /// `joystick_found`).
    pub device2: Bytes<16>,
    pub _gap_4f7e28: Bytes<16>,
    /// `GUID_XAxis` .. `GUID_RzAxis`: the six axes' GUIDs, 16 bytes each (`input::axis_found`).
    pub axes: Bytes<96>,
    pub _gap_4f7e98: Bytes<128>,
    pub constant_force: Bytes<16>,
    pub ramp_force: Bytes<16>,
    pub square: Bytes<16>,
    pub sine: Bytes<16>,
    pub triangle: Bytes<16>,
    pub sawtooth_up: Bytes<16>,
    pub sawtooth_down: Bytes<16>,
    pub spring: Bytes<16>,
    pub damper: Bytes<16>,
    pub inertia: Bytes<16>,
    pub friction: Bytes<16>,
    pub _gap_4f7fc8: Bytes<16>,
    /// Up to the record's alignment (no byte of a record of cells outside a field).
    pub _tail: Bytes<8>,
}

impl InputGuids {
    pub const fn new() -> Self {
        InputGuids {
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
    _tail: Bytes::new([0; 8]),
        }
    }
}

/// A GUID of these by handle: `GUID_TAG | n`, `n` the effect's number in `EffectGuid::ALL`.
pub const GUID_TAG: u32 = 0xCD00_0000;

/// One of the eleven force-feedback effect GUIDs (the image's `GUID_ConstantForce` and so on).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectGuid {
    ConstantForce,
    RampForce,
    Square,
    Sine,
    Triangle,
    SawtoothUp,
    SawtoothDown,
    Spring,
    Damper,
    Inertia,
    Friction,
}

impl EffectGuid {
    pub const ALL: [EffectGuid; 11] = [
        EffectGuid::ConstantForce,
        EffectGuid::RampForce,
        EffectGuid::Square,
        EffectGuid::Sine,
        EffectGuid::Triangle,
        EffectGuid::SawtoothUp,
        EffectGuid::SawtoothDown,
        EffectGuid::Spring,
        EffectGuid::Damper,
        EffectGuid::Inertia,
        EffectGuid::Friction,
    ];

    /// The handle the device records hold.
    pub const fn handle(self) -> u32 {
        GUID_TAG | self as u32
    }

    /// The GUID a handle names, if it is one.
    pub fn from_handle(handle: u32) -> Option<Self> {
        let n = handle.checked_sub(GUID_TAG)?;
        Self::ALL.get(n as usize).copied()
    }

    fn field(self, guids: &InputGuids) -> &Bytes<16> {
        match self {
            EffectGuid::ConstantForce => &guids.constant_force,
            EffectGuid::RampForce => &guids.ramp_force,
            EffectGuid::Square => &guids.square,
            EffectGuid::Sine => &guids.sine,
            EffectGuid::Triangle => &guids.triangle,
            EffectGuid::SawtoothUp => &guids.sawtooth_up,
            EffectGuid::SawtoothDown => &guids.sawtooth_down,
            EffectGuid::Spring => &guids.spring,
            EffectGuid::Damper => &guids.damper,
            EffectGuid::Inertia => &guids.inertia,
            EffectGuid::Friction => &guids.friction,
        }
    }
}

/// The effect GUID a handle names, read from the field that holds it. The handles are the
/// device records' `wanted` dwords, which nothing writes: `GUID_Spring`, `GUID_Sine`,
/// `GUID_ConstantForce` and `GUID_Damper`, so every read is one of the eleven effect GUIDs,
/// a field of its own. Refused: what is no handle and a handle that names no effect GUID.
pub fn guid_of(guids: &InputGuids, handle: u32) -> Result<[u8; 16], crate::errors::BadInput> {
    let effect = EffectGuid::from_handle(handle).ok_or(crate::errors::BadInput(
        "an effect's GUID handle that names none",
    ))?;
    let field = effect.field(guids);
    Ok(std::array::from_fn(|i| field.get(i)))
}
