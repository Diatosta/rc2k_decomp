//! The options screens' work copies, out of RAL.EXE's image (IDA 0x755640 .. 0x7556B4; phase 7, typed):
//! the switches and settings the options pages change, the sound options and volumes, the menu sound
//! and the key being redefined. The first 0x2C and the next 0x18 bytes are the options file's
//! (`saves::load_options`). A `#[repr(C)]` struct with the image's layout, one static;
//! `moved::MOVED` points the range at it. (Split 2026-10-25 from the front end's block `menu::store`.)

use crate::session::Dword;

/// The options' work copies (IDA 0x755640 .. 0x7556B4).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct OptionWork {
    /// `dword_755640`: A visual page switch (a copy of `dword_4F50D0`; `OptionSwitch::Mipmapping`).
    pub switch_0: Dword,
    /// `dword_755644`: A visual page switch (a copy of `dword_4F50CC`).
    pub switch_1: Dword,
    /// `dword_755648`: Set when the screen mode was changed (`menu::option_steps`).
    pub mode_changed: Dword,
    /// `dword_75564C`: The screen modes' switch (`OptionSwitch::ScreenModeCard`).
    pub modes_switch: Dword,
    /// `dword_755650`: The graphics page's lens flare option, work copy (the race's `lens_flare`).
    pub lens_flare: Dword,
    /// `dword_755654`: Not worked out.
    pub dword_755654: Dword,
    /// `dword_755658`: Not worked out.
    pub dword_755658: Dword,
    /// `dword_75565C`: The graphics page's reflections option, work copy.
    pub reflections: Dword,
    /// +0x10 (0x755660) `dword_755660`: the options page's car shadows setting (row 6 of the visual options, strings 0x396 "reduzido" / 0x397 "completo"); copied to the race's `settings.car_shadows` (0x75AD10) and `settings_words[2]` (0x6238B0). Probable: the debug tab reads the race copy as "Car shadows".
    pub car_shadows: Dword,
    /// +0x14 (0x755664) `dword_755664`: the options page's car detail setting, 0 .. 2 (row 7, strings 0x393 "nenhum carro" / "o seu carro" / "todos os carros": which cars get the detailed textures; stepped by `menu::next_texture_setting`); copied to the race's `settings.car_detail` (0x75AD14) and `settings_words[3]` (0x6238B4). Probable, from the string and the page's row order.
    pub car_detail: Dword,
    /// `dword_755668`: The graphics page's headlights option, work copy.
    pub headlights: Dword,
    /// `dword_75566C`: The audio quality chosen (`menu::next_audio_quality`).
    pub audio_quality: Dword,
    /// `dword_755670`: The DirectSound 3D switch.
    pub ds3d: Dword,
    /// `dword_755674`: The EAX switch.
    pub eax: Dword,
    /// `dword_755678`: The A3D switch.
    pub a3d: Dword,
    /// `dword_75567C`: The Dolby switch.
    pub dolby: Dword,
    /// `dword_755680`: The channels chosen.
    pub channels: Dword,
    /// `g_masterVolumeShown`: The master volume shown (`audio::primary`, `menu::run_front_end`).
    pub master_volume_shown: Dword,
    /// `dword_755688`: The music level (`menu::option_steps`).
    pub music_level: Dword,
    /// `dword_75568C`: Read by the music "up" step (`menu::music_volume_up`).
    pub other_volume: Dword,
    /// `dword_755690`: The menu sounds' third volume (`sound::play_menu_sound`).
    pub menu_sounds_c: Dword,
    /// `dword_755694`: The menu sounds' first volume.
    pub menu_sounds_a: Dword,
    /// `dword_755698`: The speech level (`menu::option_steps`, `sound::play_car_stream`).
    pub speech_level: Dword,
    /// `dword_75569C`: The sounds' first level.
    pub sounds_a: Dword,
    /// `dword_7556A0`: The sounds' second level.
    pub sounds_b: Dword,
    /// `dword_7556A4`: The menu's sound: 0x6F, or 0x14D after Enter (`menu::handle_keys`).
    pub menu_sound: Dword,
    /// `dword_7556A8`: 1 while a text is being typed (`menu::text_entry`).
    pub typing: Dword,
    /// `g_redefiningKey`: 1 while a key is being redefined; also 1 = the highlight does not blink (`menu::update_blink`).
    pub redefining_key: Dword,
    /// `g_redefinedKeyPlace`: The place of the key being redefined (999 = none).
    pub redefined_key_place: Dword,
}

