//! A player, 0x28C bytes (`Player` in IDA; converted 2026-10-21 with the players' details
//! pages' settings, `menu::next_nationality` 0x467AFD .. `menu::flip_codriver` 0x467BD6,
//! `menu::next_car_number` 0x470D7F / `previous_car_number` 0x470DA1,
//! `menu::raise_sensitivity` 0x470F05 / `lower_sensitivity` 0x470F2D and the nine visual
//! preferences' toggles 0x467DE2 .. 0x467E92; `menu::take_shown_car` 0x47B29D and
//! `menu::set_player_entry_kind` 0x4A1922): the player's settings (name, nationality,
//! views, co-driver, keys, controller) and the state of the player's car (which car,
//! damage, setup, wear). What is known of it; the rest are gaps.

use crate::menu::event::HOUR;

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
    /// The controller: 0 the keyboard, n the input device n - 1 (`menu::next_controller`).
    pub controller: u32,
    /// The gearing chosen on the car choice, 0 .. 2 (its name string 0x21F + it;
    /// `menu::next_gearing`, `menu::previous_gearing`).
    pub gearing: u32,
    /// The controller's sensitivity, 0 .. 3 (string 0x3A1 + it on the controller page;
    /// the bindings' texts go by it, `screens::controls::Binding::kind`).
    pub sensitivity_level: u32,
    /// The learned pedals' axes (+0x3A the second pedal's, +0x3B the first's;
    /// `screens::learning::pedal`), handed to the device's set-up as one dword.
    pub axis_codes: [u8; 4],
    /// The controller's mode (bit 0; string 0x56D + it; `menu::toggle_player_3c`).
    pub controller_mode: u32,
    /// The controller page's two sliders, the dead zones: their sum leaves the rest of
    /// 100 (`menu::lower_player_40`, `menu::lower_player_44`, `raise_dead_zone`).
    pub dead_zone_1: u32,
    pub dead_zone_2: u32,
    /// The device's strength, 0 .. 100 in tens (`menu::raise_player_48`,
    /// `lower_player_48`; shown for a device that has it).
    pub strength: u32,
    pub gap_4c: u32,
    /// The 0x2C bytes the controller pages copy out (`screens::pick_player_n`) and back
    /// (`menu::keep_player_keys`).
    pub controls_50: [u8; 0x2C],
    pub gap_7c: [u8; 8],
    /// Zeroed for each player by `menu::begin_event`.
    pub field_84: u32,
    /// The stage's time (`HOUR`: none yet; `menu::reset_player_times`).
    pub time: u32,
    /// The event's total time (`menu::reset_player_totals`; 0 at `menu::begin_event`).
    pub total: u32,
    /// The car's class, 0 .. 3 (its description's +8, `menu::take_shown_car`; the
    /// standings' group).
    pub class: u32,
    /// The stage's figure (the time trial's; 0 at `menu::ready_players`).
    pub figure: u32,
    /// Six sections' stage points and their total (`menu::award_stage_points`).
    pub stage_points: [u32; 7],
    /// Six sections' overall points and their total (`menu::award_overall_points`).
    pub overall_points: [u32; 7],
    /// How the player is in the event: 1 taking part (`TAKES_PART`), 2 an entry below
    /// the score (`entry_kind`), 6 out (`menu::reset_player_times`).
    pub entry: u32,
    /// The car's fourteen parts' damage (the service and inspection pages' figures).
    pub damage: [u32; 14],
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
    /// The money the service spent (the money times 100, `menu::service_pass`); 0 again
    /// after an event's races (`menu::clear_players_288`).
    pub service_spent: u32,
}

