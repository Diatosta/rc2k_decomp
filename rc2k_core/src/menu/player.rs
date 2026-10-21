//! A player, 0x28C bytes (`Player` in IDA; converted 2026-10-21 with the players' details
//! pages' settings, `menu::next_nationality` 0x467AFD .. `menu::flip_codriver` 0x467BD6,
//! `menu::next_car_number` 0x470D7F / `previous_car_number` 0x470DA1,
//! `menu::raise_sensitivity` 0x470F05 / `lower_sensitivity` 0x470F2D and the nine visual
//! preferences' toggles 0x467DE2 .. 0x467E92): the player's settings (name, nationality,
//! views, co-driver, keys, controller) and the state of the player's car (which car,
//! damage, setup, wear). What is known of it; the rest are gaps.

/// The players.
pub const PLAYERS: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub gap_0: u32,
    /// The player's nationality (0 Great Britain, 1 Argentina, ..; `dword_758816` has
    /// their strings, the flag is sprite 0x1C + it).
    pub nationality: u32,
    /// The number of the player's car.
    pub car_number: u32,
    /// The view (`dword_7589BA`: bumper, bonnet, driver, interior, chase 1 .. 3).
    pub view: u32,
    /// The second view (`dword_7589DA`: bonnet, bumper, chase 1 .. 3; 5 is made 0).
    pub second_view: u32,
    pub field_14: u32,
    /// The player's name (a place in `g_playerNames`).
    pub name: u32,
    pub keys: [u8; 12],
    /// The number on the player's car, 1 .. 90, chosen on the player details page.
    pub race_number: u32,
    pub gap_2c: u32,
    /// The gearing chosen on the car choice, 0 .. 2 (its name string 0x21F + it;
    /// `menu::next_gearing`, `menu::previous_gearing`).
    pub gearing: u32,
    pub gap_34: [u8; 80],
    /// Zeroed for each player by `menu::begin_event`.
    pub field_84: u32,
    pub gap_88: u32,
    /// Zeroed for each player by `menu::begin_event`.
    pub field_8c: u32,
    pub no_turbo: u32,
    pub gap_94: [u8; 56],
    pub damage: [u32; 16],
    pub tyre: u32,
    pub brake_balance: u32,
    pub ride_height: u32,
    pub suspension_stiffness: u32,
    pub steering_sensitivity: u32,
    pub gear_ratio: u32,
    pub name_copy: [u8; 32],
    pub field_144: u32,
    pub field_148: u32,
    pub field_14c: u32,
    pub field_150: u32,
    /// The co-driver's sex, from the player details page: 0 male, 1 female (strings
    /// 0x18A, 0x18B).
    pub codriver_sex: u32,
    // The visual preferences (the page of `menu::visual_preferences_loop`), each 0 or 1,
    // flipped by `menu::toggle_*`.
    /// "mapa local".
    pub map_area: u32,
    /// "retrovisor".
    pub rear_mirror: u32,
    /// "contador de rotações".
    pub rev_counter: u32,
    /// "informação do carro".
    pub car_info: u32,
    /// "barra de mensagens".
    pub message_display: u32,
    /// "barra de progresso".
    pub progress_bar: u32,
    /// "informação do piloto".
    pub driver_info: u32,
    /// "notas informativas".
    pub pace_notes: u32,
    /// "indicador de unidades": 0 imperial, 1 metric.
    pub metric_units: u32,
    pub wear_17c: [u32; 31],
    pub tyre_wear: [u32; 5],
    pub gap_20c: [u8; 64],
    pub field_24c: [u32; 8],
    pub gap_26c: [u8; 8],
    pub field_274: u32,
    pub flags_278: u32,
    pub gap_27c: u32,
    /// The controller's steering sensitivity, 0 .. 100 ("sensibilidade da direcção"; the
    /// player details page shows it as a marker at x + 2 * it).
    pub controller_sensitivity: u32,
    /// Set to -1 for every player by `menu::clear_players_284` at an event's start, and kept
    /// over a race by `sub_4843EC` (`dword_75B010`); meaning not read.
    pub field_284: i32,
    pub gap_288: u32,
}

const _: () = assert!(std::mem::size_of::<Player>() == 0x28C);

/// A setting the players' pages change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    /// 0 .. 0x27, round.
    Nationality,
    /// 0 .. 6, round.
    View,
    /// 0 .. 4, round.
    SecondView,
    /// 1 .. 90, round (signed).
    RaceNumber,
    /// 0 .. 100, held (signed).
    Sensitivity,
    /// The rest flip (xor 1).
    CodriverSex,
    MapArea,
    RearMirror,
    RevCounter,
    CarInfo,
    MessageDisplay,
    ProgressBar,
    DriverInfo,
    PaceNotes,
    MetricUnits,
}

