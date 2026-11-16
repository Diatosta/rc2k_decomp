//! The options as the new file shows them: named, readable values in place of the old file's
//! runs of bytes. `Document::from(&Options)` and `Document::into_options` convert both ways;
//! the model (`Options`) stays the old file's byte form.
//!
//! - Texts (names, addresses) are strings in the ANSI code page, read up to their first NUL.
//!   What a buffer holds after the NUL is not kept (nothing reads it: the texts are copied
//!   whole or up to their NUL).
//! - The saved connection is a GUID in its usual text form.
//! - A list of car numbers leaves out its trailing "none" entries (-1 in the game); reading
//!   puts them back.
//! - Each player is a table of its own (`players.player_1` .. `player_4`) with the record's
//!   values by name; the bytes of the record nothing reads are `kept_*` values.
//! - The stage records (`records.championship`, `records.events`) and the rallies' best times
//!   (`records.rallies`) are lists of tables.
//!
//! A value the game treats as signed (a time, a "none" of -1) is written signed; reading takes
//! either way of writing a dword.

use super::OptionsError;
use super::model::{Bytes, Display, Kept, Options, Sound, Words};
use crate::files::ansi::{ansi_bytes, ansi_text};
use crate::menu::player::{ControlsBlock, Player};
use crate::saves::entries::{BestTime, TIME_LEN};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The stage records' size.
const RECORD_LEN: usize = crate::files::records::RECORD_LEN;
/// Records in the championship's table, in an event's table.
const CHAMPIONSHIP_RECORDS: usize = 66;
const EVENT_RECORDS: usize = 7;
/// Best times in a rally's table.
const RALLY_TIMES: usize = 20;
const EVENTS: usize = 12;
const RALLIES: usize = 6;

fn problem(what: impl std::fmt::Display) -> OptionsError {
    OptionsError::Text(what.to_string())
}

// ---- values ----

/// The text in `bytes` up to the first NUL.
fn text_in(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    ansi_text(&bytes[..end])
}

/// `text` as the `len` bytes of a buffer, the rest zeros.
fn buffer_of(text: &str, len: usize, what: &str) -> Result<Vec<u8>, OptionsError> {
    let mut bytes = ansi_bytes(text);
    if bytes.len() > len {
        return Err(problem(format!(
            "{what} is {} characters, its room {len}",
            bytes.len()
        )));
    }
    bytes.resize(len, 0);
    Ok(bytes)
}

/// A dword as the number it is written as: signed.
fn signed(word: u32) -> i32 {
    word as i32
}

/// A dword from a number written signed or not.
fn word_of(number: i64, what: &str) -> Result<u32, OptionsError> {
    if (i64::from(i32::MIN)..=i64::from(u32::MAX)).contains(&number) {
        Ok(number as u32)
    } else {
        Err(problem(format!("{what}: {number} does not fit a dword")))
    }
}

