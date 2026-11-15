//! Reads `var\ral.cfg` and stores each setting it names in its variable; this file holds the
//! table of those variables, the key list and parser are `rc2k_core::files::settings`.
//! Callers: start-up and `system::load_game_data`.

use crate::errors::BadInput;
use crate::owners::Owners;
use crate::session::DwordLike;
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;

// ---- game data ----

/// Where a setting's variable lives: its store's cell
/// (`tests::targets_are_the_variables_the_games_table_names` checks each cell).
#[derive(Clone, Copy)]
enum Target {
    /// A whole aligned dword of a store (`dword!`).
    Dword(fn(Owners<'static>) -> &'static dyn DwordLike),
    /// A dword that straddles two cells of its store's untyped tail: the store's own
    /// setter writes the four bytes (`race::viewer_screen_store`'s).
    Unaligned(fn(Owners<'static>, u32)),
}

impl Target {
    /// The four bytes of the variable set.
    fn set(self, o: Owners<'static>, number: u32) {
        match self {
            Target::Dword(cell) => cell(o).set(number),
            Target::Unaligned(set) => set(o, number),
        }
    }
}

/// `dword!(o => o.owner.field)`: the dword as a `Target` (a `fn` item, so the table stays
/// `const`); `o` names the owners the path starts from.
macro_rules! dword {
    ($o:ident => $($path:tt)+) => {
        Target::Dword({
            fn cell($o: Owners<'static>) -> &'static dyn DwordLike {
                &$($path)+
            }
            cell
        })
    };
}

/// `wrreplayenable`: `ViewerAndKinds.viewer_panel.untyped_tail[0]` (unaligned, in the viewer panel's tail).
fn set_developer_keys(o: Owners<'static>, number: u32) {
    o.race.viewer_panel().set_developer_keys(number);
}

/// `wrreplayrate`: `ViewerAndKinds.viewer_panel.untyped_tail[1]` (unaligned, in the viewer panel's tail).
fn set_picture_rate(o: Owners<'static>, number: u32) {
    o.race.viewer_panel().set_picture_rate(number);
}

/// "var\ral.cfg" (0x5295CC in the image).
static PATH: &[u8; 12] = b"var\\ral.cfg\0";
/// The settings in `rc2k_core::files::settings::SETTINGS` order (`tests::table_matches_the_keys`
/// checks it): key, address of the variable in the 1999 image, target.
const ENTRIES: [(&[u8], u32, Target); 77] = [
    (
        b"notaskswitch",
        0x518384,
        dword!(o => o.system.window_loop().no_task_switch),
    ),
    (
        b"noprichange",
        0x518388,
        dword!(o => o.system.window_loop().keep_priority),
    ),
    (
        b"ignoremmx",
        0x5183AC,
        dword!(o => o.system.machine().ignore_mmx),
    ),
    (
        b"ignorekatmai",
        0x5183B4,
        dword!(o => o.system.machine().ignore_katmai),
    ),
    (
        b"ignoreamdk3d",
        0x5183BC,
        dword!(o => o.system.machine().ignore_3dnow),
    ),
    (
        b"altmemenable",
        0x4E0584,
        dword!(_o => *crate::memory::state::OWN_HEAP.get()),
    ),
    (
        b"altmemheapsize",
        0x4E0588,
        dword!(_o => *crate::memory::state::OWN_HEAP_KB.get()),
    ),
    (b"noaudio", 0x4F82B4, dword!(o => o.audio.state.no_audio)),
    (b"nosound", 0x4F82B4, dword!(o => o.audio.state.no_audio)),
    (
        b"nocdaudio",
        0x518104,
        dword!(o => o.sound.menu.cd_audio.get().no_cd_audio),
    ),
    (b"nomusic", 0x5069BC, dword!(o => o.audio.state.no_music)),
    (
        b"audiobasefreq",
        0x4F82D4,
        dword!(o => o.audio.state.mix_rate_setting),
    ),
    (
        b"ignoreds3d",
        0x506B42,
        dword!(o => o.audio.state.no_listener),
    ),
    (
        b"ignoredsv",
        0x506C50,
        dword!(o => o.audio.state.ignore_dsv_setting),
    ),
    (
        b"ignoreeax",
        0x506C6C,
        dword!(o => o.audio.state.ignore_eax_setting),
    ),
    (
        b"ignorea3d",
        0x506C5C,
        dword!(o => o.audio.state.no_a3d_setting),
    ),
    (
        b"wavheapsize",
        0x5068EC,
        dword!(o => o.audio.state.sample_buffer_kb),
    ),
    (
        b"bundleheapsize",
        0x51B980,
        dword!(o => o.gfx.fonts.glyph_tables.units),
    ),
    (
        b"imageheapsize",
        0x519554,
        dword!(o => o.strings.lexicon.memory.kilobytes),
    ),
    (
        b"gamegfxheapsize",
        0x5BE894,
        dword!(o => o.gfx.fonts.game_fonts_memory.units),
    ),
    (
        b"forcedx5",
        0x5AB958,
        dword!(o => o.gfx.d3d.cards.get().force_dx5),
    ),
    (
        b"forceno3dcard",
        0x5AB94C,
        dword!(o => o.gfx.d3d.cards.get().no_search),
    ),
    (
        b"ignoreprimary",
        0x5AB950,
        dword!(o => o.gfx.d3d.cards.get().not_primary),
    ),
    (
        b"ignoresecondary",
        0x5AB954,
        dword!(o => o.gfx.d3d.cards.get().ignore_secondary),
    ),
    (
        b"agpmode",
        0x5AB95C,
        dword!(o => o.gfx.d3d.cards.get().agp_mode),
    ),
    (
        b"force3dtrans",
        0x5AB97C,
        dword!(o => o.gfx.d3d.cards.get().force_trans),
    ),
    (
        b"force3dalpha",
        0x5AB984,
        dword!(o => o.gfx.d3d.cards.get().force_alpha),
    ),
    (
        b"force3dshadow",
        0x5AB980,
        dword!(o => o.gfx.d3d.cards.get().force_shadow),
    ),
    (
        b"force3dfogging",
        0x5AB988,
        dword!(o => o.gfx.d3d.cards.get().force_fogging),
    ),
    (
        b"textflashmode",
        0x4F3F74,
        dword!(o => o.strings.text.blend_mode),
    ),
    (
        b"videomodemain",
        0x4F4944,
        dword!(o => o.gfx.display.mode.menu_mode),
    ),
    (
        b"videomodegame",
        0x4F4948,
        dword!(o => o.gfx.display.mode.race_mode),
    ),
    (
        b"v3driverfix",
        0x624238,
        dword!(o => o.scene.drawing.scene.get().v3_driver_fix),
    ),
    (b"nopngdecode", 0x51B97C, dword!(o => o.gfx.png.setting)),
    (
        b"nojoy",
        0x4F68DC,
        dword!(o => o.input.devices.no_joysticks),
    ),
    (b"joymode", 0x4F68E0, dword!(o => o.input.devices.joy_mode)),
    (b"joycode", 0x4F68E4, dword!(o => o.input.devices.joy_code)),
    (
        b"joylowerdz",
        0x4F68E8,
        dword!(o => o.input.devices.joy_dead_zone),
    ),
    (
        b"joyupperdz",
        0x4F68EC,
        dword!(o => o.input.devices.joy_end_zone),
    ),
    (
        b"noforcefeedback",
        0x4F68F4,
        dword!(o => o.input.devices.no_force_feedback),
    ),
    (b"joyffgain", 0x4F68F8, dword!(o => o.input.devices.ff_gain)),
    (
        b"mergejoy",
        0x4F68FC,
        dword!(o => o.input.devices.two_devices_one_player),
    ),
    (b"nonetwork", 0x52A4B0, dword!(o => o.net.vars.receive_done)),
    (b"inetport", 0x52AC24, dword!(o => o.net.setup().port)),
    (b"screensnapshot", 0x4F52B4, dword!(o => o.gfx.shots.on)),
    (
        b"framecounter",
        0x5EB100,
        dword!(o => o.race.frame.counter().show),
    ),
    (
        b"noreplay",
        0x5BEC7C,
        dword!(o => o.race.shared.replay_file().no_replay_switch),
    ),
    (
        b"replaykeys",
        0x5BECA8,
        dword!(o => o.race.shared.replay_file().viewer_editing),
    ),
    (
        b"logfile",
        0x4E0098,
        dword!(_o => crate::logfile::write_line::state::LEVEL),
    ),
    (b"nofmv", 0x5BE870, dword!(_o => *crate::film::NO_FMV.get())),
    (
        b"key",
        0x4E052C,
        dword!(o => o.race.replay.seal_work().key[0]),
    ),
    (
        b"allsecretcars",
        0x758710,
        dword!(o => o.menus.play.cheats.all_secret_cars),
    ),
    (
        b"secretcar",
        0x758710,
        dword!(o => o.menus.play.cheats.all_secret_cars),
    ),
    (
        b"usesavegamekeys",
        0x755144,
        dword!(o => o.saves.savegame_preview.use_savegame_keys),
    ),
    (
        b"sensiblesaving",
        0x75AAD8,
        dword!(o => o.menus.screens.state.menus.review_kind),
    ),
    (
        b"noprecipitation",
        0x71C800,
        dword!(o => o.race.shared.work.get().no_precipitation),
    ),
    (
        b"seasoncode",
        0x5BECAC,
        dword!(o => o.race.shared.replay_file().season_code),
    ),
    (
        b"noautoneutral",
        0x61416C,
        dword!(o => o.race.shared.physics_tables.get().no_auto_neutral),
    ),
    (
        b"nonlinearsteering",
        0x4F6900,
        dword!(o => o.input.devices.nonlinear_steering),
    ),
    (
        b"wrreplayenable",
        0x5C2CEB,
        Target::Unaligned(set_developer_keys),
    ),
    (
        b"wrreplayrate",
        0x5C2CEF,
        Target::Unaligned(set_picture_rate),
    ),
    (
        b"alt2axcode",
        0x4F68F0,
        dword!(o => o.input.devices.alt_axes_code),
    ),
    (
        b"fizpatch",
        0x71C6D0,
        dword!(o => o.race.shared.work.get().patches[0]),
    ),
    (
        b"patchmaxdrawdistance",
        0x71C6D4,
        dword!(o => o.race.shared.work.get().patches[1]),
    ),
    (
        b"patchstickygrass",
        0x71C6D8,
        dword!(o => o.race.shared.work.get().patches[2]),
    ),
    (
        b"patchgravity",
        0x71C6DC,
        dword!(o => o.race.shared.work.get().patches[3]),
    ),
    (
        b"patchdamages",
        0x71C6E0,
        dword!(o => o.race.shared.work.get().patches[4]),
    ),
    (
        b"patchbrakegrip",
        0x71C6E4,
        dword!(o => o.race.shared.work.get().patches[5]),
    ),
    (
        b"patchaccelgrip",
        0x71C6E8,
        dword!(o => o.race.shared.work.get().patches[6]),
    ),
    (
        b"patchslidegrip",
        0x71C6EC,
        dword!(o => o.race.shared.work.get().patches[7]),
    ),
    (
        b"patchgripgrass",
        0x71C6F0,
        dword!(o => o.race.shared.work.get().patches[8]),
    ),
    (
        b"patchgriptarmac",
        0x71C6F4,
        dword!(o => o.race.shared.work.get().patches[9]),
    ),
    (
        b"patchgripsnow",
        0x71C6F8,
        dword!(o => o.race.shared.work.get().patches[10]),
    ),
    (
        b"patchgripmud",
        0x71C6FC,
        dword!(o => o.race.shared.work.get().patches[11]),
    ),
    (
        b"patchgripgravel",
        0x71C700,
        dword!(o => o.race.shared.work.get().patches[12]),
    ),
    (
        b"lexcontrol",
        0x4E0758,
        dword!(_o => *crate::strings::maybe_compile_lexicon::RECOMPILE_LEVEL.get()),
    ),
    (
        b"sparksmode",
        0x7558DC,
        dword!(o => o.menus.screens.event.sparks_mode),
    ),
];

/// The settings of a `var\ral.cfg` text applied to their variables. Everything is parsed
/// first, so nothing is written if a key is refused.
pub fn apply_settings(o: Owners<'static>, text: &[u8]) -> Result<(), BadInput> {
    let parsed = rc2k_core::files::settings::parse_settings(text, &case_table())?;
    for (setting, number) in parsed {
        ENTRIES[setting.0].2.set(o, number);
    }
    Ok(())
}

pub fn read_settings(o: Owners<'static>) -> Result<(), BadInput> {
    match read_text(o.system, &PATH[..PATH.len() - 1])? {
        Some(text) => apply_settings(o, &text),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::{ENTRIES, Owners, Target};

    /// Where each setting's variable is: its store (a registry name) and the field in it, by
    /// path. The game's table holds the variables' addresses; these are the places of them.
    const PLACES: [(&str, &str, &str); 77] = [
        (
            "notaskswitch",
            "system.owner.SYSTEM.window_loop",
            ".no_task_switch",
        ),
        (
            "noprichange",
            "system.owner.SYSTEM.window_loop",
            ".keep_priority",
        ),
        ("ignoremmx", "system.owner.SYSTEM.machine", ".ignore_mmx"),
        (
            "ignorekatmai",
            "system.owner.SYSTEM.machine",
            ".ignore_katmai",
        ),
        (
            "ignoreamdk3d",
            "system.owner.SYSTEM.machine",
            ".ignore_3dnow",
        ),
        ("altmemenable", "memory.state.OWN_HEAP", ""),
        ("altmemheapsize", "memory.state.OWN_HEAP_KB", ""),
        ("noaudio", "audio.owner.AUDIO_OWNER.state", ".no_audio"),
        ("nosound", "audio.owner.AUDIO_OWNER.state", ".no_audio"),
        (
            "nocdaudio",
            "sound.owner.SOUND.menu.cd_audio",
            ".no_cd_audio",
        ),
        ("nomusic", "audio.owner.AUDIO_OWNER.state", ".no_music"),
        (
            "audiobasefreq",
            "audio.owner.AUDIO_OWNER.state",
            ".mix_rate_setting",
        ),
        (
            "ignoreds3d",
            "audio.owner.AUDIO_OWNER.state",
            ".no_listener",
        ),
        (
            "ignoredsv",
            "audio.owner.AUDIO_OWNER.state",
            ".ignore_dsv_setting",
        ),
        (
            "ignoreeax",
            "audio.owner.AUDIO_OWNER.state",
            ".ignore_eax_setting",
        ),
        (
            "ignorea3d",
            "audio.owner.AUDIO_OWNER.state",
            ".no_a3d_setting",
        ),
        (
            "wavheapsize",
            "audio.owner.AUDIO_OWNER.state",
            ".sample_buffer_kb",
        ),
        (
            "bundleheapsize",
            "gfx.owner.GFX.fonts.glyph_tables",
            ".units",
        ),
        (
            "imageheapsize",
            "strings.owner.STRINGS.lexicon.memory",
            ".kilobytes",
        ),
        (
            "gamegfxheapsize",
            "gfx.owner.GFX.fonts.game_fonts_memory",
            ".units",
        ),
        ("forcedx5", "gfx.owner.GFX.d3d.cards", ".force_dx5"),
        ("forceno3dcard", "gfx.owner.GFX.d3d.cards", ".no_search"),
        ("ignoreprimary", "gfx.owner.GFX.d3d.cards", ".not_primary"),
        (
            "ignoresecondary",
            "gfx.owner.GFX.d3d.cards",
            ".ignore_secondary",
        ),
        ("agpmode", "gfx.owner.GFX.d3d.cards", ".agp_mode"),
        ("force3dtrans", "gfx.owner.GFX.d3d.cards", ".force_trans"),
        ("force3dalpha", "gfx.owner.GFX.d3d.cards", ".force_alpha"),
        ("force3dshadow", "gfx.owner.GFX.d3d.cards", ".force_shadow"),
        (
            "force3dfogging",
            "gfx.owner.GFX.d3d.cards",
            ".force_fogging",
        ),
        ("textflashmode", "strings.owner.STRINGS.text.blend_mode", ""),
        ("videomodemain", "gfx.owner.GFX.display.mode", ".menu_mode"),
        ("videomodegame", "gfx.owner.GFX.display.mode", ".race_mode"),
        (
            "v3driverfix",
            "scene.owner.SCENE.drawing.scene",
            ".v3_driver_fix",
        ),
        ("nopngdecode", "gfx.owner.GFX.png.setting", ""),
        ("nojoy", "input.owner.INPUT.devices", ".no_joysticks"),
        ("joymode", "input.owner.INPUT.devices", ".joy_mode"),
        ("joycode", "input.owner.INPUT.devices", ".joy_code"),
        ("joylowerdz", "input.owner.INPUT.devices", ".joy_dead_zone"),
        ("joyupperdz", "input.owner.INPUT.devices", ".joy_end_zone"),
        (
            "noforcefeedback",
            "input.owner.INPUT.devices",
            ".no_force_feedback",
        ),
        ("joyffgain", "input.owner.INPUT.devices", ".ff_gain"),
        (
            "mergejoy",
            "input.owner.INPUT.devices",
            ".two_devices_one_player",
        ),
        ("nonetwork", "net.owner.NET_OWNER.vars", ".receive_done"),
        ("inetport", "net.owner.NET_OWNER.main.setup", ".port"),
        ("screensnapshot", "gfx.owner.GFX.shots", ".on"),
        ("framecounter", "race.owner.RACE.frame.counter", ".show"),
        (
            "noreplay",
            "race.owner.RACE.shared.replay_file",
            ".no_replay_switch",
        ),
        (
            "replaykeys",
            "race.owner.RACE.shared.replay_file",
            ".viewer_editing",
        ),
        ("logfile", "logfile.write_line.state.LEVEL", ""),
        ("nofmv", "film.NO_FMV", ""),
        ("key", "race.owner.RACE.replay.seal_work", ".key[0]"),
        (
            "allsecretcars",
            "menu.owner.MENU_OWNER.play.cheats",
            ".all_secret_cars",
        ),
        (
            "secretcar",
            "menu.owner.MENU_OWNER.play.cheats",
            ".all_secret_cars",
        ),
        (
            "usesavegamekeys",
            "saves.owner.SAVES.savegame_preview",
            ".use_savegame_keys",
        ),
        (
            "sensiblesaving",
            "menu.owner.MENU_OWNER.screens.state",
            ".menus.review_kind",
        ),
        (
            "noprecipitation",
            "race.owner.RACE.shared.work",
            ".no_precipitation",
        ),
        (
            "seasoncode",
            "race.owner.RACE.shared.replay_file",
            ".season_code",
        ),
        (
            "noautoneutral",
            "race.owner.RACE.shared.physics_tables",
            ".no_auto_neutral",
        ),
        (
            "nonlinearsteering",
            "input.owner.INPUT.devices",
            ".nonlinear_steering",
        ),
        (
            "wrreplayenable",
            "race.owner.RACE.replay.viewer_and_kinds",
            ".viewer_panel.untyped_tail",
        ),
        (
            "wrreplayrate",
            "race.owner.RACE.replay.viewer_and_kinds",
            ".viewer_panel.untyped_tail",
        ),
        ("alt2axcode", "input.owner.INPUT.devices", ".alt_axes_code"),
        ("fizpatch", "race.owner.RACE.shared.work", ".patches[0]"),
        (
            "patchmaxdrawdistance",
            "race.owner.RACE.shared.work",
            ".patches[1]",
        ),
        (
            "patchstickygrass",
            "race.owner.RACE.shared.work",
            ".patches[2]",
        ),
        ("patchgravity", "race.owner.RACE.shared.work", ".patches[3]"),
        ("patchdamages", "race.owner.RACE.shared.work", ".patches[4]"),
        (
            "patchbrakegrip",
            "race.owner.RACE.shared.work",
            ".patches[5]",
        ),
        (
            "patchaccelgrip",
            "race.owner.RACE.shared.work",
            ".patches[6]",
        ),
        (
            "patchslidegrip",
            "race.owner.RACE.shared.work",
            ".patches[7]",
        ),
        (
            "patchgripgrass",
            "race.owner.RACE.shared.work",
            ".patches[8]",
        ),
        (
            "patchgriptarmac",
            "race.owner.RACE.shared.work",
            ".patches[9]",
        ),
        (
            "patchgripsnow",
            "race.owner.RACE.shared.work",
            ".patches[10]",
        ),
        (
            "patchgripmud",
            "race.owner.RACE.shared.work",
            ".patches[11]",
        ),
        (
            "patchgripgravel",
            "race.owner.RACE.shared.work",
            ".patches[12]",
        ),
        (
            "lexcontrol",
            "strings.maybe_compile_lexicon.RECOMPILE_LEVEL",
            "",
        ),
        (
            "sparksmode",
            "menu.owner.MENU_OWNER.screens.event",
            ".sparks_mode",
        ),
    ];

    #[test]
    fn targets_are_the_variables_the_games_table_names() {
        for (key, _, target) in ENTRIES {
            let name = String::from_utf8_lossy(key);
            let cell: *const () = match target {
                Target::Dword(cell) => std::ptr::from_ref(cell(Owners::the())).cast(),
                // Their cells are bytes of a tail: checked by writing, in the test below.
                Target::Unaligned(_) => continue,
            };
            let (_, store, path) = PLACES
                .iter()
                .find(|(k, _, _)| k.as_bytes() == key)
                .unwrap_or_else(|| panic!("no place for {name}"));
            let first = crate::test_support::state::walk(store).first_addr(path);
            assert_eq!(Some(cell.addr()), first, "{name}");
        }
    }

    /// The two unaligned settings write the four bytes at their place in the viewer panel's
    /// untyped tail (bytes 3 and 7 of it, as in the game's image) and only those.
    #[test]
    fn unaligned_targets_write_their_bytes() {
        use crate::test_support::state::{Snapshot, walk};
        let _lock = crate::test_support::test_lock();
        for (key, _, target) in ENTRIES {
            if !matches!(target, Target::Unaligned(_)) {
                continue;
            }
            let (_, name, path) = PLACES.iter().find(|(k, _, _)| k.as_bytes() == key).unwrap();
            let at = match key {
                b"wrreplayenable" => 3,
                b"wrreplayrate" => 7,
                _ => unreachable!("an unaligned setting without a place"),
            };
            let snapshot = Snapshot::of(&[name]);
            let tail = walk(name);
            let before = tail.path_bytes(path).unwrap();
            target.set(Owners::the(), 0x0A0B_0C0D);
            let after = tail.path_bytes(path).unwrap();
            snapshot.restore();
            assert_eq!(
                after[at..at + 4],
                0x0A0B_0C0Du32.to_le_bytes(),
                "{}",
                String::from_utf8_lossy(key)
            );
            assert_eq!(&before[..at], &after[..at]);
            assert_eq!(&before[at + 4..], &after[at + 4..]);
        }
    }

    #[test]
    fn table_reads_whole() {
        let table = rc2k_core::files::settings::SETTINGS;
        assert_eq!(table.len(), super::ENTRIES.len());
    }

    /// The targets are in the key list's order, with the kinds it names.
    #[test]
    fn table_matches_the_keys() {
        use rc2k_core::files::settings::Kind;
        for (n, (key, _, target)) in ENTRIES.iter().enumerate() {
            let setting = rc2k_core::files::settings::Setting(n);
            assert_eq!(setting.name(), *key);
            let kind = match target {
                Target::Dword(_) => Kind::Dword,
                Target::Unaligned(_) => Kind::Unaligned,
            };
            assert_eq!(setting.kind(), kind, "{}", String::from_utf8_lossy(key));
        }
    }

    /// The variables of every setting that is a whole dword, as they are now.
    fn dword_values(o: Owners<'static>) -> Vec<Option<u32>> {
        ENTRIES
            .iter()
            .map(|(_, _, target)| match target {
                Target::Dword(cell) => Some(cell(o).get()),
                Target::Unaligned(_) => None,
            })
            .collect()
    }

    /// The two unaligned settings' variables, as they are now.
    fn unaligned_values(o: Owners<'static>) -> [u32; 2] {
        let panel = o.race.viewer_panel();
        [panel.developer_keys(), panel.picture_rate()]
    }

    /// A `ral.cfg` line `key=number` (and `key number`) of each of the 77 keys reaches its own
    /// variable: that one (and the key sharing its variable, `noaudio` / `nosound`,
    /// `allsecretcars` / `secretcar`) changes to the number and no other variable moves.
    #[test]
    fn each_key_reaches_its_own_variable_and_no_other() {
        let _lock = crate::test_support::test_lock();
        let o = Owners::the();
        let start = dword_values(o);
        let start_unaligned = unaligned_values(o);
        for form in ["{}={}\r\n", "{} {}\r\n"] {
            for (n, (key, _, target)) in ENTRIES.iter().enumerate() {
                let number = 0x0100_0000 + n as u32 * 0x101;
                let name = String::from_utf8_lossy(key).into_owned();
                let text = form
                    .replacen("{}", &name, 1)
                    .replacen("{}", &number.to_string(), 1);
                let mut bytes = text.into_bytes();
                bytes.push(0);
                super::apply_settings(o, &bytes).unwrap();
                let now = dword_values(o);
                let now_unaligned = unaligned_values(o);
                let mut changed: Vec<String> = Vec::new();
                for (m, (other, _, _)) in ENTRIES.iter().enumerate() {
                    if now[m] != start[m] {
                        assert_eq!(
                            now[m],
                            Some(number),
                            "{name}: {}",
                            String::from_utf8_lossy(other)
                        );
                        changed.push(String::from_utf8_lossy(other).into_owned());
                    }
                }
                match target {
                    Target::Dword(_) => {
                        // A key whose number equals the variable's start value changes nothing.
                        assert!(changed.contains(&name), "{name} did not reach its variable");
                        assert_eq!(now_unaligned, start_unaligned, "{name}");
                    }
                    Target::Unaligned(_) => {
                        assert!(changed.is_empty(), "{name} moved {changed:?}");
                        assert_ne!(now_unaligned, start_unaligned, "{name}");
                    }
                }
                // Each of the two keys sharing a variable moves the other's.
                let shares: Vec<&String> = changed.iter().filter(|k| **k != name).collect();
                let allowed: &[&str] = match name.as_str() {
                    "noaudio" => &["nosound"],
                    "nosound" => &["noaudio"],
                    "allsecretcars" => &["secretcar"],
                    "secretcar" => &["allsecretcars"],
                    _ => &[],
                };
                assert!(
                    shares.iter().all(|k| allowed.contains(&k.as_str())),
                    "{name} also moved {shares:?}"
                );
                for (m, (_, _, other)) in ENTRIES.iter().enumerate() {
                    if let (Target::Dword(cell), Some(value)) = (other, start[m]) {
                        cell(o).set(value);
                    }
                }
                let [keys, rate] = start_unaligned;
                o.race.viewer_panel().set_developer_keys(keys);
                o.race.viewer_panel().set_picture_rate(rate);
            }
        }
        assert_eq!(dword_values(o), start);
        assert_eq!(unaligned_values(o), start_unaligned);
    }

    /// A key a text lacks leaves its variable, a key with no number sets 0, and a text of
    /// several lines sets each of its keys.
    #[test]
    fn absent_keys_stay_and_missing_numbers_are_zero() {
        let _lock = crate::test_support::test_lock();
        let o = Owners::the();
        let start = dword_values(o);
        let start_unaligned = unaligned_values(o);
        super::apply_settings(o, b"ignoremmx=7\r\nignorekatmai\r\nsparksmode 3\r\n\0").unwrap();
        let now = dword_values(o);
        let find = |key: &[u8]| ENTRIES.iter().position(|(k, _, _)| *k == key).unwrap();
        assert_eq!(now[find(b"ignoremmx")], Some(7));
        assert_eq!(now[find(b"ignorekatmai")], Some(0));
        assert_eq!(now[find(b"sparksmode")], Some(3));
        let untouched = [b"notaskswitch".as_slice(), b"noprichange", b"ignoreamdk3d"];
        for key in untouched {
            assert_eq!(
                now[find(key)],
                start[find(key)],
                "{}",
                String::from_utf8_lossy(key)
            );
        }
        for (m, (_, _, target)) in ENTRIES.iter().enumerate() {
            if let (Target::Dword(cell), Some(value)) = (target, start[m]) {
                cell(o).set(value);
            }
        }
        assert_eq!(dword_values(o), start);
        assert_eq!(unaligned_values(o), start_unaligned);
    }
}