impl Player {
    /// The setting's dword.
    pub fn setting(&mut self, s: Setting) -> &mut u32 {
        match s {
            Setting::Nationality => &mut self.nationality,
            Setting::View => &mut self.view,
            Setting::SecondView => &mut self.second_view,
            Setting::RaceNumber => &mut self.race_number,
            Setting::Sensitivity => &mut self.controller_sensitivity,
            Setting::CodriverSex => &mut self.codriver_sex,
            Setting::MapArea => &mut self.map_area,
            Setting::RearMirror => &mut self.rear_mirror,
            Setting::RevCounter => &mut self.rev_counter,
            Setting::CarInfo => &mut self.car_info,
            Setting::MessageDisplay => &mut self.message_display,
            Setting::ProgressBar => &mut self.progress_bar,
            Setting::DriverInfo => &mut self.driver_info,
            Setting::PaceNotes => &mut self.pace_notes,
            Setting::MetricUnits => &mut self.metric_units,
        }
    }

    /// The setting one up (or down), by its rule.
    pub fn change(&mut self, s: Setting, up: bool) {
        let v = self.setting(s);
        *v = stepped(s, *v, up);
    }
}

/// Up by one, back to 0 after `last`.
pub fn wrap_up(value: u32, last: u32) -> u32 {
    if value == last {
        0
    } else {
        value.wrapping_add(1)
    }
}

/// Down by one, to `last` from 0.
pub fn wrap_down(value: u32, last: u32) -> u32 {
    if value == 0 {
        last
    } else {
        value.wrapping_sub(1)
    }
}

/// A setting's next value.
pub fn stepped(s: Setting, value: u32, up: bool) -> u32 {
    let wrap = |last| {
        if up {
            wrap_up(value, last)
        } else {
            wrap_down(value, last)
        }
    };
    match s {
        Setting::Nationality => wrap(0x27),
        Setting::View => wrap(6),
        Setting::SecondView => wrap(4),
        Setting::RaceNumber => {
            if up {
                let next = value.wrapping_add(1);
                if next as i32 > 90 { 1 } else { next }
            } else {
                let next = value.wrapping_sub(1);
                if next as i32 <= 0 { 90 } else { next }
            }
        }
        Setting::Sensitivity => {
            if up {
                let next = value.wrapping_add(1);
                if next as i32 > 100 { 100 } else { next }
            } else {
                let next = value.wrapping_sub(1);
                if next as i32 > 0 { next } else { 0 }
            }
        }
        _ => value ^ 1,
    }
}

/// The keys a player drives with.
pub const KEYS: usize = 12;
/// Escape's scan code: the redefining ends unchanged.
pub const KEY_ESCAPE: u8 = 1;

/// The work copy with `key` put at `place`; false (nothing changed) if the key is reserved.
/// A key the copy already has gets the place's old key (the two swap).
pub fn place_key(keys: &mut [u8; KEYS], place: usize, key: u8, reserved: &[u8]) -> bool {
    if let Some(had) = keys.iter().position(|&k| k == key) {
        keys[had] = keys[place];
    } else if reserved.contains(&key) {
        return false;
    }
    keys[place] = key;
    true
}

/// What a frame of redefining (converted 2026-10-21 from `menu::redefine_keys`' pages) does
/// with the pending key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Redefined {
    /// No key, or a reserved one: the page goes on waiting.
    Waiting,
    /// The copy goes to the player and the redefining ends; `placed`: the key was put in
    /// (the place then becomes 999).
    Done { keys: [u8; KEYS], placed: bool },
}

/// A frame of redefining the key at `place` of the work copy `keys`.
pub fn redefine(keys: [u8; KEYS], place: usize, key: u8, reserved: &[u8]) -> Redefined {
    if key == 0 {
        return Redefined::Waiting;
    }
    if key == KEY_ESCAPE {
        return Redefined::Done {
            keys,
            placed: false,
        };
    }
    let mut keys = keys;
    if place_key(&mut keys, place, key & 0x7F, reserved) {
        Redefined::Done { keys, placed: true }
    } else {
        Redefined::Waiting
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redefining() {
        let keys = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        assert_eq!(redefine(keys, 0, 0, &[]), Redefined::Waiting);
        assert_eq!(redefine(keys, 0, 0x3B, &[0x3B]), Redefined::Waiting);
        let Redefined::Done {
            keys: k,
            placed: true,
        } = redefine(keys, 0, 0x83, &[])
        else {
            panic!()
        };
        assert_eq!(&k[..3], &[3, 2, 1]);
    }

    #[test]
    fn settings() {
        assert_eq!(
            (
                stepped(Setting::Nationality, 0x27, true),
                stepped(Setting::View, 0, false)
            ),
            (0, 6)
        );
        assert_eq!(
            (
                stepped(Setting::RaceNumber, 90, true),
                stepped(Setting::RaceNumber, 1, false)
            ),
            (1, 90)
        );
        assert_eq!(
            (
                stepped(Setting::Sensitivity, 100, true),
                stepped(Setting::Sensitivity, 0, false)
            ),
            (100, 0)
        );
        assert_eq!(stepped(Setting::PaceNotes, 1, true), 0);
    }
}