fn dword(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// A GUID's usual text, `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`, from its 16 bytes (the first
/// three groups little-endian).
fn guid_text(b: &[u8]) -> String {
    format!(
        "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        dword(b, 0),
        u16::from_le_bytes([b[4], b[5]]),
        u16::from_le_bytes([b[6], b[7]]),
        b[8],
        b[9],
        b[10],
        b[11],
        b[12],
        b[13],
        b[14],
        b[15]
    )
}

/// The 16 bytes of a GUID's text (braces allowed).
fn guid_bytes(text: &str) -> Result<Vec<u8>, OptionsError> {
    let bad = || {
        problem(format!(
            "'{text}' is not a GUID (XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX)"
        ))
    };
    let inner = text.trim().trim_start_matches('{').trim_end_matches('}');
    let groups: Vec<&str> = inner.split('-').collect();
    if groups.iter().map(|g| g.len()).collect::<Vec<_>>() != [8, 4, 4, 4, 12] {
        return Err(bad());
    }
    let hex = |g: &str| u64::from_str_radix(g, 16).map_err(|_| bad());
    let mut out = Vec::with_capacity(16);
    out.extend_from_slice(&(hex(groups[0])? as u32).to_le_bytes());
    out.extend_from_slice(&(hex(groups[1])? as u16).to_le_bytes());
    out.extend_from_slice(&(hex(groups[2])? as u16).to_le_bytes());
    for pair in groups[3]
        .as_bytes()
        .chunks(2)
        .chain(groups[4].as_bytes().chunks(2))
    {
        let pair = std::str::from_utf8(pair).map_err(|_| bad())?;
        out.push(hex(pair)? as u8);
    }
    Ok(out)
}

/// A float as the shortest number that reads back as the same `f32`.
fn float_of(bits: u32) -> f64 {
    let value = f32::from_bits(bits);
    format!("{value}")
        .parse::<f64>()
        .unwrap_or(f64::from(value))
}

fn bits_of(float: f64) -> u32 {
    (float as f32).to_bits()
}

/// A list of `N` dwords written out (an exact length).
fn exact<const N: usize, T: Copy>(list: &[T], what: &str) -> Result<[T; N], OptionsError> {
    <[T; N]>::try_from(list)
        .map_err(|_| problem(format!("{what} has {} entries, {N} wanted", list.len())))
}

/// The car numbers without the "none"s at the end.
fn numbers_of(words: &[u32]) -> Vec<i64> {
    let used = words
        .iter()
        .rposition(|&w| w != u32::MAX)
        .map_or(0, |n| n + 1);
    words[..used]
        .iter()
        .map(|&w| i64::from(signed(w)))
        .collect()
}

/// `N` car numbers from the list, "none" after its end.
fn numbers_in<const N: usize>(list: &[i64], what: &str) -> Result<Vec<u32>, OptionsError> {
    if list.len() > N {
        return Err(problem(format!(
            "{what} has {} entries, {N} at most",
            list.len()
        )));
    }
    let mut words = list
        .iter()
        .map(|&n| word_of(n, what))
        .collect::<Result<Vec<_>, _>>()?;
    words.resize(N, u32::MAX);
    Ok(words)
}

fn words_in<const N: usize>(list: &[u32], what: &str) -> Result<Vec<u32>, OptionsError> {
    Ok(exact::<N, u32>(list, what)?.to_vec())
}

// ---- the document ----

/// The file's contents by area.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub display: Display,
    pub sound: Sound,
    pub network: NetworkView,
    pub pages: PagesView,
    pub cars: CarsView,
    pub players: PlayersView,
    pub records: RecordsView,
    pub kept: Kept,
}

/// The network settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetworkView {
    pub machine_name: String,
    /// The most players a hosted session takes.
    pub most_players: u32,
    pub host_address: String,
    pub port: u32,
    /// The saved connection's GUID.
    pub saved_connection: String,
}

/// The options pages' own settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PagesView {
    /// 11 switches: mipmapping, buffering, mode changed, modes switch, lens flare, two unused
    /// percentages, reflections, car shadows, car detail, headlights.
    pub switches: Vec<u32>,
    /// Audio quality, DirectSound 3D, EAX, the retired A3D switch, Dolby, channels.
    pub audio: Vec<u32>,
    pub services: u32,
    pub event_kept: u32,
    pub race_settings: u32,
    /// Six arcade levels, 1 for each one won.
    pub arcade_levels_won: Vec<u8>,
}

/// The car number lists, without their trailing "none" entries (-1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CarsView {
    /// Up to 40 numbers.
    pub first_numbers: Vec<i64>,
    /// Up to 20 numbers.
    pub second_numbers: Vec<i64>,
}

/// The four players, each a table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayersView {
    pub player_1: PlayerView,
    pub player_2: PlayerView,
    pub player_3: PlayerView,
    pub player_4: PlayerView,
}

/// The best times and records kept in the options file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordsView {
    pub tyre_change_holder: String,
    pub tyre_change_time: u32,
    /// The championship's 66 stage records.
    pub championship: Vec<StageRecordView>,
    /// The events' tables, `event_00` .. `event_11`: 7 stage records each.
    pub events: BTreeMap<String, Vec<StageRecordView>>,
    /// The rallies' best times, `rally_0` .. `rally_5`: 20 times each (two stages of ten).
    pub rallies: BTreeMap<String, Vec<BestTimeView>>,
}

