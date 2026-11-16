//! A player, 0x28C bytes: the player's settings (name, nationality, views, co-driver, keys,
//! controller) and the state of the player's car (which car, damage, setup, wear). What is known
//! of it; the rest are gaps.

use crate::menu::event::HOUR;

/// The players.
pub const PLAYERS: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Player {
    /// Record +0x00: nothing reads or writes it (the saved games carry it as it is).
    pub unused_0: u32,
    /// The player's nationality (0 Great Britain, 1 Argentina, ..; `CaptionTables.nationalities[0]`
    /// has their strings, the flag is sprite 0x1C + it).
    pub nationality: u32,
    /// The number of the player's car.
    pub car_number: u32,
    /// The view (`CaptionTables.views[0]`: bumper, bonnet, driver, interior, chase 1 .. 3).
    pub view: u32,
    /// The second view (`CaptionTables.second_views[0]`: bonnet, bumper, chase 1 .. 3); 5 is
    /// made 0.
    pub second_view: u32,
    /// The section a saved game starts at (`SCREEN_FLOW.start_section`; player 0's, kept
    /// in the saved game).
    pub saved_start_section: u32,
    /// Whose name in the four name buffers is the player's: 0 none, n the name of player n - 1.
    /// The saved games hold the buffer's address here instead (`to_file_bytes`).
    pub name_number: u32,
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
    /// The controller's mode (bit 0; string 0x56D + it).
    pub controller_mode: u32,
    /// +0x40: the stick's end zone in percent of its travel (0 at start): the share at the
    /// far end that counts as full. The controller page's first slider; with the centre dead
    /// zone it leaves the rest of 100 (`lower_end_zone`, `raise_dead_zone`).
    /// `input::set_up_device` copies it to the device's `end_zone` (+0x14). Sure: the
    /// device fields' names and the defaults (0 here, 10 below).
    pub end_zone_pct: u32,
    /// +0x44: the stick's centre dead zone in percent (10 at start): the share around the
    /// middle that reads as 0. The second slider; copied to the device's `dead_zone` (+0x18).
    pub centre_dead_zone_pct: u32,
    /// The device's strength, 0 .. 100 in tens (`step_strength`;
    /// shown for a device that has it).
    pub strength: u32,
    /// Record +0x4C: the word between `strength` and the bindings; no page or race
    /// copy reads it (the device set-up reads the car's copy of it as a gap).
    pub unused_4c: u32,
    /// The controller's bindings (`ControlsBlock`), which the controller pages copy out
    /// (`screens::pick_player_n`) and back (`menu::keep_player_keys`).
    pub bindings: ControlsBlock,
    /// Record +0x7C .. +0x84: nothing reads or writes it.
    pub unused_7c: [u8; 8],
    /// Zeroed for each player by `menu::begin_event` and `menu::reset_single_event`; nothing
    /// reads it.
    pub unread_84: u32,
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
    /// The player's numbers in the standings (`menu::number_standings_b`, `number_all_b`,
    /// `number_standings_a`, `number_all_a`).
    pub number_standings_b: u32,
    pub number_all_b: u32,
    pub number_standings_a: u32,
    pub number_all_a: u32,
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
    /// The wear of the car's parts, 31 words (the race's `wear[0..31]`); `tyre_wear` goes on
    /// with the next five.
    pub parts_wear: [u32; 31],
    /// The tyres' wear, the race's `wear[31..36]`.
    pub tyre_wear: [u32; 5],
    /// The 0x40 bytes of the car's best line (the time trial's; the race's result data).
    pub best_line: [u8; 64],
    /// The car's eight impact marks (`impact_octant_marks`, 0 .. 1 floats' bits, one per
    /// octant of a hit's direction), carried from stage to stage; a body repair zeroes them.
    pub impact_marks: [u32; 8],
    /// Record +0x26C .. +0x274: nothing reads or writes it.
    pub unused_26c: [u8; 8],
    /// +0x274: the A8 championship's start section, 0 .. 6 (`SCREEN_FLOW.start_section_a8`;
    /// player 0's, kept in the saved game).
    pub saved_start_section_a8: u32,
    /// The car's state bits (the tyres' and the body's are set by a repair, `workshop`; -1
    /// after `repair_car`; the race's `flags`).
    pub state_flags: u32,
    /// The word the last race left for the car (the race's result `word`).
    pub result_word: u32,
    /// The controller's steering sensitivity, 0 .. 100 ("sensibilidade da direcção"; the
    /// player details page shows it as a marker at x + 2 * it).
    pub controller_sensitivity: u32,
    /// The HUD's bits: set to -1 for every player by `menu::clear_hud_bits` at an event's
    /// start, taken into the race and given back by `single_rally_race`.
    pub hud_bits: i32,
    /// The money the service spent (the money times 100, `menu::service_pass`); 0 again
    /// after an event's races (`menu::clear_service_spent`).
    pub service_spent: u32,
}

