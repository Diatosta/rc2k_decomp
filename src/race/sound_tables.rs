//! The race's sound set-up (`race::set_up_race_sound_objects`): the sound tables are made ready
//! once (`prepare_sound_tables_plain`: whole numbers made floats, spans, reciprocals, a sine
//! table) and the race's sounds are restarted or silenced.

use crate::errors::{BadInput, Refused};
use crate::memory::session::PlainDword;
use crate::sound::sets_store::SoundSet;

/// Makes the dword a float when it looks like a whole number (its top byte 0 or 0xFF). Returns
/// its top byte (the dword masked with 0xFF000000).
pub fn whole_to_float(at: &mut u32) -> u32 {
    // One rounding only, the same at every precision.
    let (top, float) = rc2k_core::race::sound_tables::whole_to_float(*at);
    if let Some(f) = float {
        *at = f;
    }
    top
}

fn span_plain(second: u32, first: u32) -> u32 {
    rc2k_core::race::sound_tables::span(f32::from_bits(first), f32::from_bits(second)).to_bits()
}

/// Makes the two dwords `first` and `second` floats and sets `span` to their difference.
fn sound_span(first: &mut u32, second: &mut u32, span: &mut u32) {
    whole_to_float(first);
    whole_to_float(second);
    *span = span_plain(*second, *first);
}

/// Sets a `row`'s pitch and volume spans. The set's other three words are 0 in every shipped set
/// and read by no sound code, so they are left alone.
fn sound_row(row: &mut SoundSet) {
    sound_span(
        &mut row.pitch_low,
        &mut row.pitch_high,
        &mut row.pitch_scale,
    );
    sound_span(
        &mut row.volume_low,
        &mut row.volume_high,
        &mut row.volume_scale,
    );
}

/// Where `prepare_sound_tables_plain` finds its variables (the live ones, or a test's).
#[derive(Clone, Copy)]
struct TableVars<'a> {
    /// `SurfaceSounds.tables_done`: 1 once done.
    pub done: &'a std::cell::RefCell<crate::sound::store::SurfaceWork>,
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
            done: &sound.race.surface.get().work,
            kinds: sound.race.kinds.get(),
            scratch: &system.machine().scratch,
        }
    }
}

/// The number of sound kind records.
pub const RECORDS: u32 = 0x22;

/// Prepares the sound tables, run once (`SurfaceSounds.tables_done`): each of the 0x22 records
/// (`SoundKinds.kinds`) gets its rev span (`sound_span`), turned into its reciprocal, and its
/// three rows (`sound_row`). Then the sine table's last angle is left in `Machine.scratch` (the
/// table itself, 1 + 0.05 sin(x) for x from 0 by pi / 32, is read by nothing and not kept).
fn prepare_sound_tables_plain(v: &TableVars<'_>) -> Result<(), BadInput> {
    use rc2k_core::race::sound_tables::{reciprocal, sines};
    if v.done.borrow().tables_done == 1 {
        return Ok(());
    }
    v.done.borrow_mut().tables_done = 1;
    for n in 0..RECORDS {
        v.kinds.update_record(n, |record| {
            sound_span(
                &mut record.revs_low,
                &mut record.revs_high,
                &mut record.revs_scale,
            );
            record.revs_scale = reciprocal(f32::from_bits(record.revs_scale)).to_bits();
            for set in [&mut record.revs, &mut record.throttle, &mut record.speed] {
                sound_row(set);
            }
        })?;
    }
    let (_, last) = sines();
    v.scratch.set(last.to_bits());
    Ok(())
}

/// `race::restart_race_sounds`: waits for the commands, resets the race's sounds, lets them play
/// and sets `RaceClock.started` to 1.
fn restart_race_sounds(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    use crate::race::race_end::{SoundVars, reset_race_sounds, wait_for_commands};
    let (sound, audio) = (o.sound, o.audio);
    wait_for_commands(o);
    reset_race_sounds(&SoundVars::game(sound), o)?;
    crate::sound::let_sounds_play::let_sounds_play_in(audio);
    sound.race.clock.get().borrow_mut().started = 1;
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
    pub objects: &'a [crate::sound::store::SoundObjectCell; 8],
}

impl<'a> ObjectVars<'a> {
    pub fn of(sound: &'a crate::sound::owner::Sound) -> Self {
        Self {
            objects: sound.race.objects.get(),
        }
    }
}

/// An object's set-up: the place, the motion and the four wheel records cleared, and the other
/// fields cleared (0) or set (1).
fn set_up_object(o: &crate::sound::store::SoundObjectCell) {
    let mut o = o.borrow_mut();
    o.place = [0.0; 3];
    o.motion = [0.0; 3];
    for v in &mut o.voices {
        v.share_a = 0.0;
        v.share_b = 0.0;
        v.share_c = 0.0;
        v.surface = 0;
    }
    o.new_revs = 0;
    o.in_use = 0;
    o.view_flag = 1;
    o.revs = 0;
    o.throttle = 0;
    o.speed = 0.0;
    o.gear_event = 0;
    o.reading_times = [0; 2];
    o.reading_revs = [0; 2];
    o.reserved_a4 = [0; 2];
    o.no_gear = 1;
    o.gear_notice = 0;
    o.countdown = 0;
    o.knock = 0;
    o.knock_count = 0;
    o.flag_ac = 0;
    o.landed = 0;
    o.last_forward_speed = 0;
    o.reserved_280 = 0;
    o.state = 0;
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
    set_up_objects(o, v, false, None)
}