fn event_key(n: usize) -> String {
    format!("event_{n:02}")
}

fn rally_key(n: usize) -> String {
    format!("rally_{n}")
}

// ---- a player ----

/// A player's record by name.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerView {
    pub name: String,
    pub nationality: u32,
    pub car_number: u32,
    pub view: u32,
    pub second_view: u32,
    pub saved_start_section: u32,
    /// Whose name buffer the player's name is: 0 none, n the name of player n.
    pub name_slot: u32,
    /// The twelve keys' scan codes.
    pub key_scan_codes: [u8; 12],
    /// The number on the car, 1 .. 90.
    pub race_number: u32,
    /// 0 the keyboard, n the input device n - 1.
    pub controller: u32,
    /// The controller the player chose, by its name, serial number (when it has one) and which
    /// of its name it is: the player has it whatever order the controllers come in, and back
    /// when it is plugged in after being away (`controller` is then 0 meanwhile).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controller_device: Option<crate::input::device_id::DeviceId>,
    pub gearing: u32,
    pub sensitivity_level: u32,
    /// The learned pedals' axes.
    pub pedal_axis_codes: [u8; 4],
    pub controller_mode: u32,
    pub end_zone_percent: u32,
    pub centre_dead_zone_percent: u32,
    pub strength: u32,
    /// The eleven controls' buttons, in the controller page's order (0 none).
    pub binding_buttons: [u8; 11],
    /// The eleven controls' directions (0 none).
    pub binding_directions: [u8; 11],
    pub time: u32,
    pub total: u32,
    pub class: u32,
    pub figure: u32,
    pub stage_points: [i32; 7],
    pub overall_points: [i32; 7],
    pub entry: u32,
    pub damage: [u32; 14],
    pub setup_tyre: u32,
    pub setup_brake_balance: u32,
    pub setup_ride_height: u32,
    pub setup_suspension_stiffness: u32,
    pub setup_steering_sensitivity: u32,
    pub setup_gear_ratio: u32,
    pub name_copy: String,
    pub number_standings_b: i32,
    pub number_all_b: i32,
    pub number_standings_a: i32,
    pub number_all_a: i32,
    pub codriver_sex: u32,
    pub map_area: u32,
    pub rear_mirror: u32,
    pub rev_counter: u32,
    pub car_info: u32,
    pub message_display: u32,
    pub progress_bar: u32,
    pub driver_info: u32,
    pub pace_notes: u32,
    pub metric_units: u32,
    pub parts_wear: [u32; 31],
    pub tyre_wear: [u32; 5],
    /// The car's best line: sixteen checkpoint times.
    pub best_line_checkpoint_times: [i32; 16],
    /// Eight impact marks, 0 .. 1.
    pub impact_marks: [f64; 8],
    pub saved_start_section_a8: u32,
    pub state_flags: i32,
    pub result_word: u32,
    pub controller_sensitivity: u32,
    pub hud_bits: i32,
    pub service_spent: u32,
    /// Bits of the bindings' words above their button and direction.
    pub kept_binding_bits: [u16; 11],
    /// Parts of the record nothing reads, kept so that the old file is written back exactly.
    pub kept_unused_0: u32,
    pub kept_unused_4c: u32,
    pub kept_unused_7c: [u8; 8],
    pub kept_unread_84: u32,
    pub kept_unused_26c: [u8; 8],
}

