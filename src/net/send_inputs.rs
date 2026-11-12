//! `net::send_inputs` (0x464194): "send j", the player and the step logged, and the
//! message 'JGSM' (0x4D53474A) with the car's four input dwords (the current car's block
//! +0xFD0 .. +0xFDC) built in `net_message` and sent (`net::send_race_message`, which is
//! `call net::send_message; retn`: `net::send_message` here).
//!
//!
//! See `net/mod.rs`; `send_position` and `send_extra` are the same with other dwords (this
//! file has their shared body). One caller, `net::send_turn` (every fourth step).

use crate::errors::{BadInput, Refused};
use crate::net::log_send;
use crate::net::owner::Net;
use crate::net::send_message::send_message;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::session::Cell32;
use rc2k_core::net::race::Limit::{Free, Gauge, Gear, Large, Pedal};
use rc2k_core::net::race::{Kind, Limit};

/// What one of the three sends: its log text  and its kind (the
/// tag and the car block's dwords: `rc2k_core::net::race::Kind`).
pub struct Message {
    pub text: &'static std::ffi::CStr,
    pub kind: Kind,
}

pub const INPUTS: Message = Message {
    text: c"send j", // "send j"
    kind: Kind::Inputs,
};

/// A car block's field a message carries, as a dword's bits, and the limit of what a car
/// could hold in it.
pub type Field = (fn(&Car) -> &dyn Cell32, Limit);

/// The car block's dwords each kind carries, in the message's order.
pub fn fields(kind: Kind) -> &'static [Field] {
    match kind {
        Kind::Inputs => &INPUT_FIELDS,
        Kind::Position => &POSITION_FIELDS,
        Kind::Extra => &EXTRA_FIELDS,
    }
}

/// +0xFD0 .. +0xFDC.
const INPUT_FIELDS: [Field; 4] = [
    (|c| &c.kept_throttle_word, Free),
    (|c| &c.kept_keys_word, Free),
    (|c| &c.kept_steering_word, Free),
    (|c| &c.kept_brake_word, Free),
];

/// +0x134 .. +0x13C (the place, put where the car is to be on arrival), the matrix's first
/// twelve, the spin, the velocities, and others.
const POSITION_FIELDS: [Field; 40] = [
    (|c| &c.position[0], Free),
    (|c| &c.position[1], Free),
    (|c| &c.position[2], Free),
    (|c| &c.crash_matrix[0], Gauge),
    (|c| &c.crash_matrix[1], Gauge),
    (|c| &c.crash_matrix[2], Gauge),
    (|c| &c.crash_matrix[3], Gauge),
    (|c| &c.crash_matrix[4], Gauge),
    (|c| &c.crash_matrix[5], Gauge),
    (|c| &c.crash_matrix[6], Gauge),
    (|c| &c.crash_matrix[7], Gauge),
    (|c| &c.crash_matrix[8], Gauge),
    (|c| &c.crash_matrix[9], Gauge),
    (|c| &c.crash_matrix[10], Gauge),
    (|c| &c.crash_matrix[11], Gauge),
    (|c| &c.velocity[0], Large),
    (|c| &c.velocity[1], Large),
    (|c| &c.velocity[2], Large),
    (|c| &c.local_velocity[0], Large),
    (|c| &c.local_velocity[1], Large),
    (|c| &c.local_velocity[2], Large),
    (|c| &c.spin[0], Large),
    (|c| &c.spin[1], Large),
    (|c| &c.spin[2], Large),
    (|c| &c.checkpoint, Free),
    (|c| &c.reset_checkpoint, Free),
    (|c| &c.gear, Gear),
    (|c| &c.engine_speed, Large),
    (|c| &c.drive_speed_a, Large),
    (|c| &c.drive_speed_b, Large),
    (|c| &c.wheel_squeeze[0], Gauge),
    (|c| &c.wheel_squeeze[1], Gauge),
    (|c| &c.wheel_squeeze[2], Gauge),
    (|c| &c.wheel_squeeze[3], Gauge),
    (|c| &c.throttle, Pedal),
    (|c| &c.brake, Pedal),
    (|c| &c.handbrake, Pedal),
    (|c| &c.steering, Pedal),
    (|c| &c.steering_raw, Pedal),
    (|c| &c.position_message_word, Free),
];

