//! The two stream commands: command 9, `audio::stream_command`, and command 0xB,
//! `audio::music_command`; and `audio::stop_streams`. Actions (unsigned): 0 stop; 1 the thread's
//! turn (feed while playing; stopped after it: stop, and for the music the next tune); 3 the
//! volume of what plays; any other plays (stop first if playing; nothing for a volume <= 0, for
//! no music, and a stop for no path or tune). Music tunes: the file is the music path with the
//! tune's two last digits at byte 14; tune 0xA is 10 + the tune turn for the first turn (counted on,
//! 0 .. 5) whose file exists. When a tune stops, the next tune is played, and it is never cleared
//! there: the second tune repeats (kept). Fixed: a turn past 5 starts from 0, and after `RETRIES`
//! plays in a row that stop at their first feed the music stops ("The music can hang the sound",
//! `wiki/Fixed-bugs.md`).

use crate::audio::own::{AudioOwn, AudioThread};
use crate::audio::stream::*;
use crate::errors::Refused;
use crate::memory::session::Shared;
use rc2k_core::audio::commands::StreamAction;

fn playing(player: &Shared<AudioStream>) -> bool {
    player.get().playing == 1
}

/// The core's player: one of the stream players, `path` the file it starts (none for a stop).
struct GamePlayer<'a> {
    state: &'a AudioOwn,
    player: &'a Shared<AudioStream>,
    path: Option<StreamPath<'a>>,
}

/// Where the file's name is read, when the stream starts.
#[derive(Clone, Copy)]
enum StreamPath<'a> {
    /// The music's path buffer.
    Music(&'a crate::memory::session::PlainBytes<24>),
    /// A sample name in the sound set (its text, read from the names' table).
    Name(&'a [u8]),
}

impl StreamPath<'_> {
    fn text(self) -> Result<Vec<u8>, crate::errors::BadInput> {
        match self {
            StreamPath::Music(field) => crate::files::cstring(field, "a stream's path"),
            StreamPath::Name(text) => Ok(text.to_vec()),
        }
    }
}

impl GamePlayer<'_> {
    /// `work` on a copy of the player's record, written back after it.
    fn with<R>(&self, work: impl FnOnce(&mut AudioStream) -> R) -> R {
        let mut stream = self.player.get();
        let out = work(&mut stream);
        self.player.set(stream);
        out
    }
}

impl rc2k_core::audio::streams::Player for GamePlayer<'_> {
    type Error = Refused;
    fn playing(&mut self) -> bool {
        playing(self.player)
    }
    fn keep_volume(&mut self, volume: u32) {
        self.player.update(|s| s.volume = volume);
    }
    fn start(&mut self) -> Result<bool, Refused> {
        let path = self
            .path
            .ok_or(crate::errors::BadInput("a stream started without a file"))?;
        let state = self.state;
        self.with(|s| start_stream(state, s, || path.text()))
    }
    fn feed(&mut self) -> Result<(), Refused> {
        let state = self.state;
        self.with(|s| feed_stream(state, s)).map(|_| ())
    }
    fn stop(&mut self) -> Result<(), Refused> {
        self.with(stop_stream)
    }
    fn apply_volume(&mut self) -> Result<(), Refused> {
        set_stream_volume(self.state, &self.player.get())
    }
}

impl rc2k_core::audio::streams::Tunes for GamePlayer<'_> {
    fn name(&mut self, n: u32) {
        let d = crate::strings::format_u32::format_u32(n, 0x12);
        // The tune's number written as text into the path, after its first 0xE bytes.
        if let Err(e) = self.state.music_path.write_at(0xE, &d, "a tune's path") {
            crate::fatal::raise::end("audio::stream_commands", e);
        }
    }
    fn exists(&mut self) -> Result<bool, Refused> {
        let path = crate::files::cstring(&self.state.music_path, "a tune's path")?;
        Ok(crate::files::file_exists::file_exists(&path)?)
    }
    fn gave_up(&mut self, tune: u32) {
        crate::program_log::log(&format!(
            "audio: tune {tune:#x}: {} plays in a row stopped at their first feed; the music stopped",
            rc2k_core::audio::streams::RETRIES
        ));
    }
}

/// `audio::stream_command` on the player at `player`. `name` is the sample name's key
/// (`SampleName::key`), 0 with none (a stop).
pub fn stream_command(
    audio: &AudioThread,
    name: u32,
    volume: i32,
    action: StreamAction,
) -> Result<(), Refused> {
    let text = match name {
        0 => None,
        _ => Some(crate::audio::sample_records::name_in(
            &audio.sample_names,
            name,
        )?),
    };
    let path = text.as_deref().map(StreamPath::Name);
    let mut p = GamePlayer {
        state: &audio.state,
        player: &audio.state.sound_stream,
        path,
    };
    rc2k_core::audio::streams::stream_command(&mut p, name != 0, volume, action)
}

/// `audio::music_command`: `tune` with `next` to follow.
pub fn music_command(
    state: &AudioOwn,
    tune: u32,
    next: u32,
    volume: i32,
    action: StreamAction,
) -> Result<(), Refused> {
    let mut m = rc2k_core::audio::streams::Music {
        next_tune: state.next_tune.get(),
        no_music: state.no_music.get(),
        turn: state.tune_turn.get(),
    };
    let mut p = GamePlayer {
        state,
        player: &state.music_stream,
        path: Some(StreamPath::Music(&state.music_path)),
    };
    let done = rc2k_core::audio::streams::music_command(&mut p, &mut m, tune, next, volume, action);
    state.next_tune.set(m.next_tune);
    state.tune_turn.set(m.turn);
    done
}
