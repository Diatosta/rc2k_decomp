//! The race's sound set-up (`sub_415C93`, IDA 2026-10-17): the sound tables made ready
//! once (`race::prepare_sound_tables`: whole numbers made floats, spans, reciprocals, a
//! sine table) and the race's sounds restarted or silenced. Here the steps.

use crate::errors::{BadInput, Refused};
use crate::race::state_store::RACE_STATE;
use crate::race::tyre_store::TYRE_CHANGE;
use crate::session::{Dword, Float};
use crate::sound::sets_store::SoundSet;

/// `race::whole_to_float` (0x40D706): the dword at ebx made a float (`fild` / `fstp`)
/// when it looks like a whole number (its top byte 0 or 0xFF). Leaves eax its top byte's
/// mask (`and 0xFF000000`).
pub fn whole_to_float(at: &Dword) -> u32 {
    // Converted: `rc2k_core::race::sound_tables::whole_to_float` (one rounding: the same
    // at every precision, so no exact twin).
    let (top, float) = rc2k_core::race::sound_tables::whole_to_float(at.get());
    if let Some(f) = float {
        at.set(f);
    }
    top
}

fn span_plain(second: u32, first: u32) -> u32 {
    rc2k_core::race::sound_tables::span(f32::from_bits(first), f32::from_bits(second)).to_bits()
}

/// `race::sound_span` (0x40D71B), converted: the two dwords `first` and `second` made
/// floats, `span` (the dword after them) their difference (`fld` / `fsub` / `fstp`). Its
/// leftovers (eax, ebx) are read by no caller (`prepare_sound_tables` and `sound_row` only).
// Converted: `rc2k_core::race::sound_tables::span`.
pub fn sound_span(first: &Dword, second: &Dword, span: &Dword) {
    whole_to_float(first);
    whole_to_float(second);
    span.set(span_plain(second.get(), first.get()));
}

/// `race::sound_row` (0x40D731), converted: a set `row`: +4, +8, +0xC made floats, spans
/// at +0x10 and +0x1C. Its leftovers are read by no caller.
// Converted: `rc2k_core::race::sound_tables::{whole_to_float, span}`.
pub fn sound_row(row: &SoundSet) {
    for cell in [&row.unknown_04, &row.unknown_08, &row.unknown_0c] {
        whole_to_float(cell);
    }
    sound_span(&row.pitch_low, &row.pitch_high, &row.pitch_scale);
    sound_span(&row.volume_low, &row.volume_high, &row.volume_scale);
}

/// Where `prepare_sound_tables` finds its variables (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct TableVars {
    /// dword_516358: 1 once done.
    pub done: &'static Dword,
    /// unk_516258: 0x40 floats.
    pub sines: &'static [Float; 64],
    /// dword_5183CC: the scratch.
    pub scratch: &'static Dword,
}

impl TableVars {
    pub fn game() -> Self {
        Self {
            done: &crate::sound::owner::sound().race.surface.tables_done,
            sines: &crate::sound::owner::sound().race.kinds.sines,
            scratch: &crate::session::MACHINE.scratch,
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
    if v.done.get() == 1 {
        return Ok(());
    }
    v.done.set(1);
    for n in 0..RECORDS {
        let record = crate::sound::sets_store::SoundKinds::record(n)?;
        sound_span(&record.revs_low, &record.revs_high, &record.revs_scale);
        record
            .revs_scale
            .set(reciprocal(f32::from_bits(record.revs_scale.get())).to_bits());
        for set in [&record.revs, &record.throttle, &record.speed] {
            sound_row(set);
        }
    }
    let (table, last) = sines();
    for (k, f) in table.iter().enumerate() {
        v.sines[k].set(*f);
    }
    // The scratch: 0x20, the step, then each x: the last left.
    v.scratch.set(last.to_bits());
    Ok(())
}

/// `race::restart_race_sounds`: the commands waited for, the race's sounds reset, let
/// play, `dword_5124E4` 1. Every register kept.
pub fn restart_race_sounds() -> Result<(), Refused> {
    use crate::race::race_end::{SoundVars, reset_race_sounds, wait_for_commands};
    wait_for_commands();
    reset_race_sounds(&SoundVars::game())?;
    crate::sound::let_sounds_play::let_sounds_play();
    crate::sound::owner::sound().race.clock.started.set(1);
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

/// Where `set_up_race_sound_objects` finds the eight objects (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct ObjectVars {
    pub objects: &'static [crate::sound::store::SoundObject; 8],
}

impl ObjectVars {
    pub fn game() -> Self {
        Self {
            objects: &crate::sound::owner::sound().race.sounds.objects,
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
/// table)
pub fn set_up_race_sound_objects(v: &ObjectVars) -> Result<(), Refused> {
    {
        let one = crate::net::store::NET.session.get() == 1;
        if one && crate::net::store::NET.player.get() > 7 {
            return Err(crate::errors::BadInput("a chosen car past the eight").into());
        }
        // Converted: `rc2k_core::race::sound_objects::{sound_players, sound_objects,
        // object_mark}`; the tables through `prepare_sound_tables_plain`.

        use rc2k_core::race::sound_objects::{object_mark, sound_objects, sound_players};
        prepare_sound_tables_plain(&TableVars::game())?;
        let players = sound_players(RACE_STATE.settings.race_mode.get());
        crate::sound::owner::sound()
            .race
            .sounds
            .one_player
            .set(players);
        let (count, arcade) = sound_objects(players, RACE_STATE.settings.run_kind.get());
        if arcade {
            crate::sound::owner::sound()
                .race
                .sounds
                .mark_car_voices
                .set(1);
        }
        crate::sound::owner::sound().race.sounds.count.set(count);
        crate::sound::owner::sound().race.sounds.player_cars[0].set(0);
        crate::sound::owner::sound().race.sounds.player_cars[1].set(1);
        if one {
            crate::sound::owner::sound().race.sounds.player_cars[0]
                .set(crate::net::store::NET.player.get());
            crate::sound::owner::sound()
                .race
                .sounds
                .count
                .set(crate::net::store::NET.players.get());
        }
        for (n, o) in v.objects.iter().enumerate() {
            o.sound_offset.set(object_mark(n as u32));
        }
        let first = crate::sound::store::SoundObject::number(
            crate::sound::owner::sound().race.sounds.player_cars[0].get(),
        )?;
        first.sound_offset.set(0);
        first.kind.set(RACE_STATE.settings.car_kind_current.get());
        first
            .cue_queue
            .first
            .set(RACE_STATE.settings.cars[0].codriver_sex.get());
        if crate::sound::owner::sound().race.sounds.one_player.get() != 1 {
            let second = crate::sound::store::SoundObject::number(
                crate::sound::owner::sound().race.sounds.player_cars[1].get(),
            )?;
            second.sound_offset.set(1);
            second.kind.set(RACE_STATE.settings.car_kind_first.get());
            second
                .cue_queue
                .first
                .set(RACE_STATE.settings.cars[1].codriver_sex.get());
        }
        for o in v.objects {
            set_up_object(o);
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
    let sounds = &crate::sound::owner::sound().race.sounds;
    for d in [&sounds.one_player, &sounds.count, &sounds.mark_car_voices] {
        d.set(0);
    }
    Ok(())
}