impl Player {
    /// The player whose name buffer is the player's name; None without a name or past the
    /// four players.
    pub const fn name_player(&self) -> Option<usize> {
        match self.name_number {
            n @ 1..=PLAYERS => Some(n as usize - 1),
            _ => None,
        }
    }

    /// The name set to the buffer of player `player` (counted from 0).
    pub const fn set_name_player(&mut self, player: usize) {
        self.name_number = player as u32 + 1;
    }

    /// The record as the saved game keeps it: the layout of `to_le_bytes`, with the word at
    /// +0x18 the address of the name's buffer (`name_address(player)`), or 0 without a name.
    pub fn to_file_bytes(&self, name_address: impl Fn(usize) -> u32) -> [u8; 0x28c] {
        let mut b = self.to_le_bytes();
        let at = self.name_player().map_or(0, name_address);
        b[0x18..0x1c].copy_from_slice(&at.to_le_bytes());
        b
    }

    /// The record from a saved game's bytes. The word at +0x18 is an address of the game that
    /// wrote the file and names nothing here: `name_number` is 0 and the caller keeps its
    /// own.
    pub const fn from_file_bytes(mut b: [u8; 0x28c]) -> Self {
        b[0x18] = 0;
        b[0x19] = 0;
        b[0x1a] = 0;
        b[0x1b] = 0;
        Self::from_le_bytes(b)
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
}

/// The controller's bindings: 0x2C bytes, 11 little-endian dwords, kept in the player's record
/// (+0x50), in the keys' table and in the controller pages' copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlsBlock([u8; CONTROLS_LEN]);

/// The bindings' bytes.
pub const CONTROLS_LEN: usize = 0x2C;

impl ControlsBlock {
    /// The block from its bytes.
    pub const fn from_bytes(b: [u8; CONTROLS_LEN]) -> Self {
        Self(b)
    }

    /// The block's bytes.
    pub const fn to_bytes(self) -> [u8; CONTROLS_LEN] {
        self.0
    }

    /// Dword `k` (0 .. 10; at byte 4 k).
    pub fn word(&self, k: usize) -> u32 {
        let b = &self.0[4 * k..4 * k + 4];
        u32::from_le_bytes([b[0], b[1], b[2], b[3]])
    }

    /// Dword `k` stored.
    pub fn set_word(&mut self, k: usize, v: u32) {
        self.0[4 * k..4 * k + 4].copy_from_slice(&v.to_le_bytes());
    }

    /// The eleven dwords in order.
    pub fn words(&self) -> impl Iterator<Item = u32> + '_ {
        (0..CONTROLS_LEN / 4).map(|k| self.word(k))
    }
}

const _: () = assert!(std::mem::size_of::<Player>() == RECORD_LEN);

/// The record's length.
const RECORD_LEN: usize = 0x28C;