impl PlayerView {
    fn new(record: &[u8], name: &[u8]) -> Self {
        let p = Player::from_le_bytes(record.try_into().expect("a player's 0x28C bytes"));
        let words = p.bindings.words().collect::<Vec<_>>();
        let (best_line, impact) = (
            std::array::from_fn(|n| signed(dword(&p.best_line, 4 * n))),
            std::array::from_fn(|n| float_of(p.impact_marks[n])),
        );
        Self {
            name: text_in(name),
            nationality: p.nationality,
            car_number: p.car_number,
            view: p.view,
            second_view: p.second_view,
            saved_start_section: p.saved_start_section,
            name_slot: p.name_number,
            key_scan_codes: p.keys,
            race_number: p.race_number,
            controller: p.controller,
            controller_device: None,
            gearing: p.gearing,
            sensitivity_level: p.sensitivity_level,
            pedal_axis_codes: p.axis_codes,
            controller_mode: p.controller_mode,
            end_zone_percent: p.end_zone_pct,
            centre_dead_zone_percent: p.centre_dead_zone_pct,
            strength: p.strength,
            binding_buttons: std::array::from_fn(|n| words[n] as u8),
            binding_directions: std::array::from_fn(|n| (words[n] >> 8) as u8),
            time: p.time,
            total: p.total,
            class: p.class,
            figure: p.figure,
            stage_points: p.stage_points.map(signed),
            overall_points: p.overall_points.map(signed),
            entry: p.entry,
            damage: p.damage,
            setup_tyre: p.tyre,
            setup_brake_balance: p.brake_balance,
            setup_ride_height: p.ride_height,
            setup_suspension_stiffness: p.suspension_stiffness,
            setup_steering_sensitivity: p.steering_sensitivity,
            setup_gear_ratio: p.gear_ratio,
            name_copy: text_in(&p.name_copy),
            number_standings_b: signed(p.number_standings_b),
            number_all_b: signed(p.number_all_b),
            number_standings_a: signed(p.number_standings_a),
            number_all_a: signed(p.number_all_a),
            codriver_sex: p.codriver_sex,
            map_area: p.map_area,
            rear_mirror: p.rear_mirror,
            rev_counter: p.rev_counter,
            car_info: p.car_info,
            message_display: p.message_display,
            progress_bar: p.progress_bar,
            driver_info: p.driver_info,
            pace_notes: p.pace_notes,
            metric_units: p.metric_units,
            parts_wear: p.parts_wear,
            tyre_wear: p.tyre_wear,
            best_line_checkpoint_times: best_line,
            impact_marks: impact,
            saved_start_section_a8: p.saved_start_section_a8,
            state_flags: signed(p.state_flags),
            result_word: p.result_word,
            controller_sensitivity: p.controller_sensitivity,
            hud_bits: p.hud_bits,
            service_spent: p.service_spent,
            kept_binding_bits: std::array::from_fn(|n| (words[n] >> 16) as u16),
            kept_unused_0: p.unused_0,
            kept_unused_4c: p.unused_4c,
            kept_unused_7c: p.unused_7c,
            kept_unread_84: p.unread_84,
            kept_unused_26c: p.unused_26c,
        }
    }

