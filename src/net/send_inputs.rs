//! Logs "send j" and sends the 'JGSM' message with the car's four input dwords; `send_position`
//! and `send_extra` are the same with other dwords, sharing this body. Called every fourth
//! step.

use crate::errors::{BadInput, Refused};
use crate::memory::plain_place::WordMut;
use crate::net::log_send;
use crate::net::owner::Net;
use crate::net::send_message::send_payload;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use rc2k_core::net::message::Payload;
use rc2k_core::net::race::Limit::{Free, Gauge, Gear, Large, Pedal};
use rc2k_core::net::race::{Kind, Limit};

/// What one of the three sends: its log text  and its kind (the
/// tag and the car block's dwords: `rc2k_core::net::race::Kind`).
pub struct Message {
    pub text: &'static std::ffi::CStr,
    pub kind: Kind,
}

const INPUTS: Message = Message {
    text: c"send j", // "send j"
    kind: Kind::Inputs,
};

/// A car block's field a message carries, as a dword's bits, and the limit of what a car
/// could hold in it.
pub type Field = (for<'a> fn(&'a mut Car) -> WordMut<'a>, Limit);

/// The car block's dwords each kind carries, in the message's order.
pub fn fields(kind: Kind) -> &'static [Field] {
    match kind {
        Kind::Inputs => &INPUT_FIELDS,
        Kind::Position => &POSITION_FIELDS,
        Kind::Extra => &EXTRA_FIELDS,
    }
}

/// The kept throttle, key bits, steering and brake words.
const INPUT_FIELDS: [Field; 4] = [
    (|c| (&mut c.kept_throttle_word).into(), Free),
    (|c| (&mut c.kept_keys_word).into(), Free),
    (|c| (&mut c.kept_steering_word).into(), Free),
    (|c| (&mut c.kept_brake_word).into(), Free),
];

/// The place (put where the car is to be on arrival), the matrix's first
/// twelve, the spin, the velocities, and others.
const POSITION_FIELDS: [Field; 40] = [
    (|c| (&mut c.position[0]).into(), Free),
    (|c| (&mut c.position[1]).into(), Free),
    (|c| (&mut c.position[2]).into(), Free),
    (|c| (&mut c.crash_matrix[0]).into(), Gauge),
    (|c| (&mut c.crash_matrix[1]).into(), Gauge),
    (|c| (&mut c.crash_matrix[2]).into(), Gauge),
    (|c| (&mut c.crash_matrix[3]).into(), Gauge),
    (|c| (&mut c.crash_matrix[4]).into(), Gauge),
    (|c| (&mut c.crash_matrix[5]).into(), Gauge),
    (|c| (&mut c.crash_matrix[6]).into(), Gauge),
    (|c| (&mut c.crash_matrix[7]).into(), Gauge),
    (|c| (&mut c.crash_matrix[8]).into(), Gauge),
    (|c| (&mut c.crash_matrix[9]).into(), Gauge),
    (|c| (&mut c.crash_matrix[10]).into(), Gauge),
    (|c| (&mut c.crash_matrix[11]).into(), Gauge),
    (|c| (&mut c.velocity[0]).into(), Large),
    (|c| (&mut c.velocity[1]).into(), Large),
    (|c| (&mut c.velocity[2]).into(), Large),
    (|c| (&mut c.local_velocity[0]).into(), Large),
    (|c| (&mut c.local_velocity[1]).into(), Large),
    (|c| (&mut c.local_velocity[2]).into(), Large),
    (|c| (&mut c.spin[0]).into(), Large),
    (|c| (&mut c.spin[1]).into(), Large),
    (|c| (&mut c.spin[2]).into(), Large),
    (|c| (&mut c.checkpoint).into(), Free),
    (|c| (&mut c.reset_checkpoint).into(), Free),
    (|c| (&mut c.gear).into(), Gear),
    (|c| (&mut c.engine_speed).into(), Large),
    (|c| (&mut c.drive_speed_a).into(), Large),
    (|c| (&mut c.drive_speed_b).into(), Large),
    (|c| (&mut c.wheel_squeeze[0]).into(), Gauge),
    (|c| (&mut c.wheel_squeeze[1]).into(), Gauge),
    (|c| (&mut c.wheel_squeeze[2]).into(), Gauge),
    (|c| (&mut c.wheel_squeeze[3]).into(), Gauge),
    (|c| (&mut c.throttle).into(), Pedal),
    (|c| (&mut c.brake).into(), Pedal),
    (|c| (&mut c.handbrake).into(), Pedal),
    (|c| (&mut c.steering).into(), Pedal),
    (|c| (&mut c.steering_raw).into(), Pedal),
    (|c| (&mut c.position_message_word).into(), Free),
];

/// The eight gauges and the lights.
const EXTRA_FIELDS: [Field; 9] = [
    (|c| (&mut c.impact_octant_marks[0]).into(), Gauge),
    (|c| (&mut c.impact_octant_marks[1]).into(), Gauge),
    (|c| (&mut c.impact_octant_marks[2]).into(), Gauge),
    (|c| (&mut c.impact_octant_marks[3]).into(), Gauge),
    (|c| (&mut c.impact_octant_marks[4]).into(), Gauge),
    (|c| (&mut c.impact_octant_marks[5]).into(), Gauge),
    (|c| (&mut c.impact_octant_marks[6]).into(), Gauge),
    (|c| (&mut c.impact_octant_marks[7]).into(), Gauge),
    (|c| (&mut c.lights_on).into(), Free),
];

/// The shared body: the car block, the message built at `RaceWork::net_message`.
pub(super) fn send_fields(race: &Race, net: &Net, m: &Message) -> Result<(), Refused> {
    let shared = &race.shared;
    let mut car = shared.car_mut();
    log_send(race, net, m.text);
    let values: Vec<u32> = fields(m.kind)
        .iter()
        .map(|(field, _)| field(&mut car).bits())
        .collect();
    // The work area keeps a copy of the message: the tag, then the kind's fields' bits.
    {
        let mut work = shared.work_mut();
        let cells = work
            .net_message
            .get_mut(..1 + values.len())
            .ok_or(BadInput("a network message past its buffer"))?;
        for (cell, &v) in cells
            .iter_mut()
            .zip(std::iter::once(&m.kind.tag()).chain(&values))
        {
            *cell = v;
        }
    }
    send_payload(net, Payload::of(m.kind, values))
}
pub fn send_inputs(race: &Race, net: &Net) -> Result<(), Refused> {
    send_fields(race, net, &INPUTS)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each field sits where the core's table had its position in the car's block.
    #[test]
    fn the_fields_sit_where_the_offsets_were() {
        let offsets = |kind| -> Vec<u32> {
            fields(kind)
                .iter()
                .map(|(field, _)| {
                    crate::physics::car_offsets_1999::offset_changed_by(|c| field(c).set_bits(1))
                        as u32
                })
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

    /// The limits are the ranges the core's table matched by position.
    #[test]
    fn the_limits_are_the_old_ranges() {
        for kind in [Kind::Inputs, Kind::Position, Kind::Extra] {
            for (field, limit) in fields(kind) {
                let at =
                    crate::physics::car_offsets_1999::offset_changed_by(|c| field(c).set_bits(1))
                        as u32;
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
