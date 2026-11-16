//! Copies an acquired device's state into its record: `Poll`, then `GetDeviceState`; when that
//! fails, `Acquire` and the record's first 0x50 bytes are copied over the state, and the rest then
//! works on the record 0x50 bytes further on (a slip kept as is: zeros were meant). Then the three
//! axes by the record's kind (x, y, and a third from a negative y or the state's other dwords, made
//! positive) go into +0x2C, +0x30, +0x34 and the 32 buttons (state +0x30 .., bit 7) into +0x38 as
//! bits. Record layout: `input/devices.rs`.

use crate::errors::Refused;
use crate::input::devices::{STATE_LEN, joy_state};
use crate::input::store::Devices;
use crate::input::types::InputDevice;
use rc2k_core::input::joystick::{SELECT_SECOND, SELECT_THIRD, SELECT_X, SELECT_Y};

fn state(devices: &Devices, n: u32) -> u32 {
    devices.device_state[n as usize].get()
}

/// The record's byte at one of the four places of the axis codes (+0x1A .. +0x1D).
fn selector(record: &InputDevice, place: u32) -> u8 {
    match place {
        SELECT_SECOND => record.axis_code_byte(2),
        SELECT_THIRD => record.axis_code_byte(3),
        SELECT_Y => record.axis_pick_byte(0),
        SELECT_X => record.axis_pick_byte(1),
        _ => unreachable!("the core asks for the four places of the axis codes"),
    }
}

/// The device `record` polled (see the header).
pub fn poll_device(devices: &Devices, record: &InputDevice) -> Result<(), Refused> {
    let device = record.device.get();
    if record.acquired.get() != 1 || device == 0 {
        return Ok(());
    }
    crate::input::di_objects::poll(device);
    let failed = match crate::input::di_objects::get_device_state(device, STATE_LEN as u32) {
        Ok(state) => {
            for (cell, v) in devices.device_state.iter().zip(state.dwords()) {
                cell.set(v);
            }
            false
        }
        Err(_) => true,
    };
    // The copy of a failed read moves its pointer on by 0x50 as well: what follows reads
    // and writes the record from there (sic) to the function's end.
    if failed {
        crate::input::di_objects::acquire(device);
        // The record's first 0x50 bytes are its fields up to the first effect's GUID.
        let bytes = record.to_bytes();
        let (dwords, _) = bytes.as_chunks::<4>();
        for (cell, dword) in devices.device_state.iter().zip(dwords) {
            cell.set(u32::from_le_bytes(*dword));
        }
        poll_shifted(devices, record);
        return Ok(());
    }
    use rc2k_core::input::joystick::{poll_axes, poll_buttons};
    let [x, y, z] = poll_axes(
        record.kind.get(),
        |place| selector(record, place),
        |n| state(devices, n),
    );
    record.axes[0].set(x);
    record.axes[1].set(y);
    record.axes[2].set(z);
    record.buttons.set(poll_buttons(
        &joy_state(devices)[0x30..].try_into().unwrap(),
    ));
    Ok(())
}

/// The rest of the poll after a failed read, on the record 0x50 bytes on (sic: the first
/// effect's fields stand where the axes were meant): the kind is the dword at +0x58, the
/// third and fourth bytes of the effect's `pending` and the first two of its `object` are the
/// four axis selectors (+0x6A .. +0x6D), and the axes and buttons go to +0x7C .. +0x88, the
/// effect's `sample_period`, `gain`, `trigger_button` and `trigger_repeat_interval`.
fn poll_shifted(devices: &Devices, record: &InputDevice) {
    use rc2k_core::input::joystick::{poll_axes, poll_buttons};
    let effect = &record.effects[0];
    let kind = effect.guid.get_u32(0xC);
    let [x, y, z] = poll_axes(
        kind,
        |place| match place {
            SELECT_SECOND => effect.pending.get().to_le_bytes()[2],
            SELECT_THIRD => effect.pending.get().to_le_bytes()[3],
            SELECT_Y => effect.object.get().to_le_bytes()[0],
            SELECT_X => effect.object.get().to_le_bytes()[1],
            _ => unreachable!("the core asks for the four places of the axis codes"),
        },
        |n| state(devices, n),
    );
    effect.effect.sample_period.set(x);
    effect.effect.gain.set(y);
    effect.effect.trigger_button.set(z);
    effect.effect.trigger_repeat_interval.set(poll_buttons(
        &joy_state(devices)[0x30..].try_into().unwrap(),
    ));
}