/// +0x64 .. +0x84: the eight gauges and the lights.
const EXTRA_FIELDS: [Field; 9] = [
    (|c| &c.impact_octant_marks[0], Gauge),
    (|c| &c.impact_octant_marks[1], Gauge),
    (|c| &c.impact_octant_marks[2], Gauge),
    (|c| &c.impact_octant_marks[3], Gauge),
    (|c| &c.impact_octant_marks[4], Gauge),
    (|c| &c.impact_octant_marks[5], Gauge),
    (|c| &c.impact_octant_marks[6], Gauge),
    (|c| &c.impact_octant_marks[7], Gauge),
    (|c| &c.lights_on, Free),
];

/// The shared body: the car block `g_car`, the message built at `RaceWork::net_message`.
pub fn send_fields(race: &Race, net: &Net, m: &Message) -> Result<(), Refused> {
    let shared = &race.shared;
    let car = shared.car();
    let message = &shared.work.get().net_message;
    log_send(race, &net.vars, m.text);
    // The tag, then the kind's fields' bits.
    let mut dwords = vec![m.kind.tag()];
    dwords.extend(fields(m.kind).iter().map(|(field, _)| field(car).bits()));
    let cells = message
        .get(..dwords.len())
        .ok_or(BadInput("a network message past its buffer"))?;
    for (cell, &v) in cells.iter().zip(&dwords) {
        cell.set(v);
    }
    let bytes: Vec<u8> = dwords.iter().flat_map(|v| v.to_le_bytes()).collect();
    send_message(net, &bytes)
}
pub fn send_inputs(race: &Race, net: &Net) -> Result<(), Refused> {
    send_fields(race, net, &INPUTS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::race::owner::race;

    /// Each field sits where the core's table had its offset in the car's block.
    #[test]
    fn the_fields_sit_where_the_offsets_were() {
        let car = race().shared.car();
        let base = core::ptr::from_ref(car).addr();
        let offsets = |kind| -> Vec<u32> {
            fields(kind)
                .iter()
                .map(|(field, _)| (core::ptr::from_ref(field(car)).addr() - base) as u32)
                .collect()
        };
        assert_eq!(offsets(Kind::Inputs), [0xFD0, 0xFD4, 0xFD8, 0xFDC]);
        assert_eq!(
            offsets(Kind::Position),
            [
                0x134, 0x138, 0x13C, 0xB4, 0xB8, 0xBC, 0xC0, 0xC4, 0xC8, 0xCC, 0xD0, 0xD4, 0xD8,
                0xDC, 0xE0, 0x198, 0x19C, 0x1A0, 0x1A4, 0x1A8, 0x1AC, 0x184, 0x188, 0x18C, 0x98,
                0x94, 0x1D4, 0x1D8, 0x1B8, 0x1BC, 0x600, 0x604, 0x608, 0x60C, 0x638, 0x63C, 0x640,
                0x644, 0x648, 0x1EC,
            ]
        );
        assert_eq!(
            offsets(Kind::Extra),
            [0x64, 0x68, 0x6C, 0x70, 0x74, 0x78, 0x7C, 0x80, 0x84]
        );
    }

    /// The limits are the ranges the core's table matched by offset.
    #[test]
    fn the_limits_are_the_old_ranges() {
        let car = race().shared.car();
        let base = core::ptr::from_ref(car).addr();
        for kind in [Kind::Inputs, Kind::Position, Kind::Extra] {
            for (field, limit) in fields(kind) {
                let at = (core::ptr::from_ref(field(car)).addr() - base) as u32;
                let old = match at {
                    0x1D4 => Limit::Gear,
                    0x64..=0x80 | 0xB4..=0xE0 | 0x600..=0x60C => Limit::Gauge,
                    0x184..=0x18C | 0x198..=0x1AC | 0x1B8 | 0x1BC | 0x1D8 => Limit::Large,
                    0x638..=0x648 => Limit::Pedal,
                    _ => Limit::Free,
                };
                assert_eq!(*limit, old, "{at:#x}");
            }
        }
    }
}
