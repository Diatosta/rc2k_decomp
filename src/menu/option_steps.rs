//! The options pages' small steps (item and left/right handlers, agent E 2026-10-19):
//! - the volume steps (`menu::music_volume_down` 0x467F48 .. `menu_sounds_b_volume_up`
//!   0x46810A): a work value one less (below 0: 0) or one more (above 0x40: 0x40), signed,
//!   then: the music's (`dword_755688`) into `g_musicVolume` and `sound::set_music_volume`
//!   (eax the value); the speech's (`dword_755698`) heard with `sound::start_stream` (eax
//!   the path ";3files\%speech\rally\st1.wav", ecx the value); two of the menu sounds'
//!   (`dword_75569C`, `dword_7556A0`) into `dword_7554E8` and `sound::play_menu_sound`
//!   with eax 0x10 / 0x20 (the first's) or 0x40 (the second's). Not `pusha`'d: they leave
//!   eax (the value, the path or the sound), ecx (the speech's: the value) as loaded.
//! - `menu::next_screen_mode` (0x46FFEF): `dword_4F50C8` one more, 0 past `dword_4F50E4` -
//!   1 (signed); `dword_755648` flipped; leaves eax `dword_755648`, ebx the last mode.
//! - `menu::previous_texture_setting` (0x47012B), `next_texture_setting` (0x47015B):
//!   `dword_755664` cycled through 0 .. 2 and copied to `dword_75AD14`; leave eax it.
//! - `menu::next_colour_a` .. `previous_colour_b` (0x471AE4, 0x471B16, 0x471B48, 0x471B7A):
//!   `dword_7554C0` (or `dword_7554C4`) cycled through 0 .. 6 and the entry of
//!   `dword_758A66` (or `dword_758A46`) it picks into `unk_758A2E` (or `unk_758A32`).
//!   Every register kept.
//! - `menu::apply_audio_choices` (0x470766): the advanced audio page's six work values
//!   (quality, channels, Dolby, A3D, EAX, DS3D) into `g_audioOptions`' choices,
//!   `audio::choose_modes`, and the six as applied back. Every register kept.
//!
//! The names of the texture and colour steps are readings of their variables.

use crate::audio::names_store::SampleNames;
use crate::errors::Refused;
use crate::gfx::owner::gfx;
use crate::menu::owner::Screens;
use crate::menu::owner::Tables;
use crate::session::Dword;

// ---- the volume steps ----

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Then {
    Music,
    Speech,
    /// `play_menu_sound` with this.
    Sound(u32),
}

#[derive(Clone, Copy)]
pub struct Volume {
    pub value: &'static Dword,
    pub up: bool,
    pub then: Then,
}

pub const MUSIC_LEVEL_RVA: &Dword = &crate::menu::owner::MENUS.screens.option_work.music_level; // dword_755688
pub const SPEECH_LEVEL_RVA: &Dword = &crate::menu::owner::MENUS.screens.option_work.speech_level; // dword_755698
pub const SOUNDS_A_RVA: &Dword = &crate::menu::owner::MENUS.screens.option_work.sounds_a; // dword_75569C
pub const SOUNDS_B_RVA: &Dword = &crate::menu::owner::MENUS.screens.option_work.sounds_b; // dword_7556A0
/// The Rust version of a volume step. What it leaves in registers is read by none of its
/// callers (the item call sites have nothing live after the call).
pub fn volume_step(screens: &Screens, v: Volume) {
    // Converted: `rc2k_core::menu::steps::volume`.
    let value = rc2k_core::menu::steps::volume(v.value.get() as i32, v.up);
    v.value.set(value as u32);
    match v.then {
        Then::Music => {
            crate::sound::owner::sound()
                .menu
                .options
                .music_volume
                .set(value as u32);
            crate::sound::set_music_volume::set_music_volume(value as u32);
        }
        Then::Speech => crate::sound::start_stream::start_stream(
            core::mem::offset_of!(SampleNames, menu_speech_st1) as u32,
            value as u32,
        ),
        Then::Sound(sound) => {
            screens.flow.menu_sounds_volume.set(value as u32);
            crate::sound::play_menu_sound::play_menu_sound_in_game(sound);
        }
    }
}