    /// The record's 0x28C bytes and the name's 0x20.
    fn bytes(&self) -> Result<(Vec<u8>, Vec<u8>), OptionsError> {
        let mut bindings = [0u8; crate::menu::player::CONTROLS_LEN];
        for n in 0..11 {
            let word = u32::from(self.binding_buttons[n])
                | u32::from(self.binding_directions[n]) << 8
                | u32::from(self.kept_binding_bits[n]) << 16;
            bindings[4 * n..4 * n + 4].copy_from_slice(&word.to_le_bytes());
        }
        let mut best_line = [0u8; 64];
        for (n, t) in self.best_line_checkpoint_times.iter().enumerate() {
            best_line[4 * n..4 * n + 4].copy_from_slice(&t.to_le_bytes());
        }
        let p = Player {
            unused_0: self.kept_unused_0,
            nationality: self.nationality,
            car_number: self.car_number,
            view: self.view,
            second_view: self.second_view,
            saved_start_section: self.saved_start_section,
            name_number: self.name_slot,
            keys: self.key_scan_codes,
            race_number: self.race_number,
            controller: self.controller,
            gearing: self.gearing,
            sensitivity_level: self.sensitivity_level,
            axis_codes: self.pedal_axis_codes,
            controller_mode: self.controller_mode,
            end_zone_pct: self.end_zone_percent,
            centre_dead_zone_pct: self.centre_dead_zone_percent,
            strength: self.strength,
            unused_4c: self.kept_unused_4c,
            bindings: ControlsBlock::from_bytes(bindings),
            unused_7c: self.kept_unused_7c,
            unread_84: self.kept_unread_84,
            time: self.time,
            total: self.total,
            class: self.class,
            figure: self.figure,
            stage_points: self.stage_points.map(|v| v as u32),
            overall_points: self.overall_points.map(|v| v as u32),
            entry: self.entry,
            damage: self.damage,
            tyre: self.setup_tyre,
            brake_balance: self.setup_brake_balance,
            ride_height: self.setup_ride_height,
            suspension_stiffness: self.setup_suspension_stiffness,
            steering_sensitivity: self.setup_steering_sensitivity,
            gear_ratio: self.setup_gear_ratio,
            name_copy: buffer_of(&self.name_copy, 32, "name_copy")?
                .try_into()
                .expect("32 bytes"),
            number_standings_b: self.number_standings_b as u32,
            number_all_b: self.number_all_b as u32,
            number_standings_a: self.number_standings_a as u32,
            number_all_a: self.number_all_a as u32,
            codriver_sex: self.codriver_sex,
            map_area: self.map_area,
            rear_mirror: self.rear_mirror,
            rev_counter: self.rev_counter,
            car_info: self.car_info,
            message_display: self.message_display,
            progress_bar: self.progress_bar,
            driver_info: self.driver_info,
            pace_notes: self.pace_notes,
            metric_units: self.metric_units,
            parts_wear: self.parts_wear,
            tyre_wear: self.tyre_wear,
            best_line,
            impact_marks: self.impact_marks.map(bits_of),
            unused_26c: self.kept_unused_26c,
            saved_start_section_a8: self.saved_start_section_a8,
            state_flags: self.state_flags as u32,
            result_word: self.result_word,
            controller_sensitivity: self.controller_sensitivity,
            hud_bits: self.hud_bits,
            service_spent: self.service_spent,
        };
        Ok((
            p.to_le_bytes().to_vec(),
            buffer_of(&self.name, 0x20, "name")?,
        ))
    }
}

// ---- stage records and best times ----

/// A stage record (the stages, service areas and rally ends of the game's tables) with its
/// two record holders.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StageRecordView {
    /// 0 a stage, 1 a service area, 3 or 4 a rally's end, 5 a welcome, -1 a table's end.
    pub tag: i32,
    pub track: u32,
    pub pack_id: u32,
    pub service_figure: u32,
    /// The time of day it starts at (hhmm).
    pub start_hhmm: u32,
    pub service_time: u32,
    /// The four classes' times.
    pub class_times: [i32; 4],
    pub time_trial_speed: i32,
    pub fastest_speed: i32,
    pub race_stage: u32,
    pub name_string: u32,
    pub event_slot: u32,
    pub number: u32,
    /// The two map markers' (x, y) each.
    pub marker_points: [i32; 4],
    pub fastest_name: String,
    pub fastest_nation: u32,
    pub fastest_car: u32,
    pub time_trial_name: String,
    pub time_trial_nation: u32,
    pub time_trial_car: u32,
    /// The word at +0x28, which nothing reads; left out while it is 0.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub kept_unread_28: u32,
}

fn is_zero(v: &u32) -> bool {
    *v == 0
}

impl StageRecordView {
    fn new(b: &[u8]) -> Self {
        Self {
            tag: signed(dword(b, 0)),
            track: dword(b, 4),
            pack_id: dword(b, 8),
            service_figure: dword(b, 0xC),
            start_hhmm: dword(b, 0x10),
            service_time: dword(b, 0x14),
            class_times: std::array::from_fn(|n| signed(dword(b, 0x18 + 4 * n))),
            kept_unread_28: dword(b, 0x28),
            time_trial_speed: signed(dword(b, 0x2C)),
            fastest_speed: signed(dword(b, 0x30)),
            race_stage: dword(b, 0x34),
            name_string: dword(b, 0x38),
            event_slot: dword(b, 0x3C),
            number: dword(b, 0x40),
            marker_points: std::array::from_fn(|n| signed(dword(b, 0x44 + 4 * n))),
            fastest_name: text_in(&b[0x54..0x74]),
            fastest_nation: dword(b, 0x74),
            fastest_car: dword(b, 0x78),
            time_trial_name: text_in(&b[0x7C..0x9C]),
            time_trial_nation: dword(b, 0x9C),
            time_trial_car: dword(b, 0xA0),
        }
    }

