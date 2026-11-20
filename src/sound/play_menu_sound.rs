//! `sound::play_menu_sound`: plays one of the menus' sounds.
//!
//! Nothing for no sound bits (0), and nothing before the time kept (125 ms after the last sound;
//! compared by the sign of the difference). The time is moved on before the bits are looked at. The
//! first bit set of these, in this order, picks the sound and the volume it is played at: 0x80
//! sound 0xE4 at `levels.race_effects_volume`; 0x10, 0x20, 0x40 sounds 0x163, 0x197, 0x1CB at
//! `ScreenFlow.menu_sounds_volume`; 0x100, 8, 2, 4, 1 sounds 0x1CC, 0x1CA, 0x1C8, 0x1C9, 0x1C7 at
//! `MenuOptions.menu_effects_volume`. Then `sound::play_sound` with the sample's own rate and no
//! place.

use crate::audio::owner::Audio;
use crate::clock::tick_count::tick_count;
use crate::menu::owner::Menus;
use crate::sound::owner::Sound;

/// Plays the menu's sound `sound` (0: none), the gap since the last one taken from the picture
/// clock.
pub fn play_menu_sound(
    sounds: &Sound,
    audio: &Audio,
    menus: &Menus,
    picture_clock: &crate::clock::tick_count::PictureClock,
    sound: u32,
) {
    // The logic is `rc2k_core::sound::senders::menu_sound`.
    use rc2k_core::sound::senders::{MenuVolume, menu_sound};
    if sound == 0 {
        return;
    }
    let Some((next, pick)) = menu_sound(
        sound,
        tick_count(picture_clock),
        sounds.menu.next_sound_time.get().get(),
    ) else {
        return;
    };
    sounds.menu.next_sound_time.get().set(next);
    let Some((sound, volume)) = pick else {
        return;
    };
    let volume: u32 = match volume {
        MenuVolume::A => audio.levels.race_effects_volume.get(),
        MenuVolume::B => menus.screens.flow.read().menu_sounds_volume,
        MenuVolume::C => menus.options.snapshot().menu_effects_volume,
    };
    crate::sound::play_sound::play_sound_in(audio, sound, volume, 0xFFFF_FF91, [0; 3]);
}
