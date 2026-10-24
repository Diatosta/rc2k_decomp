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

use crate::audio::stream::*;
use crate::errors::Refused;
use crate::store::addr32 as a;
use crate::store::game_addr;

fn playing(player: u32) -> bool {
    unsafe { stream(player).playing == 1 }
}

/// The core's player: the game's stream player at `player`, `path` the file it starts.
struct GamePlayer {
    player: u32,
    path: u32,
}

impl rc2k_core::audio::streams::Player for GamePlayer {
    type Error = Refused;
    fn playing(&mut self) -> bool {
        playing(self.player)
    }
    fn keep_volume(&mut self, volume: u32) {
        unsafe { stream(self.player).volume = volume };
    }
    fn start(&mut self) -> Result<bool, Refused> {
        unsafe {
            start_stream(
                stream(self.player),
                self.path as *const u8,
                &StartVars::game(),
            )
        }
    }
    fn feed(&mut self) -> Result<(), Refused> {
        let parts = game_addr(crate::audio::fill_buffer::LOCKED_PARTS_RVA) as u32;
        unsafe { feed_stream(stream(self.player), parts) }.map(|_| ())
    }
    fn stop(&mut self) -> Result<(), Refused> {
        unsafe { stop_stream(stream(self.player)) }
    }
    fn apply_volume(&mut self) -> Result<(), Refused> {
        unsafe { set_stream_volume(stream(self.player)) }
    }
}

impl rc2k_core::audio::streams::Tunes for GamePlayer {
    fn name(&mut self, n: u32) {
        let d = crate::strings::format_u32::format_u32(n, 0x12);
        unsafe { std::ptr::copy_nonoverlapping(d.as_ptr(), (self.path + 0xE) as *mut u8, d.len()) };
    }
    fn exists(&mut self) -> Result<bool, Refused> {
        Ok(unsafe { crate::files::file_exists::file_exists(self.path as *const u8) }?)
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
pub fn stream_command(path: u32, volume: i32, action: u32, player: u32) -> Result<(), Refused> {
    let mut p = GamePlayer { player, path };
    rc2k_core::audio::streams::stream_command(&mut p, path != 0, volume, action)
}

/// Where `music_command` keeps its things.
#[derive(Clone, Copy)]
pub struct MusicVars {
    pub player: u32,
    pub next_tune: u32,
    pub no_music: u32,
    pub path: u32,
    pub turn: u32,
}

impl MusicVars {
    pub fn game() -> Self {
        Self {
            player: a(MUSIC_STREAM_RVA),
            next_tune: a(NEXT_TUNE_RVA),
            no_music: a(NO_MUSIC_RVA),
            path: a(MUSIC_PATH_RVA),
            turn: a(TUNE_TURN_RVA),
        }
    }
}

/// `audio::music_command`: `tune` with `next` to follow. Converted:
/// `rc2k_core::audio::streams::music_command` (its calls do not read the three variables,
/// which are stored once it is done).
pub fn music_command(
    tune: u32,
    next: u32,
    volume: i32,
    action: u32,
    v: &MusicVars,
) -> Result<(), Refused> {
    use crate::audio::voices::{read, write};
    let mut m = unsafe {
        rc2k_core::audio::streams::Music {
            next_tune: read(v.next_tune),
            no_music: read(v.no_music),
            turn: read(v.turn),
        }
    };
    let mut p = GamePlayer {
        player: v.player,
        path: v.path,
    };
    let done = rc2k_core::audio::streams::music_command(&mut p, &mut m, tune, next, volume, action);
    unsafe {
        write(v.next_tune, m.next_tune);
        write(v.turn, m.turn);
    }
    done
}
