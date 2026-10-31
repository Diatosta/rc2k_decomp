//! The race's sound set-up (`sub_415C93`, IDA 2026-10-17): the sound tables made ready
//! once (`race::prepare_sound_tables`: whole numbers made floats, spans, reciprocals, a
//! sine table) and the race's sounds restarted or silenced. Here the steps.

use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::race::state_store::RACE_STATE;
use crate::race::tyre_store::TYRE_CHANGE;
use crate::sound::sets_store::SOUND_KINDS;

fn get(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read("a sound table word")
}

fn put(at: u32, v: u32) -> Result<(), BadInput> {
    GamePtr::new(at).write(v, "a sound table word")
}

/// `race::whole_to_float` (0x40D706): the dword at ebx made a float (`fild` / `fstp`)
/// when it looks like a whole number (its top byte 0 or 0xFF). Leaves eax its top byte's
/// mask (`and 0xFF000000`).
pub fn whole_to_float(at: u32) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::race::sound_tables::whole_to_float` (one rounding: the same
    // at every precision, so no exact twin).
    let (top, float) = rc2k_core::race::sound_tables::whole_to_float(get(at)?);
    if let Some(f) = float {
        put(at, f)?;
    }
    Ok(top)
}

fn span_plain(second: u32, first: u32) -> u32 {
    rc2k_core::race::sound_tables::span(f32::from_bits(first), f32::from_bits(second)).to_bits()
}

/// `race::sound_span` (0x40D71B), converted: the two dwords at `at` made floats, the third
/// (at `at` + 8) their difference (`fld` / `fsub` / `fstp`). Its leftovers (eax, ebx) are
/// read by no caller (`prepare_sound_tables` and `sound_row` only).
// Converted: `rc2k_core::race::sound_tables::span`.
pub fn sound_span(at: u32) -> Result<(), BadInput> {
    whole_to_float(at)?;
    whole_to_float(at + 4)?;
    put(at + 8, span_plain(get(at + 4)?, get(at)?))
}

/// `race::sound_row` (0x40D731), converted: a row at `row`: +4, +8, +0xC made floats, spans
/// at +0x10 and +0x1C. Its leftovers are read by no caller.
// Converted: `rc2k_core::race::sound_tables::{whole_to_float, span}`.
pub fn sound_row(row: u32) -> Result<(), BadInput> {
    for off in [4, 8, 0xC] {
        whole_to_float(row + off)?;
    }
    sound_span(row + 0x10)?;
    sound_span(row + 0x1C)
}

/// Where `prepare_sound_tables` finds its variables (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct TableVars {
    /// dword_516358: 1 once done.
    pub done: u32,
    /// unk_516258: 0x40 floats.
    pub sines: u32,
    /// dword_5183CC: the scratch.
    pub scratch: u32,
}

impl TableVars {
    pub fn game() -> Self {
        Self {
            done: crate::sound::store::SURFACE_SOUNDS.tables_done.addr(),
            sines: crate::session::address_of(&SOUND_KINDS.sines),
            scratch: crate::session::MACHINE.scratch.addr(),
        }
    }
}

pub const RECORDS: u32 = 0x22;
/// `race::prepare_sound_tables` (0x40D636), converted (`set_up_race_sound_objects`'s too): once (`dword_516358`): each of the 0x22 records
/// (`off_514F90`) its span at +4 (`sound_span`), +0xC made its reciprocal (`fld1` /
/// `fdiv`), its three rows (`sound_row`, 0x28 apart from +0x10); then the 0x40 floats at
/// 0x516258 1 + 0.05 sin(x) for x from 0 by pi / 32 (the scratch `dword_5183CC` left the
/// last x). Every register kept.
// Converted: `rc2k_core::race::sound_tables::{reciprocal, sines}`.
pub fn prepare_sound_tables_plain(v: &TableVars) -> Result<(), BadInput> {
    use rc2k_core::race::sound_tables::{reciprocal, sines};
    if get(v.done)? == 1 {
        return Ok(());
    }
    put(v.done, 1)?;
    for n in 0..RECORDS {
        let record = crate::sound::sets_store::SoundKinds::record_of(n)?;
        sound_span(record + 4)?;
        put(
            record + 0xC,
            reciprocal(f32::from_bits(get(record + 0xC)?)).to_bits(),
        )?;
        for k in 0..3 {
            sound_row(record + 0x10 + 0x28 * k)?;
        }
    }
    let (table, last) = sines();
    for (k, f) in table.iter().enumerate() {
        put(v.sines + 4 * k as u32, f.to_bits())?;
    }
    // The scratch: 0x20, the step, then each x: the last left.
    put(v.scratch, last.to_bits())
}

/// `race::restart_race_sounds`: the commands waited for, the race's sounds reset, let
/// play, `dword_5124E4` 1. Every register kept.
pub fn restart_race_sounds() -> Result<(), Refused> {
    use crate::race::race_end::{SoundVars, reset_race_sounds, wait_for_commands};
    wait_for_commands();
    reset_race_sounds(&SoundVars::game())?;
    crate::sound::let_sounds_play::let_sounds_play();
    crate::sound::store::RACE_CLOCK.started.set(1);
    Ok(())
}

/// `race::silence_race_sounds` (0x40D366): the race's sounds reset, command 4, the
/// commands waited for. Every register kept.
pub fn silence_race_sounds() -> Result<(), Refused> {
    use crate::race::race_end::{SoundVars, command, reset_race_sounds, wait_for_commands};
    reset_race_sounds(&SoundVars::game())?;
    command(4, 0);
    wait_for_commands();
    Ok(())
}

// ---- the race's sound objects ----

/// Where `set_up_race_sound_objects` finds the objects: the pointer table
/// (`g_raceSoundObjects`, 0x512A10) and the eight objects it names, also written by their
/// own addresses (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct ObjectVars {
    pub table: u32,
    pub objects: [u32; 8],
}

impl ObjectVars {
    pub fn game() -> Self {
        Self {
            table: crate::pointers::game_address(crate::sound::store::RACE_SOUNDS.table.as_ptr()),
            objects: std::array::from_fn(|k| crate::sound::store::RACE_SOUNDS.objects[k].addr()),
        }
    }
}

/// An object's set-up: the 0x68 bytes from +0x2C (the place, the motion and the four
/// wheel records) and 24 fields cleared (0) or set (1).
fn set_up_object(o: &crate::sound::store::SoundObject) {
    for f in o.place.iter().chain(&o.motion) {
        f.set(0.0);
    }
    for v in &o.voices {
        v.share_a.set(0.0);
        v._gap_4.set(0);
        v.share_b.set(0.0);
        v.share_c.set(0.0);
        v.surface.set(0);
    }
    o.new_revs.set(0);
    o.in_use.set(0);
    o.view_flag.set(1);
    o.revs.set(0);
    o.throttle.set(0);
    o.speed.set(0.0);
    o.gear_event.set(0);
    for d in o
        .reading_times
        .iter()
        .chain(&o.reading_revs)
        .chain(&o.cleared_a4)
    {
        d.set(0);
    }
    o.no_gear.set(1);
    o.gear_notice.set(0);
    o.countdown.set(0);
    o.knock.set(0);
    o.knock_count.set(0);
    o.flag_ac.set(0);
    o.landed.set(0);
    o.last_effect_bc.set(0);
    o.cleared_280.set(0);
    o.state.set(0);
}

/// `race::set_up_race_sound_objects` (0x415C93): `prepare_sound_tables`; the players'
/// count (`dword_514F78`: 1, 2 with `dword_75ACE0` not 1), the objects' count
/// (`dword_514F7C`: that, 4 in mode 2 with `dword_514F8C` 1), the players' objects
/// (`dword_514F70` 0 or the chosen car `dword_52A920` with `dword_52A910` 1, which also
/// gives the count `dword_52AD98`; `dword_514F74` 1); each object's first dword -1 .. -8;
/// the players' objects their number, kind and setting; each object's 0x68 bytes from
/// +0x2C and 24 fields reset; `dword_5BEBE8` 0; `restart_race_sounds`. Every register
/// kept. Refuses, before anything, a chosen car past 7 (the original reads past the
/// table) or an object missing.
pub fn set_up_race_sound_objects(v: &ObjectVars) -> Result<(), Refused> {
    {
        let one = crate::net::store::NET.session.get() == 1;
        if one && crate::net::store::NET.player.get() > 7 {
            return Err(crate::errors::BadInput("a chosen car past the eight").into());
        }
        let objects: Vec<u32> = (0..8)
            .map(|n| get(v.table + 4 * n))
            .collect::<Result<_, _>>()?;
        if objects.contains(&0) {
            return Err(crate::errors::BadInput("a race sound object missing").into());
        }
        // Converted: `rc2k_core::race::sound_objects::{sound_players, sound_objects,
        // object_mark}`; the tables through `prepare_sound_tables_plain`.
        use crate::sound::store::RACE_SOUNDS;
        use rc2k_core::race::sound_objects::{object_mark, sound_objects, sound_players};
        prepare_sound_tables_plain(&TableVars::game())?;
        let players = sound_players(RACE_STATE.settings.race_mode.get());
        RACE_SOUNDS.one_player.set(players);
        let (count, arcade) = sound_objects(players, RACE_STATE.settings.run_kind.get());
        if arcade {
            RACE_SOUNDS.mark_car_voices.set(1);
        }
        RACE_SOUNDS.count.set(count);
        RACE_SOUNDS.player_cars[0].set(0);
        RACE_SOUNDS.player_cars[1].set(1);
        if one {
            RACE_SOUNDS.player_cars[0].set(crate::net::store::NET.player.get());
            RACE_SOUNDS.count.set(crate::net::store::NET.players.get());
        }
        for (n, &o) in v.objects.iter().enumerate() {
            crate::sound::store::SoundObject::at(o)?
                .sound_offset
                .set(object_mark(n as u32));
        }
        let first = crate::sound::store::SoundObject::at(
            objects[RACE_SOUNDS.player_cars[0].get() as usize],
        )?;
        first.sound_offset.set(0);
        first.kind.set(RACE_STATE.settings.car_kind_current.get());
        first
            .cue_queue
            .first
            .set(RACE_STATE.settings.cars[0].codriver_sex.get());
        if RACE_SOUNDS.one_player.get() != 1 {
            let second = crate::sound::store::SoundObject::at(
                objects[RACE_SOUNDS.player_cars[1].get() as usize],
            )?;
            second.sound_offset.set(1);
            second.kind.set(RACE_STATE.settings.car_kind_first.get());
            second
                .cue_queue
                .first
                .set(RACE_STATE.settings.cars[1].codriver_sex.get());
        }
        for &o in &objects {
            set_up_object(crate::sound::store::SoundObject::at(o)?);
        }
        TYRE_CHANGE.drawn.set(0);
        restart_race_sounds()?;
    }
    Ok(())
}

/// `race::clear_race_sound_objects` (0x415C6D): `silence_race_sounds`; `dword_514F78`,
/// `dword_514F7C`, `dword_514F8C` 0. Every register kept.
pub fn clear_race_sound_objects() -> Result<(), Refused> {
    silence_race_sounds()?;
    let sounds = &crate::sound::store::RACE_SOUNDS;
    for d in [&sounds.one_player, &sounds.count, &sounds.mark_car_voices] {
        d.set(0);
    }
    Ok(())
}
