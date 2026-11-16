//! A joystick as the input thread reads it.
//!
//! DirectInput's state of a joystick (`DIJOYSTATE`: six axes, two sliders, four hats as dwords,
//! then 32 buttons as bytes) is read into a device record's three axes and its buttons' bits;
//! the record's selector bytes say which state dwords make the axes (`& 7` the dword, bit 0x10
//! the sign or a half's offset).

use crate::input::force::Guid;

/// The six axes' bits in a controller's entry, in the order of their GUIDs.
pub const AXES: u32 = 6;

/// `input::axis_found`: the bits a controller's axes have once an object of type
/// `object_type` is listed: its axis's bit added when it is one of the six.
pub fn axis_bits(bits: u32, object_type: &Guid, axes: &[Guid; AXES as usize]) -> u32 {
    axes.iter()
        .enumerate()
        .filter(|(_, axis)| *axis == object_type)
        .fold(bits, |bits, (n, _)| bits | 1 << n)
}

/// The most controllers the list holds.
pub const CONTROLLERS: u32 = 16;

/// What a found joystick's entry gets from its capabilities.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Found {
    pub caps_flags: u32,
    pub axes: u32,
    pub buttons: u32,
    pub hats: u32,
    /// 1 when attached.
    pub attached: u32,
    /// 1 with force feedback (`DIDC_FORCEFEEDBACK`); not written when the configuration
    /// turned force feedback off.
    pub force_feedback: Option<u32>,
}

/// The entry from the capabilities' flags, axes, buttons and hats; `no_force` (not 0) turns
/// force feedback off.
pub fn found(flags: u32, axes: u32, buttons: u32, hats: u32, no_force: u32) -> Found {
    Found {
        caps_flags: flags,
        axes,
        buttons,
        hats,
        attached: u32::from(flags & 1 != 0),
        force_feedback: (no_force == 0).then_some(u32::from(flags & 0x100 != 0)),
    }
}

/// The entry's name before the device's own: "(n) ", n the count of controllers listed
/// before it plus 1.
pub fn name_prefix(count: u32) -> Vec<u8> {
    use crate::strings::format::{Pad, decimal};
    let mut name = vec![b'('];
    name.extend(decimal(count.wrapping_add(1), Pad::None));
    name.extend(b") ");
    name
}

/// The room the device's own name has in the entry.
pub const NAME_ROOM: u32 = 0x19;

/// The selectors' places in a device record: the second and third axes' (kinds above 2),
/// the y and the x axes' (kinds up to 2).
pub const SELECT_SECOND: u32 = 0x1A;
pub const SELECT_THIRD: u32 = 0x1B;
pub const SELECT_Y: u32 = 0x1C;
pub const SELECT_X: u32 = 0x1D;

/// The absolute value (wrapping).
fn positive(v: u32) -> u32 {
    if (v as i32) < 0 { v.wrapping_neg() } else { v }
}

/// A record's three axes for a `kind` from the state's dwords (`state`) by its selector bytes
/// (`selector`, by place), each asked for only when used. Kinds up to 2: x and y the selected
/// dwords (negated with bit 0x10), a y below 0 being the third axis (made positive) and the second
/// 0; above 2: x the first dword, the others the selected (plus 0x100 with bit 0x10). The second
/// and third are made positive.
pub fn poll_axes(kind: u32, selector: impl Fn(u32) -> u8, state: impl Fn(u32) -> u32) -> [u32; 3] {
    let selected = |place: u32| {
        let b = selector(place);
        (state(u32::from(b & 7)), b & 0x10 != 0)
    };
    let (x, mut y, mut z);
    if kind <= 2 {
        let (v, n) = selected(SELECT_X);
        x = if n { v.wrapping_neg() } else { v };
        let (v, n) = selected(SELECT_Y);
        y = if n { v.wrapping_neg() } else { v };
        z = 0;
        if (y as i32) < 0 {
            z = 0u32.wrapping_sub(y);
            y = 0;
        }
    } else {
        x = state(0);
        let (v, n) = selected(SELECT_SECOND);
        y = if n { v.wrapping_add(0x100) } else { v };
        let (v, n) = selected(SELECT_THIRD);
        z = if n { v.wrapping_add(0x100) } else { v };
    }
    [x, positive(y), positive(z)]
}

/// The 32 buttons' bits: a button is down with its byte's bit 7.
pub fn poll_buttons(buttons: &[u8; 32]) -> u32 {
    buttons
        .iter()
        .enumerate()
        .filter(|(_, b)| *b & 0x80 != 0)
        .fold(0, |bits, (k, _)| bits | 1 << k)
}

/// What `input::request_device_setup` gives the first device record (the input thread then
/// gives the device back and acquires it again with these).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceSetup {
    pub controller: u32,
    pub sensitivity: u32,
    /// The kind: the sensitivity, 2 for 0.
    pub kind: u32,
    pub axis_code: u32,
    /// No force feedback asked.
    pub strength: u32,
    pub dead_zone: u32,
    pub end_zone: u32,
}

pub fn device_setup(
    controller: u32,
    sensitivity: u32,
    axis_code: u32,
    dead_zone: u32,
    end_zone: u32,
) -> DeviceSetup {
    DeviceSetup {
        controller,
        sensitivity,
        kind: if sensitivity == 0 { 2 } else { sensitivity },
        axis_code,
        strength: 0,
        dead_zone,
        end_zone,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axes_listed() {
        let axes: [Guid; 6] = std::array::from_fn(|n| [n as u8 + 1; 16]);
        assert_eq!(axis_bits(1, &[3; 16], &axes), 5);
        assert_eq!(axis_bits(1, &[9; 16], &axes), 1);
    }

    #[test]
    fn polling() {
        let state = |n: u32| [5u32, 0xFFFF_FFF0, 7, 0x80, 0, 0, 0, 0][n as usize];
        // kinds up to 2: x the third dword negated, y the second (negative: the third axis)
        let sel = |p: u32| match p {
            SELECT_X => 0x12,
            SELECT_Y => 1,
            _ => unreachable!(),
        };
        assert_eq!(poll_axes(2, sel, state), [(-7i32) as u32, 0, 0x10]);
        let sel = |p: u32| match p {
            SELECT_SECOND => 0x13,
            SELECT_THIRD => 1,
            _ => unreachable!(),
        };
        assert_eq!(poll_axes(3, sel, state), [5, 0x180, 0x10]);
        let mut b = [0u8; 32];
        b[0] = 0x80;
        b[31] = 0xFF;
        b[2] = 0x7F;
        assert_eq!(poll_buttons(&b), 0x8000_0001);
    }

    #[test]
    fn setup() {
        assert_eq!(device_setup(1, 0, 2, 3, 4).kind, 2);
        assert_eq!(device_setup(1, 3, 2, 3, 4).kind, 3);
    }

    #[test]
    fn found_entries() {
        assert_eq!(
            found(0x101, 3, 8, 1, 0),
            Found {
                caps_flags: 0x101,
                axes: 3,
                buttons: 8,
                hats: 1,
                attached: 1,
                force_feedback: Some(1),
            }
        );
        assert_eq!(found(0x100, 0, 0, 0, 1).force_feedback, None);
        assert_eq!(found(0x100, 0, 0, 0, 1).attached, 0);
        assert_eq!(name_prefix(0), b"(1) ");
        assert_eq!(name_prefix(11), b"(12) ");
    }
}