impl Player {
    /// The record as its 0x28c bytes in the game's layout.
    pub fn to_le_bytes(&self) -> [u8; 0x28c] {
        let mut b = [0; 0x28c];
        let mut word = |at: usize, v: u32| b[at..at + 4].copy_from_slice(&v.to_le_bytes());
        word(0x0, self.gap_0);
        word(0x4, self.nationality);
        word(0x8, self.car_number);
        word(0xc, self.view);
        word(0x10, self.second_view);
        word(0x14, self.field_14);
        word(0x18, self.name);
        word(0x28, self.race_number);
        word(0x2c, self.controller);
        word(0x30, self.gearing);
        word(0x34, self.sensitivity_level);
        word(0x3c, self.controller_mode);
        word(0x40, self.dead_zone_1);
        word(0x44, self.dead_zone_2);
        word(0x48, self.strength);
        word(0x4c, self.gap_4c);
        word(0x84, self.field_84);
        word(0x88, self.time);
        word(0x8c, self.total);
        word(0x90, self.class);
        word(0x94, self.figure);
        for (k, &v) in self.stage_points.iter().enumerate() {
            word(0x98 + 4 * k, v);
        }
        for (k, &v) in self.overall_points.iter().enumerate() {
            word(0xb4 + 4 * k, v);
        }
        word(0xd0, self.entry);
        for (k, &v) in self.damage.iter().enumerate() {
            word(0xd4 + 4 * k, v);
        }
        word(0x10c, self.tyre);
        word(0x110, self.brake_balance);
        word(0x114, self.ride_height);
        word(0x118, self.suspension_stiffness);
        word(0x11c, self.steering_sensitivity);
        word(0x120, self.gear_ratio);
        word(0x144, self.field_144);
        word(0x148, self.field_148);
        word(0x14c, self.field_14c);
        word(0x150, self.field_150);
        word(0x154, self.codriver_sex);
        word(0x158, self.map_area);
        word(0x15c, self.rear_mirror);
        word(0x160, self.rev_counter);
        word(0x164, self.car_info);
        word(0x168, self.message_display);
        word(0x16c, self.progress_bar);
        word(0x170, self.driver_info);
        word(0x174, self.pace_notes);
        word(0x178, self.metric_units);
        for (k, &v) in self.wear_17c.iter().enumerate() {
            word(0x17c + 4 * k, v);
        }
        for (k, &v) in self.tyre_wear.iter().enumerate() {
            word(0x1f8 + 4 * k, v);
        }
        for (k, &v) in self.field_24c.iter().enumerate() {
            word(0x24c + 4 * k, v);
        }
        word(0x274, self.field_274);
        word(0x278, self.flags_278);
        word(0x27c, self.gap_27c);
        word(0x280, self.controller_sensitivity);
        word(0x284, self.field_284 as u32);
        word(0x288, self.service_spent);
        b[0x1c..0x1c + self.keys.len()].copy_from_slice(&self.keys);
        b[0x38..0x3c].copy_from_slice(&self.axis_codes);
        b[0x50..0x50 + self.controls_50.len()].copy_from_slice(&self.controls_50);
        b[0x7c..0x7c + self.gap_7c.len()].copy_from_slice(&self.gap_7c);
        b[0x124..0x124 + self.name_copy.len()].copy_from_slice(&self.name_copy);
        b[0x20c..0x20c + self.gap_20c.len()].copy_from_slice(&self.gap_20c);
        b[0x26c..0x26c + self.gap_26c.len()].copy_from_slice(&self.gap_26c);
        b
    }

    /// The dword at byte `at`.
    pub fn word(&self, at: usize) -> u32 {
        let b = self.to_le_bytes();
        u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
    }

    /// The byte at `at`.
    pub fn byte(&self, at: usize) -> u8 {
        self.to_le_bytes()[at]
    }

    /// Bytes stored by offset.
    pub fn store_bytes(&mut self, at: usize, v: &[u8]) {
        let mut b = self.to_le_bytes();
        b[at..at + v.len()].copy_from_slice(v);
        *self = Self::from_le_bytes(b);
    }

    /// One byte stored by offset.
    pub fn store_byte(&mut self, at: usize, v: u8) {
        let mut b = self.to_le_bytes();
        b[at] = v;
        *self = Self::from_le_bytes(b);
    }

    /// Dwords stored by byte offset, as the event's tables list them.
    pub fn store(&mut self, stores: &[crate::menu::standings::Store]) {
        let mut b = self.to_le_bytes();
        for &(at, v) in stores {
            b[at..at + 4].copy_from_slice(&v.to_le_bytes());
        }
        *self = Self::from_le_bytes(b);
    }