    fn bytes(&self) -> Result<Vec<u8>, OptionsError> {
        let mut b = vec![0u8; RECORD_LEN];
        put(&mut b, 0, self.tag as u32);
        put(&mut b, 4, self.track);
        put(&mut b, 8, self.pack_id);
        put(&mut b, 0xC, self.service_figure);
        put(&mut b, 0x10, self.start_hhmm);
        put(&mut b, 0x14, self.service_time);
        for (n, t) in self.class_times.iter().enumerate() {
            put(&mut b, 0x18 + 4 * n, *t as u32);
        }
        put(&mut b, 0x28, self.kept_unread_28);
        put(&mut b, 0x2C, self.time_trial_speed as u32);
        put(&mut b, 0x30, self.fastest_speed as u32);
        put(&mut b, 0x34, self.race_stage);
        put(&mut b, 0x38, self.name_string);
        put(&mut b, 0x3C, self.event_slot);
        put(&mut b, 0x40, self.number);
        for (n, p) in self.marker_points.iter().enumerate() {
            put(&mut b, 0x44 + 4 * n, *p as u32);
        }
        b[0x54..0x74].copy_from_slice(&buffer_of(&self.fastest_name, 0x20, "fastest_name")?);
        put(&mut b, 0x74, self.fastest_nation);
        put(&mut b, 0x78, self.fastest_car);
        b[0x7C..0x9C].copy_from_slice(&buffer_of(&self.time_trial_name, 0x20, "time_trial_name")?);
        put(&mut b, 0x9C, self.time_trial_nation);
        put(&mut b, 0xA0, self.time_trial_car);
        Ok(b)
    }
}

/// A best time of a rally's table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BestTimeView {
    /// -1 ends a list.
    pub first: i32,
    /// The player (1 .. 4); 0 for a made-up entry.
    pub player: u32,
    pub nationality: u32,
    /// A made-up entry's name string, an entered one's name pointer.
    pub name_id: u32,
    pub car: u32,
    pub key: i32,
    pub rally_time: i32,
    pub group: u32,
    pub name: String,
    /// 1 for the entry just made.
    pub mark: u32,
    pub figure: u32,
    pub entered: u32,
    /// The cheats on when it was driven.
    pub cheats: u32,
    pub sealed_time: [i32; 2],
    pub checkpoint_times: [i32; 16],
    /// The word after the name, which nothing reads.
    pub kept_unread_48: u32,
}

impl BestTimeView {
    fn new(b: &[u8]) -> Self {
        let t = BestTime::read(b);
        let h = &t.head;
        Self {
            first: signed(h.first),
            player: h.player,
            nationality: h.nation,
            name_id: h.name_id,
            car: h.car,
            key: signed(h.key),
            rally_time: signed(h.rally_time),
            group: h.group,
            name: text_in(&h.name),
            mark: h.mark,
            figure: h.figure,
            entered: h.entered,
            cheats: t.cheats,
            sealed_time: t.sealed.map(signed),
            checkpoint_times: t.checkpoints.map(signed),
            kept_unread_48: h.unread_48,
        }
    }

    fn bytes(&self) -> Result<Vec<u8>, OptionsError> {
        let mut t = BestTime::default();
        let h = &mut t.head;
        h.first = self.first as u32;
        h.player = self.player;
        h.nation = self.nationality;
        h.name_id = self.name_id;
        h.car = self.car;
        h.key = self.key as u32;
        h.rally_time = self.rally_time as u32;
        h.group = self.group;
        h.name = buffer_of(&self.name, 0x20, "name")?
            .try_into()
            .expect("32 bytes");
        h.mark = self.mark;
        h.figure = self.figure;
        h.entered = self.entered;
        h.unread_48 = self.kept_unread_48;
        t.cheats = self.cheats;
        t.sealed = self.sealed_time.map(|v| v as u32);
        t.checkpoints = self.checkpoint_times.map(|v| v as u32);
        let mut b = vec![0u8; TIME_LEN];
        t.write(&mut b);
        Ok(b)
    }
}

