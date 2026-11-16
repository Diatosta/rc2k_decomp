//! The options the player chooses on the options pages: the visual and sound settings and the
//! volumes. The front end owns them (`Menus::options`); the race, the network, the sound and the
//! options file read them through `MenuOptions::snapshot`. The key-entry state of the pages is
//! `menu::entry_store`.

use crate::memory::session::PlainDword;

/// The options as the pages edit and the options file keeps them.
#[derive(rc2k_derive::Cells)]
pub struct MenuOptions {
    /// Mipmapping, 0 off, 1 on (a copy of `ScreenChoices.mipmapping`; `OptionSwitch::Mipmapping`).
    pub mipmapping: PlainDword,
    /// Buffering, 0 double, 1 triple (a copy of `ScreenChoices.buffering`).
    pub buffering: PlainDword,
    /// Set when the screen mode was changed (`menu::option_steps`).
    pub mode_changed: PlainDword,
    /// The screen modes' switch (`OptionSwitch::ScreenModeCard`).
    pub modes_switch: PlainDword,
    /// The lens flare option (the race's `lens_flare`).
    pub lens_flare: PlainDword,
    /// A 0 ..= 100 level, stepped by dead code only; kept in the options file.
    pub unused_percent_a: PlainDword,
    /// The same, the second level.
    pub unused_percent_b: PlainDword,
    /// The reflections option.
    pub reflections: PlainDword,
    /// The car shadows setting (row 6 of the visual options, strings 0x396 "reduzido" / 0x397
    /// "completo"); copied to the race's `settings.car_shadows` and `settings_words[2]`.
    pub car_shadows: PlainDword,
    /// The car detail setting, 0 .. 2 (row 7, strings 0x393 "nenhum carro" / "o seu carro" /
    /// "todos os carros": which cars get the detailed textures; stepped by
    /// `menu::next_texture_setting`); copied to the race's `settings.car_detail` and
    /// `settings_words[3]`.
    pub car_detail: PlainDword,
    /// The headlights option.
    pub headlights: PlainDword,
    /// The audio quality chosen (`menu::next_audio_quality`).
    pub audio_quality: PlainDword,
    /// The DirectSound 3D switch.
    pub ds3d: PlainDword,
    /// The EAX switch.
    pub eax: PlainDword,
    /// The switch of Aureal's A3D as the options files hold it. The game has no A3D: no page
    /// shows this and nothing reads it; it is kept so that a file is written back as it was
    /// read (a file loaded at the start has it 0: `Options::without_a3d`).
    pub retired_a3d: PlainDword,
    /// The Dolby switch.
    pub dolby: PlainDword,
    /// The channels chosen.
    pub channels: PlainDword,
    /// The music level (`menu::option_steps`).
    pub music_level: PlainDword,
    /// The race music's level, read by the music "up" step (`menu::music_volume_up`).
    pub race_music_level: PlainDword,
    /// The menu sounds' third volume (`sound::play_menu_sound`).
    pub menu_sounds_level: PlainDword,
    /// The speech level (`menu::option_steps`, `sound::play_car_stream`).
    pub speech_level: PlainDword,
}

/// The options other subsystems read: the race's visual settings and the volumes the sound
/// code starts streams and menu sounds with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptionsSnapshot {
    pub lens_flare: u32,
    pub reflections: u32,
    pub car_shadows: u32,
    pub car_detail: u32,
    pub headlights: u32,
    pub music_level: u32,
    pub race_music_level: u32,
    pub menu_sounds_level: u32,
    pub speech_level: u32,
}

impl MenuOptions {
    /// The values the race, the network and the sound read.
    pub fn snapshot(&self) -> OptionsSnapshot {
        OptionsSnapshot {
            lens_flare: self.lens_flare.get(),
            reflections: self.reflections.get(),
            car_shadows: self.car_shadows.get(),
            car_detail: self.car_detail.get(),
            headlights: self.headlights.get(),
            music_level: self.music_level.get(),
            race_music_level: self.race_music_level.get(),
            menu_sounds_level: self.menu_sounds_level.get(),
            speech_level: self.speech_level.get(),
        }
    }

    pub const fn new() -> Self {
        MenuOptions {
            mipmapping: PlainDword::new(0x0),
            buffering: PlainDword::new(0x0),
            mode_changed: PlainDword::new(0x0),
            modes_switch: PlainDword::new(0x1),
            lens_flare: PlainDword::new(0x1),
            unused_percent_a: PlainDword::new(0x64),
            unused_percent_b: PlainDword::new(0x64),
            reflections: PlainDword::new(0x1),
            car_shadows: PlainDword::new(0x1),
            car_detail: PlainDword::new(0x1),
            headlights: PlainDword::new(0x0),
            audio_quality: PlainDword::new(0x3),
            ds3d: PlainDword::new(0x0),
            eax: PlainDword::new(0x0),
            retired_a3d: PlainDword::new(0x0),
            dolby: PlainDword::new(0x0),
            channels: PlainDword::new(0x1),
            music_level: PlainDword::new(0x40),
            race_music_level: PlainDword::new(0x40),
            menu_sounds_level: PlainDword::new(0x40),
            speech_level: PlainDword::new(0x40),
        }
    }
}

impl MenuOptions {
    /// Stores the five values of the sound page: the audio quality, DS3D, EAX, Dolby and the
    /// channels (`screens::load_audio_settings`).
    pub fn set_audio_page(&self, values: [u32; 5]) {
        let cells = [
            &self.audio_quality,
            &self.ds3d,
            &self.eax,
            &self.dolby,
            &self.channels,
        ];
        for (cell, value) in cells.into_iter().zip(values) {
            cell.set(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The snapshot holds each option's value and follows a change; the defaults are the
    /// options a fresh install starts with.
    #[test]
    fn snapshot_follows_the_cells() {
        let options = MenuOptions::new();
        let fresh = options.snapshot();
        assert_eq!(
            fresh,
            OptionsSnapshot {
                lens_flare: 1,
                reflections: 1,
                car_shadows: 1,
                car_detail: 1,
                headlights: 0,
                music_level: 0x40,
                race_music_level: 0x40,
                menu_sounds_level: 0x40,
                speech_level: 0x40,
            }
        );
        options.car_detail.set(2);
        options.speech_level.set(7);
        let changed = options.snapshot();
        assert_eq!((changed.car_detail, changed.speech_level), (2, 7));
        assert_eq!(
            OptionsSnapshot {
                car_detail: 1,
                speech_level: 0x40,
                ..changed
            },
            fresh
        );
    }
}