// ---- the plain steps (no calls) ----

/// `menu::next_screen_mode`: the next screen mode chosen, the changed flag flipped.
pub fn next_screen_mode(screens: &Screens) {
    let display = &gfx().display;
    // Converted: `rc2k_core::menu::steps::next_mode`.
    let mode = rc2k_core::menu::steps::next_mode(
        display.choices.resolution.get(),
        display.choices.resolutions.get(),
    );
    display.choices.resolution.set(mode);
    let changed = screens.option_work.mode_changed.get() ^ 1;
    screens.option_work.mode_changed.set(changed);
}

/// `menu::previous_texture_setting` / `next_texture_setting`: the texture setting one
/// step down or up.
pub fn step_texture_setting(screens: &Screens, up: bool) {
    // Converted: `rc2k_core::menu::steps::texture`.
    let v = rc2k_core::menu::steps::texture(screens.option_work.setting_3.get(), up);
    screens.option_work.setting_3.set(v);
    crate::race::owner::race()
        .shared
        .state
        .settings
        .setting_3
        .set(v);
}

/// One of the colour choices: (its number, the table, where the pick goes).
#[derive(Clone, Copy)]
pub struct Colour {
    pub number: &'static Dword,
    /// The table of the pictures' string ids, in `CAPTIONS.caption_lists`, from `table_at`.
    pub table_at: usize,
    pub pick: &'static Dword,
}

pub const COLOUR_A: Colour = Colour {
    number: &crate::menu::owner::MENUS.screens.flow.start_section,
    table_at: 21,                                                      // dword_758A66
    pick: &crate::menu::owner::MENUS.tables.captions.caption_lists[7], // dword_758A2E
};
pub const COLOUR_B: Colour = Colour {
    number: &crate::menu::owner::MENUS.screens.flow.start_section_a8,
    table_at: 13,                                                      // dword_758A46
    pick: &crate::menu::owner::MENUS.tables.captions.caption_lists[8], // dword_758A32
};

/// `menu::next_colour_a` and the three others.
pub fn step_colour(tables: &Tables, c: Colour, up: bool) {
    // Converted: `rc2k_core::menu::steps::colour`.
    let n = rc2k_core::menu::steps::colour(c.number.get() as i32, up);
    c.number.set(n as u32);
    // `CAPTIONS.caption_lists` from `table_at`: the original does not check `n` (a kept
    // number past 6 steps on from it); the address is resolved by `MOVED`.
    let pick = crate::session::read_on_in(
        &tables.captions,
        &tables.captions.caption_lists,
        c.table_at.wrapping_add(n as usize),
        "a caption list",
    )
    .unwrap_or_else(|e| crate::fatal::raise::end("a caption list", e));
    c.pick.set(pick);
}

// ---- the advanced audio page's choices ----

/// (the work value, its place among `g_audioOptions`' six choices and six applied).
const CHOICES: [(&Dword, usize); 6] = [
    (
        &crate::menu::owner::MENUS.screens.option_work.audio_quality,
        0,
    ), // quality
    (&crate::menu::owner::MENUS.screens.option_work.channels, 1), // channels
    (&crate::menu::owner::MENUS.screens.option_work.dolby, 5),    // Dolby
    (&crate::menu::owner::MENUS.screens.option_work.a3d, 4),      // A3D
    (&crate::menu::owner::MENUS.screens.option_work.eax, 3),      // EAX
    (&crate::menu::owner::MENUS.screens.option_work.ds3d, 2),     // DS3D
];
/// `menu::apply_audio_choices`: the registers it leaves (`popa`'s).
pub fn apply_audio_choices() -> Result<(), Refused> {
    for (work, n) in CHOICES {
        let value = work.get();
        crate::audio::owner::audio()
            .state
            .audio_options
            .update(|o| o.choice[n] = value);
    }
    // `audio::choose_modes` with the game's log (the original's eax there, the last
    // value copied, is read by none of it).
    crate::audio::choose_modes::in_game(&mut crate::audio::choose_modes::real_log)?;
    for (work, n) in CHOICES {
        work.set(
            crate::audio::owner::audio()
                .state
                .audio_options
                .get()
                .applied[n],
        );
    }

    Ok(())
}