    /// The record from its 0x28c bytes in the game's layout (little-endian dwords).
    pub const fn from_le_bytes(b: [u8; 0x28c]) -> Self {
        const fn word(b: &[u8; 0x28c], at: usize) -> u32 {
            u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
        }
        const fn bytes<const N: usize>(b: &[u8; 0x28c], at: usize) -> [u8; N] {
            let mut out = [0; N];
            let mut n = 0;
            while n < N {
                out[n] = b[at + n];
                n += 1;
            }
            out
        }
        const fn words<const N: usize>(b: &[u8; 0x28c], at: usize) -> [u32; N] {
            let mut out = [0; N];
            let mut n = 0;
            while n < N {
                out[n] = word(b, at + 4 * n);
                n += 1;
            }
            out
        }
        Self {
            gap_0: word(&b, 0x0),
            nationality: word(&b, 0x4),
            car_number: word(&b, 0x8),
            view: word(&b, 0xc),
            second_view: word(&b, 0x10),
            field_14: word(&b, 0x14),
            name: word(&b, 0x18),
            keys: bytes::<0xc>(&b, 0x1c),
            race_number: word(&b, 0x28),
            controller: word(&b, 0x2c),
            gearing: word(&b, 0x30),
            sensitivity_level: word(&b, 0x34),
            axis_codes: bytes::<4>(&b, 0x38),
            controller_mode: word(&b, 0x3c),
            dead_zone_1: word(&b, 0x40),
            dead_zone_2: word(&b, 0x44),
            strength: word(&b, 0x48),
            gap_4c: word(&b, 0x4c),
            controls_50: bytes::<0x2c>(&b, 0x50),
            gap_7c: bytes::<0x8>(&b, 0x7c),
            field_84: word(&b, 0x84),
            time: word(&b, 0x88),
            total: word(&b, 0x8c),
            class: word(&b, 0x90),
            figure: word(&b, 0x94),
            stage_points: words::<7>(&b, 0x98),
            overall_points: words::<7>(&b, 0xb4),
            entry: word(&b, 0xd0),
            damage: words::<14>(&b, 0xd4),
            tyre: word(&b, 0x10c),
            brake_balance: word(&b, 0x110),
            ride_height: word(&b, 0x114),
            suspension_stiffness: word(&b, 0x118),
            steering_sensitivity: word(&b, 0x11c),
            gear_ratio: word(&b, 0x120),
            name_copy: bytes::<0x20>(&b, 0x124),
            field_144: word(&b, 0x144),
            field_148: word(&b, 0x148),
            field_14c: word(&b, 0x14c),
            field_150: word(&b, 0x150),
            codriver_sex: word(&b, 0x154),
            map_area: word(&b, 0x158),
            rear_mirror: word(&b, 0x15c),
            rev_counter: word(&b, 0x160),
            car_info: word(&b, 0x164),
            message_display: word(&b, 0x168),
            progress_bar: word(&b, 0x16c),
            driver_info: word(&b, 0x170),
            pace_notes: word(&b, 0x174),
            metric_units: word(&b, 0x178),
            wear_17c: words::<31>(&b, 0x17c),
            tyre_wear: words::<5>(&b, 0x1f8),
            gap_20c: bytes::<0x40>(&b, 0x20c),
            field_24c: words::<8>(&b, 0x24c),
            gap_26c: bytes::<0x8>(&b, 0x26c),
            field_274: word(&b, 0x274),
            flags_278: word(&b, 0x278),
            gap_27c: word(&b, 0x27c),
            controller_sensitivity: word(&b, 0x280),
            field_284: word(&b, 0x284) as i32,
            service_spent: word(&b, 0x288),
        }
    }
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
    /// `menu::lower_player_40`: the first dead zone one less while above 0 (signed).
    pub fn lower_dead_zone_1(&mut self) {
        if (self.dead_zone_1 as i32) > 0 {
            self.dead_zone_1 -= 1;
        }
    }

    /// `menu::lower_player_44`: the second dead zone one less unless 0.
    pub fn lower_dead_zone_2(&mut self) {
        if self.dead_zone_2 != 0 {
            self.dead_zone_2 = self.dead_zone_2.wrapping_sub(1);
        }
    }

    /// `menu::raise_player_48` / `lower_player_48`: the strength ten more unless 100, ten
    /// less unless 0.
    pub fn step_strength(&mut self, up: bool) {
        if up && self.strength != 100 {
            self.strength = self.strength.wrapping_add(10);
        } else if !up && self.strength != 0 {
            self.strength = self.strength.wrapping_sub(10);
        }
    }

    /// `menu::take_shown_car`: the car of a car list's entry (its +0 the car's number, +8
    /// its class) given to the player.
    pub fn take_car(&mut self, number: u32, class: u32) {
        self.car_number = number;
        self.class = class;
    }

    /// Binding `k` (0 .. 10; +0x50 + 4 k).
    pub fn binding(&self, k: usize) -> u32 {
        let b = &self.controls_50[4 * k..4 * k + 4];
        u32::from_le_bytes([b[0], b[1], b[2], b[3]])
    }

    pub fn set_binding(&mut self, k: usize, v: u32) {
        self.controls_50[4 * k..4 * k + 4].copy_from_slice(&v.to_le_bytes());
    }

    /// `menu::toggle_player_3c`: the controller mode's bit 0 turned over.
    pub fn toggle_controller_mode(&mut self) {
        self.controller_mode ^= 1;
    }

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

/// A player's `entry` while taking part.
pub const TAKES_PART: u32 = 1;
/// A player's `entry` once out of the event (`menu::reset_player_times`).
pub const OUT: u32 = 6;

impl Player {
    /// `menu::begin_event` and `reset_single_event`: no total, +0x84 0, taking part.
    pub fn begin(&mut self) {
        self.total = 0;
        self.field_84 = 0;
        self.entry = TAKES_PART;
    }

    /// `menu::reset_player_times`: no stage time yet, out.
    pub fn reset_times(&mut self) {
        self.time = HOUR;
        self.entry = OUT;
    }