const fn word(b: &[u8; RECORD_LEN], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

const fn bytes<const N: usize>(b: &[u8; RECORD_LEN], at: usize) -> [u8; N] {
    let mut out = [0; N];
    let mut n = 0;
    while n < N {
        out[n] = b[at + n];
        n += 1;
    }
    out
}

const fn words<const N: usize>(b: &[u8; RECORD_LEN], at: usize) -> [u32; N] {
    let mut out = [0; N];
    let mut n = 0;
    while n < N {
        out[n] = word(b, at + 4 * n);
        n += 1;
    }
    out
}

fn put_words(b: &mut [u8; RECORD_LEN], at: usize, values: &[u32]) {
    for (n, v) in values.iter().enumerate() {
        b[at + 4 * n..at + 4 * n + 4].copy_from_slice(&v.to_le_bytes());
    }
}

/// Where each field lies in the record's bytes: one table, read both ways by `to_le_bytes`
/// and `from_le_bytes`. A `word` is a dword, a `signed` word a dword kept as `i32`, `bytes`
/// and `words` arrays of bytes and of dwords. Bytes in no row are 0 when written.
macro_rules! layout {
    ($($at:literal $kind:ident $field:ident,)*) => {
        impl Player {
            /// The record as its 0x28c bytes in memory's layout (+0x18 the name number).
            pub fn to_le_bytes(&self) -> [u8; RECORD_LEN] {
                let mut b = [0; RECORD_LEN];
                $( layout!(@put $kind b, $at, self.$field); )*
                b
            }

            /// The record from its 0x28c bytes in the game's layout (little-endian dwords).
            pub const fn from_le_bytes(b: [u8; RECORD_LEN]) -> Self {
                Self { $( $field: layout!(@get $kind b, $at), )* }
            }
        }
    };
    (@put word $b:ident, $at:literal, $v:expr) => { put_words(&mut $b, $at, &[$v]) };
    (@put signed $b:ident, $at:literal, $v:expr) => { put_words(&mut $b, $at, &[$v as u32]) };
    (@put words $b:ident, $at:literal, $v:expr) => { put_words(&mut $b, $at, &$v) };
    (@put controls $b:ident, $at:literal, $v:expr) => { $b[$at..$at + CONTROLS_LEN].copy_from_slice(&$v.to_bytes()) };
    (@put bytes $b:ident, $at:literal, $v:expr) => { $b[$at..$at + $v.len()].copy_from_slice(&$v) };
    (@get word $b:ident, $at:literal) => { word(&$b, $at) };
    (@get signed $b:ident, $at:literal) => { word(&$b, $at) as i32 };
    (@get words $b:ident, $at:literal) => { words(&$b, $at) };
    (@get controls $b:ident, $at:literal) => { ControlsBlock::from_bytes(bytes(&$b, $at)) };
    (@get bytes $b:ident, $at:literal) => { bytes(&$b, $at) };
}

layout! {
    0x000 word unused_0,
    0x004 word nationality,
    0x008 word car_number,
    0x00C word view,
    0x010 word second_view,
    0x014 word saved_start_section,
    0x018 word name_number,
    0x01C bytes keys,
    0x028 word race_number,
    0x02C word controller,
    0x030 word gearing,
    0x034 word sensitivity_level,
    0x038 bytes axis_codes,
    0x03C word controller_mode,
    0x040 word end_zone_pct,
    0x044 word centre_dead_zone_pct,
    0x048 word strength,
    0x04C word unused_4c,
    0x050 controls bindings,
    0x07C bytes unused_7c,
    0x084 word unread_84,
    0x088 word time,
    0x08C word total,
    0x090 word class,
    0x094 word figure,
    0x098 words stage_points,
    0x0B4 words overall_points,
    0x0D0 word entry,
    0x0D4 words damage,
    0x10C word tyre,
    0x110 word brake_balance,
    0x114 word ride_height,
    0x118 word suspension_stiffness,
    0x11C word steering_sensitivity,
    0x120 word gear_ratio,
    0x124 bytes name_copy,
    0x144 word number_standings_b,
    0x148 word number_all_b,
    0x14C word number_standings_a,
    0x150 word number_all_a,
    0x154 word codriver_sex,
    0x158 word map_area,
    0x15C word rear_mirror,
    0x160 word rev_counter,
    0x164 word car_info,
    0x168 word message_display,
    0x16C word progress_bar,
    0x170 word driver_info,
    0x174 word pace_notes,
    0x178 word metric_units,
    0x17C words parts_wear,
    0x1F8 words tyre_wear,
    0x20C bytes best_line,
    0x24C words impact_marks,
    0x26C bytes unused_26c,
    0x274 word saved_start_section_a8,
    0x278 word state_flags,
    0x27C word result_word,
    0x280 word controller_sensitivity,
    0x284 signed hud_bits,
    0x288 word service_spent,
}

/// A setting the players' pages change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    /// The first dead zone one less while above 0 (signed).
    pub fn lower_end_zone(&mut self) {
        if (self.end_zone_pct as i32) > 0 {
            self.end_zone_pct -= 1;
        }
    }

    /// The second dead zone one less unless 0.
    pub fn lower_centre_dead_zone(&mut self) {
        if self.centre_dead_zone_pct != 0 {
            self.centre_dead_zone_pct = self.centre_dead_zone_pct.wrapping_sub(1);
        }
    }

    /// The strength ten more unless 100, ten
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
        self.bindings.word(k)
    }

    pub fn set_binding(&mut self, k: usize, v: u32) {
        self.bindings.set_word(k, v);
    }

    /// The controller mode's bit 0 turned over.
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

/// A car setup value the player's record keeps (the service pages', the setup files').
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupField {
    Tyre,
    BrakeBalance,
    RideHeight,
    Stiffness,
    SteeringSensitivity,
    GearRatio,
}

impl SetupField {
    pub fn of(self, p: &mut Player) -> &mut u32 {
        match self {
            SetupField::Tyre => &mut p.tyre,
            SetupField::BrakeBalance => &mut p.brake_balance,
            SetupField::RideHeight => &mut p.ride_height,
            SetupField::Stiffness => &mut p.suspension_stiffness,
            SetupField::SteeringSensitivity => &mut p.steering_sensitivity,
            SetupField::GearRatio => &mut p.gear_ratio,
        }
    }

