//! The race's sound set-up (`race::set_up_race_sound_objects`): the sound tables are made ready
//! once (`prepare_sound_tables_plain`: whole numbers made floats, spans, reciprocals, a sine
//! table) and the race's sounds are restarted or silenced.

use crate::errors::{BadInput, Refused};
use crate::memory::session::{PlainDword, PlainFloat};
use crate::sound::sets_store::SoundSet;

/// `race::whole_to_float`: makes the dword a float when it looks like a whole number (its top
/// byte 0 or 0xFF). Returns its top byte (the dword masked with 0xFF000000).
pub fn whole_to_float(at: &PlainDword) -> u32 {
    // One rounding only, the same at every precision.
    let (top, float) = rc2k_core::race::sound_tables::whole_to_float(at.get());
    if let Some(f) = float {
        at.set(f);
    }
    top
}

fn span_plain(second: u32, first: u32) -> u32 {
    rc2k_core::race::sound_tables::span(f32::from_bits(first), f32::from_bits(second)).to_bits()
}

/// `race::sound_span`: makes the two dwords `first` and `second` floats and sets `span` (the
/// dword after them) to their difference.
pub fn sound_span(first: &PlainDword, second: &PlainDword, span: &PlainDword) {
    whole_to_float(first);
    whole_to_float(second);
    span.set(span_plain(second.get(), first.get()));
}

/// `race::sound_row`: sets a `row`'s pitch and volume spans. The set's other three words are 0 in
/// every shipped set and read by no sound code, so they are left alone.
pub fn sound_row(row: &SoundSet) {
    sound_span(&row.pitch_low, &row.pitch_high, &row.pitch_scale);
    sound_span(&row.volume_low, &row.volume_high, &row.volume_scale);
}

/// Where `prepare_sound_tables_plain` finds its variables (the live ones, or a test's).
#[derive(Clone, Copy)]
pub struct TableVars<'a> {
    /// `SurfaceSounds.tables_done`: 1 once done.
    pub done: &'a PlainDword,
    /// `SoundKinds.sines`: 0x40 floats.
    pub sines: &'a [PlainFloat; 64],
    /// The kinds' records.
    pub kinds: &'a crate::sound::sets_store::SoundKinds,
    /// `Machine.scratch`: left holding the last angle of the sine table.
    pub scratch: &'a PlainDword,
}

impl<'a> TableVars<'a> {
    pub fn of(
        sound: &'a crate::sound::owner::Sound,
        system: &'a crate::system::owner::System,
    ) -> Self {
        Self {
            done: &sound.race.surface.get().tables_done,
            sines: &sound.race.kinds.get().sines,
            kinds: sound.race.kinds.get(),
            scratch: &system.machine().scratch,
        }
    }
}

/// The number of sound kind records.
pub const RECORDS: u32 = 0x22;

/// Prepares the sound tables, run once (`SurfaceSounds.tables_done`): each of the 0x22 records
/// (`SoundKinds.kinds`) gets its rev span (`sound_span`), turned into its reciprocal, and its
/// three rows (`sound_row`). Then the 0x40 floats of `sines` are set to 1 + 0.05 sin(x) for x from
/// 0 by pi / 32, and `Machine.scratch` keeps the last x.
pub fn prepare_sound_tables_plain(v: &TableVars<'_>) -> Result<(), BadInput> {
    use rc2k_core::race::sound_tables::{reciprocal, sines};
    if v.done.get() == 1 {
        return Ok(());
    }
    v.done.set(1);
    for n in 0..RECORDS {
        let record = v.kinds.record(n)?;
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
    v.scratch.set(last.to_bits());
    Ok(())
}

/// `race::restart_race_sounds`: waits for the commands, resets the race's sounds, lets them play
/// and sets `RaceClock.started` to 1.
pub fn restart_race_sounds(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    use crate::race::race_end::{SoundVars, reset_race_sounds, wait_for_commands};
    let (sound, audio) = (o.sound, o.audio);
    wait_for_commands(o);
    reset_race_sounds(&SoundVars::game(sound), o)?;
    crate::sound::let_sounds_play::let_sounds_play_in(audio);
    sound.race.clock.get().started.set(1);
    Ok(())
}

/// `race::silence_race_sounds`: resets the race's sounds, sends the free command and waits for
/// the commands.
pub fn silence_race_sounds(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    use crate::race::race_end::{SoundVars, reset_race_sounds, wait_for_commands};
    let audio = o.audio;
    reset_race_sounds(&SoundVars::game(o.sound), o)?;
    crate::sound::queue::send_command_in(audio, rc2k_core::audio::commands::Command::Free);
    wait_for_commands(o);
    Ok(())
}

/// Where `set_up_race_sound_objects` finds the eight objects (the live ones, or a test's).
#[derive(Clone, Copy)]
pub struct ObjectVars<'a> {
    pub objects: &'a [crate::sound::store::SoundObject; 8],
}