    /// `menu::reset_player_totals`: no total or stage time yet.
    pub fn reset_totals(&mut self) {
        self.total = HOUR;
        self.time = HOUR;
    }

    /// `menu::ready_players`: taking part again, time and figure 0.
    pub fn ready(&mut self) {
        self.entry = TAKES_PART;
        self.time = 0;
        self.figure = 0;
    }

    /// `menu::add_player_times`: one that took part has its stage time added to its
    /// total, another both none yet.
    pub fn add_time(&mut self) {
        if self.entry == TAKES_PART {
            self.total = self.total.wrapping_add(self.time);
        } else {
            self.time = HOUR;
            self.total = HOUR;
        }
    }

    /// `menu::clear_result_slots`: the stage and overall points emptied
    /// (`event::empty_result_slot`).
    pub fn clear_results(&mut self) {
        for (k, slot) in self
            .stage_points
            .iter_mut()
            .chain(&mut self.overall_points)
            .enumerate()
        {
            *slot = crate::menu::event::empty_result_slot(k);
        }
    }

    /// `menu::begin_event`: the setup's defaults: gear ratio 2, brake balance 0x32, tyre
    /// 3, ride height 0x46, suspension 0x1E, steering sensitivity the controller's.
    pub fn default_setup(&mut self) {
        self.gear_ratio = 2;
        self.brake_balance = 0x32;
        self.tyre = 3;
        self.ride_height = 0x46;
        self.suspension_stiffness = 0x1E;
        self.steering_sensitivity = self.controller_sensitivity;
    }
}

/// `menu::set_player_entry_kind`: the current player's +0xD0 for a score
/// (`dword_75AC80`): 1 at 100 or more (signed), else 2.
pub fn entry_kind(score: i32) -> u32 {
    if score >= 100 { 1 } else { 2 }
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

/// The redefine keys page's state (`g_redefiningKey`, `g_redefinedKeyPlace`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Redefining {
    pub on: u32,
    pub place: u32,
}

impl Redefining {
    /// `menu::choose_key_n`: the key at `place` is to be redefined.
    pub fn start(place: u32) -> Self {
        Redefining { on: 1, place }
    }
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
    fn small() {
        assert_eq!(
            (entry_kind(100), entry_kind(99), entry_kind(-200)),
            (1, 2, 2)
        );
    }

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

    #[test]
    fn event_steps() {
        let mut p = Player::from_le_bytes([0; 0x28c]);
        p.time = 5;
        p.total = u32::MAX;
        p.entry = TAKES_PART;
        p.add_time();
        assert_eq!((p.time, p.total), (5, 4));
        p.entry = 2;
        p.add_time();
        assert_eq!((p.time, p.total), (HOUR, HOUR));
        p.clear_results();
        let b = p.to_le_bytes();
        let at = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        assert_eq!(
            (at(0x98), at(0xB0), at(0xB4), at(0xCC)),
            (u32::MAX, 0, u32::MAX, 0)
        );
        assert_eq!(at(0xD0), 2);
        p.controller_sensitivity = 9;
        p.default_setup();
        assert_eq!((p.word(0x11C), p.word(0x120), p.word(0x10C)), (9, 2, 3));
    }

    #[test]
    fn bytes_there_and_back() {
        let mut bytes = [0u8; 0x28c];
        for (n, b) in bytes.iter_mut().enumerate() {
            *b = (n * 7) as u8;
        }
        let p = Player::from_le_bytes(bytes);
        assert_eq!(p.to_le_bytes(), bytes);
        let mut q = p;
        q.store(&[(0x8C, 5), (0xD0, 1), (0x98, 7), (0xCC, 8), (0xD4, 9)]);
        assert_eq!((q.total, q.entry, q.stage_points[0]), (5, 1, 7));
        assert_eq!((q.overall_points[6], q.damage[0]), (8, 9));
        assert_eq!(q.word(0xD0), 1);
        assert_eq!(q.word(4), p.nationality);
    }

    #[test]
    fn small_fields() {
        let mut p = Player::from_le_bytes([0; 0x28c]);
        p.lower_dead_zone_1();
        assert_eq!(p.dead_zone_1, 0);
        p.dead_zone_1 = 2;
        p.lower_dead_zone_1();
        assert_eq!(p.dead_zone_1, 1);
        p.dead_zone_1 = u32::MAX;
        p.lower_dead_zone_1();
        assert_eq!(p.dead_zone_1, u32::MAX);
        p.lower_dead_zone_2();
        assert_eq!(p.dead_zone_2, 0);
        p.step_strength(false);
        assert_eq!(p.strength, 0);
        p.strength = 100;
        p.step_strength(true);
        assert_eq!(p.strength, 100);
        p.step_strength(false);
        assert_eq!(p.strength, 90);
        p.toggle_controller_mode();
        assert_eq!(p.controller_mode, 1);
    }
}