    pub fn get(self, mut p: Player) -> u32 {
        *self.of(&mut p)
    }
}

/// A player's `entry` while taking part.
pub const TAKES_PART: u32 = 1;
/// A player's `entry` once out of the event (`menu::reset_player_times`).
pub const OUT: u32 = 6;

impl Player {
    /// `menu::begin_event` and `reset_single_event`: no total, `unread_84` 0, taking part.
    pub fn begin(&mut self) {
        self.total = 0;
        self.unread_84 = 0;
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

/// `menu::set_player_entry_kind`: the current player's +0xD0 for a score: 1 at 100 or more
/// (signed), else 2.
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

/// What a frame of redefining does with the pending key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Redefined {
    /// No key, or a reserved one: the page goes on waiting.
    Waiting,
    /// The copy goes to the player and the redefining ends; `placed`: the key was put in
    /// (the place then becomes 999).
    Done { keys: [u8; KEYS], placed: bool },
}

/// The redefine keys page's state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Redefining {
    pub on: bool,
    pub place: u32,
}

impl Redefining {
    /// `menu::choose_key_n`: the key at `place` is to be redefined.
    pub fn start(place: u32) -> Self {
        Redefining { on: true, place }
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

    /// A seeded stream of bytes (xorshift).
    fn random_record(seed: u64) -> [u8; RECORD_LEN] {
        let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let mut b = [0; RECORD_LEN];
        for byte in &mut b {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            *byte = (x >> 24) as u8;
        }
        b
    }

    #[test]
    fn every_byte_of_the_record_round_trips() {
        for seed in 0..10_000 {
            let bytes = random_record(seed);
            assert_eq!(Player::from_le_bytes(bytes).to_le_bytes(), bytes, "{seed}");
        }
    }

    #[test]
    fn the_controls_block_is_the_records_0x50_bytes() {
        for seed in 0..10_000 {
            let bytes = random_record(seed);
            let p = Player::from_le_bytes(bytes);
            assert_eq!(p.bindings.to_bytes()[..], bytes[0x50..0x7c]);
            for k in 0..11 {
                let at = 0x50 + 4 * k;
                let word =
                    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
                assert_eq!(p.binding(k), word);
            }
            let mut q = p;
            q.set_binding(10, 0xDEAD_BEEF);
            let mut expected = bytes;
            expected[0x78..0x7c].copy_from_slice(&0xDEAD_BEEFu32.to_le_bytes());
            assert_eq!(q.to_le_bytes(), expected);
            assert_eq!(p.bindings.words().count(), 11);
        }
    }

    /// A record whose every dword is different.
    fn numbered() -> [u8; 0x28c] {
        let mut b = [0; 0x28c];
        for (n, chunk) in b.chunks_mut(4).enumerate() {
            chunk.copy_from_slice(&(0x0101_0101 * (n as u32 + 1)).to_le_bytes());
        }
        b
    }

    #[test]
    fn the_saved_record_carries_the_names_address() {
        let address = |k: usize| 0x0075_5784 + 0x20 * k as u32;
        let bytes = numbered();
        for k in 0..4 {
            let mut p = Player::from_le_bytes(bytes);
            p.set_name_player(k);
            assert_eq!(p.name_player(), Some(k));
            // The file has what the game wrote: the record with the address at +0x18.
            let mut expected = bytes;
            expected[0x18..0x1c].copy_from_slice(&address(k).to_le_bytes());
            assert_eq!(p.to_file_bytes(address), expected);
        }
        let mut none = Player::from_le_bytes(bytes);
        none.name_number = 0;
        let mut expected = bytes;
        expected[0x18..0x1c].fill(0);
        assert_eq!(none.to_file_bytes(address), expected);
        assert_eq!(none.name_player(), None);
        none.name_number = 5;
        assert_eq!(none.name_player(), None);
    }

    #[test]
    fn a_loaded_record_keeps_every_other_byte_and_no_name() {
        let bytes = numbered();
        let p = Player::from_file_bytes(bytes);
        assert_eq!(p.name_number, 0);
        let mut expected = bytes;
        expected[0x18..0x1c].fill(0);
        assert_eq!(p.to_le_bytes(), expected);
    }

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
        p.lower_end_zone();
        assert_eq!(p.end_zone_pct, 0);
        p.end_zone_pct = 2;
        p.lower_end_zone();
        assert_eq!(p.end_zone_pct, 1);
        p.end_zone_pct = u32::MAX;
        p.lower_end_zone();
        assert_eq!(p.end_zone_pct, u32::MAX);
        p.lower_centre_dead_zone();
        assert_eq!(p.centre_dead_zone_pct, 0);
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