// ---- both ways ----

impl From<&Options> for Document {
    fn from(o: &Options) -> Self {
        let player = |n: usize| PlayerView {
            controller_device: o.players.devices[n].clone(),
            ..PlayerView::new(&o.players.records[n].0, &o.players.names[n].0)
        };
        let records = &o.records;
        Self {
            display: o.display.clone(),
            sound: o.sound.clone(),
            network: NetworkView {
                machine_name: text_in(&o.network.machine_name.0),
                most_players: o.network.most_players,
                host_address: text_in(&o.network.host_address.0),
                port: o.network.port,
                saved_connection: guid_text(&o.network.saved_connection.0),
            },
            pages: PagesView {
                switches: o.pages.switches.0.clone(),
                audio: o.pages.audio.0.clone(),
                services: o.pages.services,
                event_kept: o.pages.event_kept,
                race_settings: o.pages.race_settings,
                arcade_levels_won: o.pages.arcade_levels_won.0.clone(),
            },
            cars: CarsView {
                first_numbers: numbers_of(&o.cars.first_numbers.0),
                second_numbers: numbers_of(&o.cars.second_numbers.0),
            },
            players: PlayersView {
                player_1: player(0),
                player_2: player(1),
                player_3: player(2),
                player_4: player(3),
            },
            records: RecordsView {
                tyre_change_holder: text_in(&records.tyre_change_holder.0),
                tyre_change_time: records.tyre_change_time,
                championship: records
                    .championship
                    .0
                    .chunks(RECORD_LEN)
                    .map(StageRecordView::new)
                    .collect(),
                events: (0..EVENTS)
                    .map(|n| {
                        let list = records.events[n].0.chunks(RECORD_LEN);
                        (event_key(n), list.map(StageRecordView::new).collect())
                    })
                    .collect(),
                rallies: (0..RALLIES)
                    .map(|n| {
                        let list = records.rallies[n].0.chunks(TIME_LEN);
                        (rally_key(n), list.map(BestTimeView::new).collect())
                    })
                    .collect(),
            },
            kept: o.kept.clone(),
        }
    }
}

/// The records of a list as the bytes of their table (`count` of them).
fn table_bytes<T>(
    list: &[T],
    count: usize,
    what: &str,
    bytes: impl Fn(&T) -> Result<Vec<u8>, OptionsError>,
) -> Result<Vec<u8>, OptionsError> {
    if list.len() != count {
        return Err(problem(format!(
            "{what} has {} entries, {count} wanted",
            list.len()
        )));
    }
    let mut out = Vec::new();
    for item in list {
        out.extend(bytes(item)?);
    }
    Ok(out)
}

