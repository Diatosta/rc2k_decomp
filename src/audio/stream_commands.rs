//! The two stream commands (IDA, 2026-10-14): command 9, `audio::stream_command`
//! (0x40C194, `g_soundStream`), and command 0xB, `audio::music_command` (0x40C4DD,
//! `g_musicStream`); and `audio::stop_streams` (0x40C5E8). Actions (edx, unsigned): 0 stop;
//! 1 the thread's turn (feed while playing; stopped after it: stop, and for the music the
//! next tune); 3 the volume of what plays; any other plays (stop first if playing; nothing
//! for a volume <= 0, for `g_noMusic` (music), and a stop for no path or tune).
//!
//! Music tunes: the file is `g_musicPath` with the tune's two last digits at +0xE
//! (`strings::format_u32`, flags 0x12); tune 0xA is 10 + `g_tuneTurn` for the first turn
//! (counted on, 0 .. 5) whose file exists. When a tune stops, `g_nextTune` is played, and it
//! is never cleared there: the second tune repeats. Fixed (2026-10-20, section 16): a turn
//! past 5 starts from 0, and after `RETRIES` plays in a row that stop at their first feed
//! the music stops (the original: both for ever, the audio thread hung).

use crate::audio::store::AUDIO;
use crate::audio::stream::*;
use crate::errors::Refused;
use crate::session::Shared;

fn playing(player: &Shared<AudioStream>) -> bool {
    player.get().playing == 1
}

/// The core's player: one of the game's stream players, `path` the file it starts (an
/// address).
struct GamePlayer {
    player: &'static Shared<AudioStream>,
    path: u32,
}

impl GamePlayer {
    /// The player's record, to work on in place.
    unsafe fn stream(&self) -> &'static mut AudioStream {
        unsafe { self.player.borrow() }
    }
}

impl rc2k_core::audio::streams::Player for GamePlayer {
    type Error = Refused;
    fn playing(&mut self) -> bool {
        playing(self.player)
    }
    fn keep_volume(&mut self, volume: u32) {
        unsafe { self.stream().volume = volume };
    }
    fn start(&mut self) -> Result<bool, Refused> {
        unsafe { start_stream(self.stream(), self.path as *const u8) }
    }
    fn feed(&mut self) -> Result<(), Refused> {
        unsafe { feed_stream(self.stream()) }.map(|_| ())
    }
    fn stop(&mut self) -> Result<(), Refused> {
        unsafe { stop_stream(self.stream()) }
    }
    fn apply_volume(&mut self) -> Result<(), Refused> {
        unsafe { set_stream_volume(self.stream()) }
    }
}

impl rc2k_core::audio::streams::Tunes for GamePlayer {
    fn name(&mut self, n: u32) {
        let d = crate::strings::format_u32::format_u32(n, 0x12);
        unsafe { std::ptr::copy_nonoverlapping(d.as_ptr(), (self.path + 0xE) as *mut u8, d.len()) };
    }
    fn exists(&mut self) -> Result<bool, Refused> {
        Ok(crate::files::file_exists::file_exists(
            self.path as *const u8,
        )?)
    }
    fn gave_up(&mut self, tune: u32) {
        crate::framework::log(&format!(
            "audio: tune {tune:#x}: {} plays in a row stopped at their first feed; the music stopped",
            rc2k_core::audio::streams::RETRIES
        ));
    }
}

/// `audio::stream_command` on the player at `player`. Converted:
/// `rc2k_core::audio::streams::stream_command`.
pub fn stream_command(path: u32, volume: i32, action: u32) -> Result<(), Refused> {
    let mut p = GamePlayer {
        player: &AUDIO.sound_stream,
        path,
    };
    rc2k_core::audio::streams::stream_command(&mut p, path != 0, volume, action)
}

/// `audio::music_command`: `tune` with `next` to follow. Converted:
/// `rc2k_core::audio::streams::music_command` (its calls do not read the three variables,
/// which are stored once it is done).
pub fn music_command(tune: u32, next: u32, volume: i32, action: u32) -> Result<(), Refused> {
    let turn = &crate::sound::store::SOUND_OPTIONS.tune_turn;
    let mut m = rc2k_core::audio::streams::Music {
        next_tune: AUDIO.next_tune.get(),
        no_music: AUDIO.no_music.get(),
        turn: turn.get(),
    };
    let mut p = GamePlayer {
        player: &AUDIO.music_stream,
        path: AUDIO.music_path.addr() as u32,
    };
    let done = rc2k_core::audio::streams::music_command(&mut p, &mut m, tune, next, volume, action);
    AUDIO.next_tune.set(m.next_tune);
    turn.set(m.turn);
    done
}
