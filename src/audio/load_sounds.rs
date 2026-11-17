//! Command 3: loads a list of sounds. For each sound whose record has a file and a copy count
//! above 0, that many voices are made (while fewer than 0x200 are in use): found shared voice or
//! sample read, new voice, `fill_voice`, sample mode, prepare, keep, make and fill buffer; the
//! voice's flags get `LOADED`. Quitting ends it after any entry. Refuses, before anything is
//! done: a list without its -1 within 0x1000 entries, and a sound of 0x1CD or more.

use crate::audio::own::AudioThread;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::sound::sets_store::{MenuSounds, SoundSets};
use crate::system::owner::System;
use rc2k_core::audio::commands::SoundList;

struct GameLoader<'a> {
    audio: &'a AudioThread,
    system: &'a System,
}

impl<'a> GameLoader<'a> {
    /// The record, which `make_buffer` takes as its holder value.
    fn record_at(&self, sound: u32) -> crate::audio::property_sets::Holder {
        crate::audio::property_sets::Holder::Record(
            self.audio
                .sample_records
                .record(sound)
                .expect("a sound inside the records")
                .number,
        )
    }
}

impl rc2k_core::audio::load::Loader for GameLoader<'_> {
    type Error = Refused;
    fn record(&mut self, sound: u32) -> SampleRecord {
        self.audio
            .sample_records
            .record(sound)
            .expect("a sound inside the records")
    }
    fn set_first_voice(&mut self, sound: u32, voice: u32) {
        self.audio
            .sample_records
            .set_first_voice(sound as usize, voice);
    }
    fn full(&mut self) -> bool {
        self.audio.state.voice_count.get() == VOICES
    }
    fn shared(&mut self, at: rc2k_core::audio::load::At) -> Result<u32, Refused> {
        let audio = self.audio;
        let count = audio.state.voice_count.get();
        Ok({
            crate::audio::samples::find_shared_voice(
                audio,
                &audio
                    .sample_records
                    .record(at.sound)
                    .expect("a sound inside the records"),
                count.min(VOICES),
            )
        })
    }
    fn read(&mut self, at: rc2k_core::audio::load::At) -> Result<bool, Refused> {
        use crate::audio::{read_sample_of, samples};
        read_sample_of::read_sample_of(
            &self
                .audio
                .sample_records
                .record(at.sound)
                .expect("a sound inside the records"),
            &mut |p| {
                samples::read_sample_in_game(
                    &self.audio.state,
                    &crate::audio::store::name_in(&self.audio.sample_names, p)?,
                )
            },
        )
    }
    fn new_voice(&mut self, _: rc2k_core::audio::load::At, _: u32) -> Result<u32, Refused> {
        crate::audio::new_voice::new_voice(&self.audio.state)
    }
    fn make_ready(
        &mut self,
        at: rc2k_core::audio::load::At,
        voice_at: u32,
        shared: u32,
    ) -> Result<(), Refused> {
        use crate::audio::*;
        let state = &self.audio.state;
        let record = self.record_at(at.sound);
        fill_voice::fill_voice_in_game(self.audio, voice_at, self.record(at.sound), shared)?;
        let mode = sample_mode::sample_mode_of(
            &self.audio.sample_records,
            self.audio.state.sound_mode.get(),
            voice_of(state, voice_at).sound,
        )?;
        update_voice(state, voice_at, |v| samples::prepare_sample(state, v, mode))?;
        update_voice(state, voice_at, |v| keep_sample::keep_sample(state, v))?;
        make_buffer::make_buffer(state, voice_at, record)?;
        fill_buffer::fill_buffer(state, &voice_of(state, voice_at))?;

        Ok(())
    }
    fn mark_loaded(&mut self, voice: u32) {
        update_voice(&self.audio.state, voice, |v| v.flags |= LOADED);
    }
    fn quitting(&mut self) -> bool {
        self.system.session.flags.quitting.is_set()
    }
}

/// A list of sounds, read by position up to its -1: a race list may run on through the next fields
/// of the sets (`SoundSets::list_entry`); a read past the sets is refused.
fn list_entries<'a>(
    sets: &'a SoundSets,
    menu_sounds: &'a MenuSounds,
    list: SoundList,
) -> Result<Box<dyn Fn(usize) -> Option<u32> + 'a>, BadInput> {
    match list {
        SoundList::Menu => Ok(Box::new(|n| menu_sounds.sound(n))),
        SoundList::Race { field, entry } => {
            sets.list_entry(field, entry, 0)?;
            Ok(Box::new(move |n| {
                sets.list_entry(field, entry, n).ok().flatten()
            }))
        }
    }
}

pub fn load_sounds(
    audio: &AudioThread,
    system: &System,
    sets: &SoundSets,
    menu_sounds: &MenuSounds,
    list: Option<SoundList>,
) -> Result<(), Refused> {
    let Some(list) = list else {
        return Ok(());
    };
    let entry = list_entries(sets, menu_sounds, list)?;
    let refusal = std::cell::Cell::new(None);
    let entries = rc2k_core::audio::load::entries(|n| {
        entry(n).unwrap_or_else(|| {
            refusal.set(Some(BadInput(
                "a list of sounds without its end in its field",
            )));
            rc2k_core::audio::load::END
        })
    })?;
    if let Some(e) = refusal.get() {
        return Err(e.into());
    }
    let mut l = GameLoader { audio, system };
    rc2k_core::audio::load::load(&mut l, &entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every list a command can name ends in -1 inside its store, and holds only sounds.
    #[test]
    fn lists_end_inside_their_store() {
        let mut lists = vec![SoundList::Menu];
        for n in 0..72 {
            lists.push(SoundSets::kind_list_of(n).unwrap());
        }
        for set in crate::sound::sets_store::NamedSet::ALL {
            lists.push(set.list());
        }
        let sound = crate::test_support::own::sound();
        for list in lists {
            let entry =
                list_entries(&sound.threads.sets, &sound.threads.menu_sounds, list).unwrap();
            let entries = rc2k_core::audio::load::entries(|n| {
                entry(n).expect("a list runs out of its field")
            });
            assert!(entries.is_ok(), "{list:?}");
        }
        assert!(
            list_entries(
                &sound.threads.sets,
                &sound.threads.menu_sounds,
                SoundList::Race {
                    field: rc2k_core::audio::commands::RaceField::Tail,
                    entry: 1
                }
            )
            .is_err()
        );
    }
}