impl<'a> ObjectVars<'a> {
    pub fn of(sound: &'a crate::sound::owner::Sound) -> Self {
        Self {
            objects: &sound.race.sounds.get().objects,
        }
    }
}

/// An object's set-up: the place, the motion and the four wheel records cleared, and the other
/// fields cleared (0) or set (1).
fn set_up_object(o: &crate::sound::store::SoundObject) {
    for f in o.place.iter().chain(&o.motion) {
        f.set(0.0);
    }
    for v in &o.voices {
        v.share_a.set(0.0);
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
        .chain(&o.unused_a4)
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
    o.last_forward_speed.set(0);
    o.unused_280.set(0);
    o.state.set(0);
}

/// `race::set_up_race_sound_objects`: prepares the sound tables, then sets the players' count
/// (`RaceSounds.one_player`: 1, or 2 when `RaceState.settings.race_mode` is not 1) and the
/// objects' count (`RaceSounds.count`: that, or 4 in arcade mode with `RaceSounds.mark_car_voices`
/// set). The players' objects are `RaceSounds.player_cars` (0 and 1, or the chosen car
/// `NetShared.player` in a network game, which also gives the count `NetShared.players`). Each
/// object gets its sound offset, the players' objects their number, kind and co-driver, and every
/// object is reset; `TyreChange.drawn` is 0 and the sounds are restarted
/// (`restart_race_sounds`). Refuses, before anything, a chosen car past 7.
pub fn set_up_race_sound_objects(
    o: crate::owners::Owners<'static>,
    v: &ObjectVars<'_>,
) -> Result<(), Refused> {
    let (race, sound, system) = (o.race, o.sound, o.system);
    let shared = &race.shared;
    {
        let net_race = o.net.race_slot.get();
        if net_race.is_some_and(|n| n.player > 7) {
            return Err(crate::errors::BadInput("a chosen car past the eight").into());
        }
        use rc2k_core::race::sound_objects::{object_mark, sound_objects, sound_players};
        prepare_sound_tables_plain(&TableVars::of(sound, system))?;
        let players = sound_players(shared.state.settings.race_mode.get());
        sound.threads.shared.one_player.set(players);
        let (count, arcade) = sound_objects(players, shared.state.settings.run_kind.get());
        if arcade {
            sound.race.sounds.get().mark_car_voices.set(1);
        }
        sound.threads.shared.count.set(count);
        sound.race.sounds.get().player_cars[0].set(0);
        sound.race.sounds.get().player_cars[1].set(1);
        if let Some(net_race) = net_race {
            sound.race.sounds.get().player_cars[0].set(net_race.player);
            sound.threads.shared.count.set(net_race.players);
        }
        for (n, o) in v.objects.iter().enumerate() {
            o.sound_offset.set(object_mark(n as u32));
        }
        let first = crate::sound::store::SoundObject::of(
            sound,
            sound.race.sounds.get().player_cars[0].get(),
        )?;
        first.sound_offset.set(0);
        first.kind.set(shared.state.settings.car_kind_current.get());
        first
            .cue_queue
            .first
            .set(shared.state.settings.cars[0].codriver_sex.get());
        if sound.threads.shared.one_player.get() != 1 {
            let second = crate::sound::store::SoundObject::of(
                sound,
                sound.race.sounds.get().player_cars[1].get(),
            )?;
            second.sound_offset.set(1);
            second.kind.set(shared.state.settings.car_kind_first.get());
            second
                .cue_queue
                .first
                .set(shared.state.settings.cars[1].codriver_sex.get());
        }
        for o in v.objects {
            set_up_object(o);
        }
        shared.tyre_change().drawn.set(0);
        restart_race_sounds(o)?;
    }
    Ok(())
}

/// `race::clear_race_sound_objects`: silences the sounds and sets `RaceSounds.one_player`,
/// `RaceSounds.count` and `RaceSounds.mark_car_voices` to 0.
pub fn clear_race_sound_objects(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let sound = o.sound;
    silence_race_sounds(o)?;
    sound.threads.shared.one_player.set(0);
    sound.threads.shared.count.set(0);
    sound.race.sounds.get().mark_car_voices.set(0);
    Ok(())
}