impl OptionWork {
    /// First values the image's (`tests::option_work_starts_as_the_games`).
    pub const fn new() -> Self {
        OptionWork {
            switch_0: Dword::new(0x0),
            switch_1: Dword::new(0x0),
            mode_changed: Dword::new(0x0),
            modes_switch: Dword::new(0x1),
            lens_flare: Dword::new(0x1),
            dword_755654: Dword::new(0x64),
            dword_755658: Dword::new(0x64),
            reflections: Dword::new(0x1),
            car_shadows: Dword::new(0x1),
            car_detail: Dword::new(0x1),
            headlights: Dword::new(0x0),
            audio_quality: Dword::new(0x3),
            ds3d: Dword::new(0x0),
            eax: Dword::new(0x0),
            a3d: Dword::new(0x0),
            dolby: Dword::new(0x0),
            channels: Dword::new(0x1),
            master_volume_shown: Dword::new(0x40),
            music_level: Dword::new(0x40),
            other_volume: Dword::new(0x40),
            menu_sounds_c: Dword::new(0x40),
            menu_sounds_a: Dword::new(0x40),
            speech_level: Dword::new(0x40),
            sounds_a: Dword::new(0x40),
            sounds_b: Dword::new(0x40),
            menu_sound: Dword::new(0x6F),
            typing: Dword::new(0x0),
            redefining_key: Dword::new(0x0),
            redefined_key_place: Dword::new(0x3E7),
        }
    }
}

/// IDA 0x755640, the first byte of the piece.
pub const START: usize = 0x755640;
/// IDA 0x7556B4, the end of the piece.
pub const END: usize = 0x7556B4;

/// Where IDA 0x755640 is now (`moved::MOVED`'s home).
#[cfg(test)]
pub fn option_work_addr() -> u32 {
    crate::pointers::game_address(&raw const crate::menu::owner::MENUS.screens.option_work)
}

impl OptionWork {
    /// Stores the six values of the sound page, in memory order: the audio quality, DS3D, EAX, A3D,
    /// Dolby and the channels (`screens::load_audio_settings`).
    pub fn set_audio_page(&self, values: [u32; 6]) {
        let cells = [
            &self.audio_quality,
            &self.ds3d,
            &self.eax,
            &self.a3d,
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
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn option_work_starts_as_the_games() {
        assert!(
            END - START <= size_of::<OptionWork>()
                && size_of::<OptionWork>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(option_work_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(OptionWork, switch_0), 0x755640);
        at(offset_of!(OptionWork, switch_1), 0x755644);
        at(offset_of!(OptionWork, mode_changed), 0x755648);
        at(offset_of!(OptionWork, modes_switch), 0x75564C);
        at(offset_of!(OptionWork, lens_flare), 0x755650);
        at(offset_of!(OptionWork, dword_755654), 0x755654);
        at(offset_of!(OptionWork, dword_755658), 0x755658);
        at(offset_of!(OptionWork, reflections), 0x75565C);
        at(offset_of!(OptionWork, car_shadows), 0x755660);
        at(offset_of!(OptionWork, car_detail), 0x755664);
        at(offset_of!(OptionWork, headlights), 0x755668);
        at(offset_of!(OptionWork, audio_quality), 0x75566C);
        at(offset_of!(OptionWork, ds3d), 0x755670);
        at(offset_of!(OptionWork, eax), 0x755674);
        at(offset_of!(OptionWork, a3d), 0x755678);
        at(offset_of!(OptionWork, dolby), 0x75567C);
        at(offset_of!(OptionWork, channels), 0x755680);
        at(offset_of!(OptionWork, master_volume_shown), 0x755684);
        at(offset_of!(OptionWork, music_level), 0x755688);
        at(offset_of!(OptionWork, other_volume), 0x75568C);
        at(offset_of!(OptionWork, menu_sounds_c), 0x755690);
        at(offset_of!(OptionWork, menu_sounds_a), 0x755694);
        at(offset_of!(OptionWork, speech_level), 0x755698);
        at(offset_of!(OptionWork, sounds_a), 0x75569C);
        at(offset_of!(OptionWork, sounds_b), 0x7556A0);
        at(offset_of!(OptionWork, menu_sound), 0x7556A4);
        at(offset_of!(OptionWork, typing), 0x7556A8);
        at(offset_of!(OptionWork, redefining_key), 0x7556AC);
        at(offset_of!(OptionWork, redefined_key_place), 0x7556B0);
    }
}
