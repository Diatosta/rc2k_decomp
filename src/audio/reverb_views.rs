//! Which player's view the reverb of a sound follows, and which sounds stay dry. The game thread
//! plays a sound with its sound number only; while it plays a player's car sounds it says whose
//! they are, and while it plays a voice or an interface sound it says that is dry. The number is
//! tagged with that, and the audio thread, putting a voice's 3D buffer in the reverb, reads the
//! tag of its sound number. Sounds nobody tagged (the world's own, the stage's) follow view 0.

use crate::audio::voices::SAMPLES;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering::Relaxed};

/// A sound number's tag: 0 for the world's untagged, `1 + view` for a view's, `DRY` for no reverb.
static SOUND_VIEWS: [AtomicU8; SAMPLES as usize] = [const { AtomicU8::new(0) }; SAMPLES as usize];

const DRY: u8 = 0xFF;

/// What the sounds being played now belong to: a view, or `DRY_NOW`.
static PLAYING_FOR: AtomicUsize = AtomicUsize::new(0);
const DRY_NOW: usize = usize::MAX;

/// Runs `play` with the sounds it starts belonging to player view `view`.
pub fn playing_for<R>(view: usize, play: impl FnOnce() -> R) -> R {
    with_owner(view, play)
}

/// Runs `play` with the sounds it starts kept out of the reverb: voices (the co-driver's pace
/// notes) and the interface's sounds, which are not in the world.
pub fn dry<R>(play: impl FnOnce() -> R) -> R {
    with_owner(DRY_NOW, play)
}

fn with_owner<R>(owner: usize, play: impl FnOnce() -> R) -> R {
    let before = PLAYING_FOR.swap(owner, Relaxed);
    let done = play();
    PLAYING_FOR.store(before, Relaxed);
    done
}

/// A sound being played now: its number tagged with what is being played for.
pub fn tag(sound_word: u32) {
    if let Some(slot) = SOUND_VIEWS.get(sound_word as u16 as usize) {
        let tag = match PLAYING_FOR.load(Relaxed) {
            DRY_NOW => DRY,
            view => view.min(usize::from(DRY) - 2) as u8 + 1,
        };
        // Untagged world sounds (view 0) keep the untagged value.
        slot.store(if tag == 1 { 0 } else { tag }, Relaxed);
    }
}

/// The view a sound number's reverb follows, or none for a dry sound.
pub fn view_of(sound: u32) -> Option<usize> {
    match SOUND_VIEWS.get(sound as usize)?.load(Relaxed) {
        0 => Some(0),
        DRY => None,
        tag => Some(usize::from(tag) - 1),
    }
}