impl Document {
    /// The options the document holds.
    pub fn into_options(self) -> Result<Options, OptionsError> {
        let mut o = Options {
            display: self.display,
            sound: self.sound,
            kept: self.kept,
            ..Options::default()
        };
        let n = self.network;
        o.network.machine_name = Bytes(buffer_of(&n.machine_name, 0x40, "machine_name")?);
        o.network.most_players = n.most_players;
        o.network.host_address = Bytes(buffer_of(&n.host_address, 0x40, "host_address")?);
        o.network.port = n.port;
        o.network.saved_connection = Bytes(guid_bytes(&n.saved_connection)?);
        let p = self.pages;
        o.pages.switches = Words(words_in::<11>(&p.switches, "pages.switches")?);
        o.pages.audio = Words(words_in::<6>(&p.audio, "pages.audio")?);
        o.pages.services = p.services;
        o.pages.event_kept = p.event_kept;
        o.pages.race_settings = p.race_settings;
        o.pages.arcade_levels_won =
            Bytes(exact::<6, u8>(&p.arcade_levels_won, "arcade_levels_won")?.to_vec());
        o.cars.first_numbers = Words(numbers_in::<40>(&self.cars.first_numbers, "first_numbers")?);
        o.cars.second_numbers = Words(numbers_in::<20>(
            &self.cars.second_numbers,
            "second_numbers",
        )?);
        let players = [
            self.players.player_1,
            self.players.player_2,
            self.players.player_3,
            self.players.player_4,
        ];
        for (n, player) in players.iter().enumerate() {
            let (record, name) = player.bytes()?;
            o.players.records[n] = Bytes(record);
            o.players.names[n] = Bytes(name);
            o.players.devices[n] = player.controller_device.clone();
        }
        let r = self.records;
        o.records.tyre_change_holder = Bytes(buffer_of(
            &r.tyre_change_holder,
            0x20,
            "tyre_change_holder",
        )?);
        o.records.tyre_change_time = r.tyre_change_time;
        o.records.championship = Bytes(table_bytes(
            &r.championship,
            CHAMPIONSHIP_RECORDS,
            "records.championship",
            StageRecordView::bytes,
        )?);
        for (n, table) in o.records.events.iter_mut().enumerate() {
            let key = event_key(n);
            let list = r
                .events
                .get(&key)
                .ok_or_else(|| problem(format!("records.events.{key} is missing")))?;
            *table = Bytes(table_bytes(
                list,
                EVENT_RECORDS,
                &format!("records.events.{key}"),
                StageRecordView::bytes,
            )?);
        }
        for (n, table) in o.records.rallies.iter_mut().enumerate() {
            let key = rally_key(n);
            let list = r
                .rallies
                .get(&key)
                .ok_or_else(|| problem(format!("records.rallies.{key} is missing")))?;
            *table = Bytes(table_bytes(
                list,
                RALLY_TIMES,
                &format!("records.rallies.{key}"),
                BestTimeView::bytes,
            )?);
        }
        Ok(o)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_guid_has_its_usual_text() {
        let bytes = [
            0xe0, 0x5e, 0xe9, 0x36, 0x77, 0x85, 0xcf, 0x11, 0x96, 0x0c, 0x00, 0x80, 0xc7, 0x53,
            0x4e, 0x82,
        ];
        let text = guid_text(&bytes);
        assert_eq!(text, "36E95EE0-8577-11CF-960C-0080C7534E82");
        assert_eq!(guid_bytes(&text).unwrap(), bytes);
        assert_eq!(
            guid_bytes(&format!("{{{}}}", text.to_lowercase())).unwrap(),
            bytes
        );
        assert!(guid_bytes("36E95EE0-8577-11CF-960C").is_err());
        assert!(guid_bytes("36E95EE0-8577-11CF-960C-0080C7534E8Z").is_err());
    }

    #[test]
    fn texts_stop_at_their_nul_and_keep_windows_1252() {
        assert_eq!(text_in(b"Ana\0junk"), "Ana");
        assert_eq!(text_in(&[0x4a, 0xe9, 0x80, 0]), "Jé€");
        assert_eq!(
            buffer_of("Jé€", 6, "t").unwrap(),
            [0x4a, 0xe9, 0x80, 0, 0, 0]
        );
        assert_eq!(buffer_of("abcd", 4, "t").unwrap(), b"abcd");
        assert!(buffer_of("abcde", 4, "t").is_err());
    }

    #[test]
    fn car_numbers_leave_out_their_nones() {
        assert_eq!(
            numbers_of(&[3, u32::MAX, 7, u32::MAX, u32::MAX]),
            [3, -1, 7]
        );
        assert_eq!(numbers_of(&[u32::MAX; 4]), Vec::<i64>::new());
        assert_eq!(
            numbers_in::<5>(&[3, -1, 7], "c").unwrap(),
            [3, u32::MAX, 7, u32::MAX, u32::MAX]
        );
        assert_eq!(numbers_in::<2>(&[4_294_967_295], "c").unwrap()[0], u32::MAX);
        assert!(numbers_in::<2>(&[1, 2, 3], "c").is_err());
        assert!(numbers_in::<2>(&[-5_000_000_000], "c").is_err());
    }

    #[test]
    fn floats_keep_their_bits() {
        for bits in [
            0u32,
            0x3F80_0000,
            0x3E99_999A,
            0x3DCC_CCCD,
            0x0000_0001,
            0x7F7F_FFFF,
        ] {
            assert_eq!(bits_of(float_of(bits)), bits);
        }
    }
}
