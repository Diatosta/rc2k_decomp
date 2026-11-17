//! Which player's view the reverb of a sound follows. The game thread plays a sound with its sound
//! number only; while it plays a player's car sounds it says whose they are, and the number is
//! tagged with that view. The audio thread, putting a voice's 3D buffer in the reverb, reads the
//! tag of its sound number. Sounds nobody tagged (the world's own, the stage's) follow view 0.

use crate::audio::voices::SAMPLES;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering::Relaxed};

static SOUND_VIEWS: [AtomicU8; SAMPLES as usize] = [const { AtomicU8::new(0) }; SAMPLES as usize];

/// The view the sounds being played now belong to.
static PLAYING_FOR: AtomicUsize = AtomicUsize::new(0);

/// Runs `play` with the sounds it starts belonging to player view `view`.
pub fn playing_for<R>(view: usize, play: impl FnOnce() -> R) -> R {
    let before = PLAYING_FOR.swap(view, Relaxed);
    let done = play();
    PLAYING_FOR.store(before, Relaxed);
    done
}

/// A sound being played now: its number tagged with the view being played for.
pub fn tag(sound_word: u32) {
    if let Some(slot) = SOUND_VIEWS.get(sound_word as u16 as usize) {
        slot.store(PLAYING_FOR.load(Relaxed) as u8, Relaxed);
    }
}

/// The view a sound number's reverb follows.
pub fn view_of(sound: u32) -> usize {
    SOUND_VIEWS
        .get(sound as usize)
        .map_or(0, |slot| slot.load(Relaxed) as usize)
}