/// `set_up_race_sound_objects` for the replay viewer: with `both_cars` (a replay with two cars or
/// more that the header calls a one-player race, as a network race's is) the first two cars have
/// the sound objects of a two-player race, so the second car can be heard when it is the one
/// shown. With `every_car` (the car models of a replay of more than two cars) each car has a
/// player's sound object, so that every car is heard.
pub(super) fn set_up_viewer_race_sound_objects(
    o: crate::owners::Owners<'static>,
    v: &ObjectVars<'_>,
    both_cars: bool,
    every_car: Option<&[u32]>,
) -> Result<(), Refused> {
    set_up_objects(o, v, both_cars, every_car)
}

fn set_up_objects(
    o: crate::owners::Owners<'static>,
    v: &ObjectVars<'_>,
    both_cars: bool,
    every_car: Option<&[u32]>,
) -> Result<(), Refused> {
    let (race, sound, system) = (o.race, o.sound, o.system);
    let shared = &race.shared;
    crate::race::heard_cars::set(shared, every_car.is_some());
    {
        let net_race = o.net.race_slot.get();
        if net_race.is_some_and(|n| n.player > 7) {
            return Err(crate::errors::BadInput("a chosen car past the eight").into());
        }
        use rc2k_core::race::sound_objects::{
            heard_players, object_mark, sound_objects, sound_players,
        };
        prepare_sound_tables_plain(&TableVars::of(sound, system))?;
        let players = if both_cars {
            2
        } else {
            sound_players(shared.state.settings.race_mode.get())
        };
        sound.threads.shared.one_player.set(players);
        // The split screen's third and fourth cars have objects of their own, as the first two.
        let heard = if let Some(kinds) = every_car {
            kinds.len().clamp(2, v.objects.len()) as u32
        } else if both_cars {
            2
        } else {
            heard_players(shared.state.settings.race_mode.get())
        };
        let (count, arcade) = if every_car.is_some() {
            // Every car has its own object, none shares the computers' two voices.
            (heard, false)
        } else {
            sound_objects(heard, shared.state.settings.run_kind.get())
        };
        // The co-driver speaks to each player of a split screen of three or four (not in the
        // arcade, whose objects past the second are the computers' cars', nor in a replay, whose
        // extra cars have no settings of their own to say what they would hear).
        crate::sound::cue_players::set(
            if every_car.is_none() && !both_cars && !arcade && heard > 2 {
                heard
            } else {
                2
            },
        );
        if arcade {
            sound.race.sounds_mut().mark_car_voices = 1;
        }
        sound.threads.shared.count.set(count);
        sound.race.sounds_mut().player_cars = [0, 1];
        if let Some(net_race) = net_race {
            sound.race.sounds_mut().player_cars[0] = net_race.player;
            sound
                .threads
                .shared
                .count
                .set(net_race.players + net_race.bots);
        }
        for (n, o) in v.objects.iter().enumerate() {
            o.borrow_mut().sound_offset = object_mark(n as u32);
        }
        let player_cars = sound.race.sounds().player_cars;
        let first = crate::sound::store::SoundObject::of(sound, player_cars[0])?;
        first.borrow_mut().sound_offset = 0;
        first.borrow_mut().kind = shared.state.settings.car_kind_current.get();
        first.borrow_mut().cue_queue.first = shared.state.settings.cars[0].codriver_sex.get();
        if sound.threads.shared.one_player.get() != 1 {
            let second = crate::sound::store::SoundObject::of(sound, player_cars[1])?;
            second.borrow_mut().sound_offset = 1;
            second.borrow_mut().kind = shared.state.settings.car_kind_first.get();
            second.borrow_mut().cue_queue.first = shared.state.settings.cars[1].codriver_sex.get();
        }
        for k in 2..heard as usize {
            let extra = crate::sound::store::SoundObject::of(sound, k as u32)?;
            extra.borrow_mut().sound_offset = k as u32;
            if let Some(kinds) = every_car {
                // The replay's cars past the first two have no settings of their own: their
                // models are the file's, their co-driver is not heard.
                extra.borrow_mut().kind = kinds[k];
                extra.borrow_mut().cue_queue.first = 0;
            } else {
                extra.borrow_mut().kind = shared.state.car_kind(k).get();
                extra.borrow_mut().cue_queue.first = shared.state.car(k).codriver_sex.get();
            }
        }
        for o in v.objects {
            set_up_object(o);
        }
        shared.tyre_change_mut().drawn = 0;
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
    sound.race.sounds_mut().mark_car_voices = 0;
    Ok(())
}
