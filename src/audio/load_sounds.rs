//! `audio::load_sounds` (0x40AEB9): command 3: a list of sounds loaded.
//!
//! For each sound whose record has a file (`data_key`) and `copies` above 0 (signed), that
//! many voices while fewer than 0x200 are in use: `audio::find_shared_voice`; none:
//! `audio::read_sample_of` (a carry ends the sound's voices); `audio::new_voice` (the
//! record's first voice if it has none); `audio::fill_voice` with the shared voice;
//! `audio::sample_mode` on `g_soundMode`; `audio::prepare_sample`; `audio::keep_sample`;
//! `audio::make_buffer` and `audio::fill_buffer` (their carries not looked at); the
//! voice's flags get `LOADED`. After each entry of the list, `g_quitting` 1 ends it. The
//! callees get the record, the voice and the shared voice or the mode; `make_buffer` gets
//! the record as the holder value its EAX set-up reads through. One
//! caller, `audio::take_commands`.
//! Refuses, before anything is done: a list without its -1 within 0x1000 entries, or with
//! a sound of 0x1CD or more; A3D.

use crate::audio::owner::Audio;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::sound::sets_store::RaceSoundSets;
use crate::system::owner::system;
use rc2k_core::audio::commands::SoundList;

/// The core's loader: each step is one of the original's callees, given what it reads (the
/// record, the voice, the shared voice or the mode; `make_buffer` the record as the
/// holder value its EAX set-up reads through).
struct GameLoader<'a> {
    audio: &'a Audio,
}

impl<'a> GameLoader<'a> {
    /// The record, which `make_buffer` takes as its holder value.
    fn record_at(&self, sound: u32) -> crate::audio::property_sets::Holder<'a> {
        let audio: &'a Audio = self.audio;
        crate::audio::property_sets::Holder::Record(&audio.sample_records.records[sound as usize])
    }
}

impl rc2k_core::audio::load::Loader for GameLoader<'_> {
    type Error = Refused;
    fn record(&mut self, sound: u32) -> SampleRecord {
        self.audio.sample_records.records[sound as usize].get()
    }
    fn set_first_voice(&mut self, sound: u32, voice: u32) {
        self.audio.sample_records.records[sound as usize].update(|r| r.first_voice = voice);
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
                &audio.sample_records.records[at.sound as usize].get(),
                count.min(VOICES),
            )
        })
    }
    fn read(&mut self, at: rc2k_core::audio::load::At) -> Result<bool, Refused> {
        use crate::audio::{read_sample_of, samples};
        read_sample_of::read_sample_of(
            &self.audio.sample_records.records[at.sound as usize].get(),
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
        update_voice(state, voice_at, |v| samples::prepare_sample(v, mode))?;
        update_voice(state, voice_at, |v| keep_sample::keep_sample(state, v))?;
        make_buffer::make_buffer(state, voice_at, record)?;
        fill_buffer::fill_buffer(state, &voice_of(state, voice_at))?;

        Ok(())
    }
    fn mark_loaded(&mut self, voice: u32) {
        update_voice(&self.audio.state, voice, |v| v.flags |= LOADED);
    }
    fn quitting(&mut self) -> bool {
        system().session.flags.quitting.is_set()
    }
}

/// A list of sounds, read by position. The original reads a list on until its -1, wherever
/// that is: the menu's list is its whole record; a race list may run on through the next
/// fields of the sets (`RaceSoundSets::list_entry`), and a read past the sets is refused.
fn list_entries(list: SoundList) -> Result<Box<dyn Fn(usize) -> Option<u32>>, BadInput> {
    match list {
        SoundList::Menu => Ok(Box::new(|n| {
            crate::sound::owner::sound().menu.menu_sounds.sound(n)
        })),
        SoundList::Race { offset } => {
            RaceSoundSets::list_entry(offset, 0)?;
            Ok(Box::new(move |n| {
                RaceSoundSets::list_entry(offset, n).ok().flatten()
            }))
        }
    }
}

/// Converted: `rc2k_core::audio::load` (`entries`, `load`).
pub fn load_sounds(audio: &Audio, list: Option<SoundList>) -> Result<(), Refused> {
    let Some(list) = list else {
        return Ok(());
    };
    if crate::audio::buffer_settings::a3d(&audio.state) {
        return Err(BadInput("A3D").into());
    }
    let entry = list_entries(list)?;
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
    let mut l = GameLoader { audio };
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
            lists.push(RaceSoundSets::kind_list_of(n).unwrap());
        }
        for field in [
            core::mem::offset_of!(RaceSoundSets, replay_set),
            core::mem::offset_of!(RaceSoundSets, first_set_a),
            core::mem::offset_of!(RaceSoundSets, first_set_b),
            core::mem::offset_of!(RaceSoundSets, first_set_c),
            core::mem::offset_of!(RaceSoundSets, second_set_a),
            core::mem::offset_of!(RaceSoundSets, second_set_b),
            core::mem::offset_of!(RaceSoundSets, last_set),
        ] {
            lists.push(SoundList::Race {
                offset: field as u32,
            });
        }
        for list in lists {
            let entry = list_entries(list).unwrap();
            let entries = rc2k_core::audio::load::entries(|n| {
                entry(n).expect("a list runs out of its field")
            });
            assert!(entries.is_ok(), "{list:?}");
        }
        assert!(list_entries(SoundList::Race { offset: 0x10000 }).is_err());
    }
}
