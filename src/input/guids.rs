//! The DirectInput identifiers the input module names (the joystick device's interface, the
//! axes and the force-feedback effects). The effect
//! templates (`input::store::Devices`) name the effects' GUIDs by handle (`EffectGuid`).

use crate::memory::session::with_runs;

/// The DirectInput GUIDs, constants nothing writes.
pub struct InputGuids {
    /// `IID_IDirectInputDevice2A`: the joystick device's interface (`input::acquire_device`,
    /// `joystick_found`).
    pub device2: [u8; 16],
    /// `GUID_XAxis` .. `GUID_RzAxis`: the six axes' GUIDs (`input::axis_found`).
    pub axes: [[u8; 16]; 6],
    pub constant_force: [u8; 16],
    pub ramp_force: [u8; 16],
    pub square: [u8; 16],
    pub sine: [u8; 16],
    pub triangle: [u8; 16],
    pub sawtooth_up: [u8; 16],
    pub sawtooth_down: [u8; 16],
    pub spring: [u8; 16],
    pub damper: [u8; 16],
    pub inertia: [u8; 16],
    pub friction: [u8; 16],
}

impl InputGuids {
    pub const fn new() -> Self {
        let axes = with_runs::<96>(&[(0, b"\xe0\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xe1\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xe2\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xf4\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xf5\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST\x00\x00\xe3\x02m\xa3\xf3\xc9\xcf\x11\xbf\xc7DEST")]);
        let mut grouped = [[0u8; 16]; 6];
        let mut n = 0;
        while n < 6 {
            let mut i = 0;
            while i < 16 {
                grouped[n][i] = axes[16 * n + i];
                i += 1;
            }
            n += 1;
        }
        InputGuids {
            device2: with_runs::<16>(&[(0, b"\x82\xe6DY.\xc9\xcf\x11\xbf\xc7DEST")]),
            axes: grouped,
            constant_force: with_runs::<16>(&[(
                0,
                b" \x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5",
            )]),
            ramp_force: with_runs::<16>(&[(
                0,
                b"!\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5",
            )]),
            square: with_runs::<16>(&[(0, b"\"\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")]),
            sine: with_runs::<16>(&[(0, b"#\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")]),
            triangle: with_runs::<16>(&[(0, b"$\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")]),
            sawtooth_up: with_runs::<16>(&[(
                0,
                b"%\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5",
            )]),
            sawtooth_down: with_runs::<16>(&[(
                0,
                b"&\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5",
            )]),
            spring: with_runs::<16>(&[(0, b"'\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")]),
            damper: with_runs::<16>(&[(0, b"(\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")]),
            inertia: with_runs::<16>(&[(0, b")\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")]),
            friction: with_runs::<16>(&[(0, b"*\x1cT\x133\x8e\xd0\x11\x9a\xd0\x00\xa0\xc9\xa0n5")]),
        }
    }
}

/// A GUID of these by handle: `GUID_TAG | n`, `n` the effect's number in `EffectGuid::ALL`.
const GUID_TAG: u32 = 0xCD00_0000;

/// One of the eleven force-feedback effect GUIDs (the image's `GUID_ConstantForce` and so on).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EffectGuid {
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
    fn from_handle(handle: u32) -> Option<Self> {
        let n = handle.checked_sub(GUID_TAG)?;
        Self::ALL.get(n as usize).copied()
    }

    fn field(self, guids: &InputGuids) -> &[u8; 16] {
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
pub(super) fn guid_of(
    guids: &InputGuids,
    handle: u32,
) -> Result<[u8; 16], crate::errors::BadInput> {
    let effect = EffectGuid::from_handle(handle).ok_or(crate::errors::BadInput(
        "an effect's GUID handle that names none",
    ))?;
    Ok(*effect.field(guids))
}
