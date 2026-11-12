//! `settings::read_settings` (0x4114A6): reads `var\ral.cfg` into the game's settings.
//!
//! `read_text` on `var\ral.cfg`; if it reads, `settings::apply_settings` (0x4114FF, one caller,
//! here as
//! `apply_settings`): for each of the 77 entries of the table at 0x518C09 (a key, then the address
//! of its variable), if `find_key` finds the key, the variable becomes `parse_number` of its value.
//! A key that is not in the file leaves its variable alone; a key without a number makes it 0. Two
//! keys share a variable twice (`noaudio` and `nosound`, `allsecretcars` and `secretcar`): the
//! later one in the table wins. Callers: `start` and `system::load_game_data`.

use crate::errors::BadInput;
use crate::owners::Owners;
use crate::session::{Cell32, Loose};
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;

// ---- game data ----

/// Where a setting's variable lives now: its store's cell. The table keeps the variable's
/// IDA address too (the table's bytes are the image's; `tests::targets_are_the_addresses`
/// proves each cell is what `crate::moved::place` answers for that address).
#[derive(Clone, Copy)]
enum Target {
    /// A whole aligned dword of a store (`dword!`).
    Dword(fn(Owners<'static>) -> &'static dyn Cell32),
    /// A dword at an address that is not a multiple of 4 (`loose!`).
    Loose(fn(Owners<'static>) -> &'static Loose),
    /// A dword that straddles two cells of its store's untyped tail: the store's own
    /// setter writes the four bytes (`race::viewer_screen_store`'s).
    Unaligned(fn(Owners<'static>, u32)),
}

impl Target {
    /// The four bytes of the variable set.
    fn set(self, o: Owners<'static>, number: u32) {
        match self {
            Target::Dword(cell) => cell(o).set_bits(number),
            Target::Loose(cell) => cell(o).set(number),
            Target::Unaligned(set) => set(o, number),
        }
    }
}

/// `dword!(o => o.owner.field)`: the dword as a `Target` (a `fn` item, so the table stays
/// `const`); `o` names the owners the path starts from.
macro_rules! dword {
    ($o:ident => $($path:tt)+) => {
        Target::Dword({
            fn cell($o: Owners<'static>) -> &'static dyn Cell32 {
                &$($path)+
            }
            cell
        })
    };
}

/// `loose!(o => o.owner.field)`: an unaligned dword as a `Target`.
macro_rules! loose {
    ($o:ident => $($path:tt)+) => {
        Target::Loose({
            fn cell($o: Owners<'static>) -> &'static Loose {
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
/// aNotaskswitch (0x518C09 .. 0x5190F1 in the image): the settings, each a key and the
/// IDA address of its dword and its store's cell; the table's bytes are the keys and
/// addresses, key, NUL, address (`tests::table_is_the_games`).
const ENTRIES: [(&[u8], u32, Target); 77] = [
    (
        b"notaskswitch",
        0x518384,
        dword!(o => o.system.session.flags.no_task_switch),
    ),
    (
        b"noprichange",
        0x518388,
        dword!(o => o.system.session.flags.keep_priority),
    ),
    (
        b"ignoremmx",
        0x5183AC,
        dword!(o => o.system.session.machine.ignore_mmx),
    ),
    (
        b"ignorekatmai",
        0x5183B4,
        dword!(o => o.system.session.machine.ignore_katmai),
    ),
    (
        b"ignoreamdk3d",
        0x5183BC,
        dword!(o => o.system.session.machine.ignore_3dnow),
    ),
    (
        b"altmemenable",
        0x4E0584,
        dword!(_o => crate::memory::state::OWN_HEAP),
    ),
    (
        b"altmemheapsize",
        0x4E0588,
        dword!(_o => crate::memory::state::OWN_HEAP_KB),
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
        loose!(o => o.audio.state.no_listener),
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
    (b"forcedx5", 0x5AB958, dword!(o => o.d3d.cards.force_dx5)),
    (
        b"forceno3dcard",
        0x5AB94C,
        dword!(o => o.d3d.cards.no_search),
    ),
    (
        b"ignoreprimary",
        0x5AB950,
        dword!(o => o.d3d.cards.not_primary),
    ),
    (
        b"ignoresecondary",
        0x5AB954,
        dword!(o => o.d3d.cards.ignore_secondary),
    ),
    (b"agpmode", 0x5AB95C, dword!(o => o.d3d.cards.agp_mode)),
    (
        b"force3dtrans",
        0x5AB97C,
        dword!(o => o.d3d.cards.force_trans),
    ),
    (
        b"force3dalpha",
        0x5AB984,
        dword!(o => o.d3d.cards.force_alpha),
    ),
    (
        b"force3dshadow",
        0x5AB980,
        dword!(o => o.d3d.cards.force_shadow),
    ),
    (
        b"force3dfogging",
        0x5AB988,
        dword!(o => o.d3d.cards.force_fogging),
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
        dword!(o => o.scene.drawing.scene.v3_driver_fix),
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
        dword!(o => o.race.frame.counter.show),
    ),
    (
        b"noreplay",
        0x5BEC7C,
        dword!(o => o.race.shared.replay_file.no_replay_switch),
    ),
    (
        b"replaykeys",
        0x5BECA8,
        dword!(o => o.race.shared.replay_file.viewer_editing),
    ),
    (
        b"logfile",
        0x4E0098,
        dword!(_o => crate::logfile::write_line::state::LEVEL),
    ),
    (b"nofmv", 0x5BE870, dword!(_o => crate::film::NO_FMV)),
    (
        b"key",
        0x4E052C,
        dword!(o => o.race.replay.seal_work.key[0]),
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
        dword!(o => o.race.shared.work.no_precipitation),
    ),
    (
        b"seasoncode",
        0x5BECAC,
        dword!(o => o.race.shared.replay_file.season_code),
    ),
    (
        b"noautoneutral",
        0x61416C,
        dword!(o => o.race.shared.physics_tables.no_auto_neutral),
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
        dword!(o => o.race.shared.work.patch_switch),
    ),
    (
        b"patchmaxdrawdistance",
        0x71C6D4,
        dword!(o => o.race.shared.work.patch_settings[0]),
    ),
    (
        b"patchstickygrass",
        0x71C6D8,
        dword!(o => o.race.shared.work.patch_settings[1]),
    ),
    (
        b"patchgravity",
        0x71C6DC,
        dword!(o => o.race.shared.work.patch_settings[2]),
    ),
    (
        b"patchdamages",
        0x71C6E0,
        dword!(o => o.race.shared.work.patch_settings[3]),
    ),
    (
        b"patchbrakegrip",
        0x71C6E4,
        dword!(o => o.race.shared.work.patch_settings[4]),
    ),
    (
        b"patchaccelgrip",
        0x71C6E8,
        dword!(o => o.race.shared.work.patch_settings[5]),
    ),
    (
        b"patchslidegrip",
        0x71C6EC,
        dword!(o => o.race.shared.work.patch_settings[6]),
    ),
    (
        b"patchgripgrass",
        0x71C6F0,
        dword!(o => o.race.shared.work.patch_settings[7]),
    ),
    (
        b"patchgriptarmac",
        0x71C6F4,
        dword!(o => o.race.shared.work.patch_settings[8]),
    ),
    (
        b"patchgripsnow",
        0x71C6F8,
        dword!(o => o.race.shared.work.patch_settings[9]),
    ),
    (
        b"patchgripmud",
        0x71C6FC,
        dword!(o => o.race.shared.work.patch_settings[10]),
    ),
    (
        b"patchgripgravel",
        0x71C700,
        dword!(o => o.race.shared.work.patch_settings[11]),
    ),
    (
        b"lexcontrol",
        0x4E0758,
        dword!(_o => crate::strings::maybe_compile_lexicon::RECOMPILE_LEVEL),
    ),
    (
        b"sparksmode",
        0x7558DC,
        dword!(o => o.menus.screens.event.sparks_mode),
    ),
];

/// The table as the image held it (before relocation): `rc2k_core::files::settings::table`
/// reads these bytes.
fn table_bytes() -> Vec<u8> {
    let mut out = Vec::new();
    for (key, address, _) in ENTRIES {
        out.extend_from_slice(key);
        out.push(0);
        out.extend_from_slice(&address.to_le_bytes());
    }
    out
}

/// `settings::apply_settings`: the settings a text names. Converted:
/// `rc2k_core::files::settings::table` and
/// `apply`; here the variables (the table's dwords are their addresses) are stored.
fn settings() -> Result<Vec<(Vec<u8>, u32)>, BadInput> {
    // The empty key that ends it (0x5190F1 in the image).
    let mut bytes = table_bytes();
    bytes.push(0);
    Ok(rc2k_core::files::settings::table(
        bytes.as_slice(),
        bytes.len(),
    )?)
}

pub fn apply_settings(o: Owners<'static>, text: &[u8]) -> Result<(), BadInput> {
    let table = settings()?;
    let keys: Vec<&[u8]> = table.iter().map(|(key, _)| key.as_slice()).collect();
    // Found first, stored after: nothing is written if a key is refused.
    let found = rc2k_core::files::settings::apply(text, &keys, &case_table())?;
    for (n, number) in found {
        ENTRIES[n].2.set(o, number);
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

    /// Where each setting's variable is: its store (a registry name) and the byte offset in
    /// it. The game's table holds the variables' addresses; these are the places of them.
    const PLACES: [(&str, &str, usize); 77] = [
        ("notaskswitch", "system.owner.SYSTEM.session.flags", 0x14),
        ("noprichange", "system.owner.SYSTEM.session.flags", 0x18),
        ("ignoremmx", "system.owner.SYSTEM.session.machine", 0x4),
        ("ignorekatmai", "system.owner.SYSTEM.session.machine", 0xc),
        ("ignoreamdk3d", "system.owner.SYSTEM.session.machine", 0x14),
        ("altmemenable", "memory.state.OWN_HEAP", 0x0),
        ("altmemheapsize", "memory.state.OWN_HEAP_KB", 0x0),
        ("noaudio", "audio.owner.AUDIO_OWNER.state", 0x20),
        ("nosound", "audio.owner.AUDIO_OWNER.state", 0x20),
        ("nocdaudio", "sound.owner.SOUND.menu.cd_audio", 0x0),
        ("nomusic", "audio.owner.AUDIO_OWNER.state", 0xe728),
        ("audiobasefreq", "audio.owner.AUDIO_OWNER.state", 0x40),
        ("ignoreds3d", "audio.owner.AUDIO_OWNER.state", 0xe8ae),
        ("ignoredsv", "audio.owner.AUDIO_OWNER.state", 0xe9bc),
        ("ignoreeax", "audio.owner.AUDIO_OWNER.state", 0xe9d8),
        ("ignorea3d", "audio.owner.AUDIO_OWNER.state", 0xe9c8),
        ("wavheapsize", "audio.owner.AUDIO_OWNER.state", 0xe658),
        ("bundleheapsize", "gfx.owner.GFX.fonts.glyph_tables", 0x0),
        ("imageheapsize", "strings.owner.STRINGS.lexicon.memory", 0x0),
        (
            "gamegfxheapsize",
            "gfx.owner.GFX.fonts.game_fonts_memory",
            0x0,
        ),
        ("forcedx5", "d3d.owner.D3D.cards", 0x4a4),
        ("forceno3dcard", "d3d.owner.D3D.cards", 0x498),
        ("ignoreprimary", "d3d.owner.D3D.cards", 0x49c),
        ("ignoresecondary", "d3d.owner.D3D.cards", 0x4a0),
        ("agpmode", "d3d.owner.D3D.cards", 0x4a8),
        ("force3dtrans", "d3d.owner.D3D.cards", 0x4c8),
        ("force3dalpha", "d3d.owner.D3D.cards", 0x4d0),
        ("force3dshadow", "d3d.owner.D3D.cards", 0x4cc),
        ("force3dfogging", "d3d.owner.D3D.cards", 0x4d4),
        (
            "textflashmode",
            "strings.owner.STRINGS.text.blend_mode",
            0x0,
        ),
        ("videomodemain", "gfx.owner.GFX.display.mode", 0x18),
        ("videomodegame", "gfx.owner.GFX.display.mode", 0x1c),
        ("v3driverfix", "scene.owner.SCENE.drawing.scene", 0x2b8),
        ("nopngdecode", "gfx.owner.GFX.png.setting", 0x0),
        ("nojoy", "input.owner.INPUT.devices", 0x4),
        ("joymode", "input.owner.INPUT.devices", 0x8),
        ("joycode", "input.owner.INPUT.devices", 0xc),
        ("joylowerdz", "input.owner.INPUT.devices", 0x10),
        ("joyupperdz", "input.owner.INPUT.devices", 0x14),
        ("noforcefeedback", "input.owner.INPUT.devices", 0x1c),
        ("joyffgain", "input.owner.INPUT.devices", 0x20),
        ("mergejoy", "input.owner.INPUT.devices", 0x24),
        ("nonetwork", "net.owner.NET_OWNER.vars", 0x0),
        ("inetport", "net.owner.NET_OWNER.main.setup", 0xb4),
        ("screensnapshot", "gfx.owner.GFX.shots", 0x0),
        ("framecounter", "race.owner.RACE.frame.counter", 0x0),
        ("noreplay", "race.owner.RACE.shared.replay_file", 0x4),
        ("replaykeys", "race.owner.RACE.shared.replay_file", 0x30),
        ("logfile", "logfile.write_line.state.LEVEL", 0x0),
        ("nofmv", "film.NO_FMV", 0x0),
        ("key", "race.owner.RACE.replay.seal_work", 0x80),
        ("allsecretcars", "menu.owner.MENUS.play.cheats", 0x138),
        ("secretcar", "menu.owner.MENUS.play.cheats", 0x138),
        ("usesavegamekeys", "saves.owner.SAVES.savegame_preview", 0x0),
        ("sensiblesaving", "menu.owner.MENUS.screens.state", 0x480),
        ("noprecipitation", "race.owner.RACE.shared.work", 0x1848),
        ("seasoncode", "race.owner.RACE.shared.replay_file", 0x34),
        (
            "noautoneutral",
            "race.owner.RACE.shared.physics_tables",
            0x944,
        ),
        ("nonlinearsteering", "input.owner.INPUT.devices", 0x28),
        (
            "wrreplayenable",
            "race.owner.RACE.replay.viewer_and_kinds",
            0x3cf,
        ),
        (
            "wrreplayrate",
            "race.owner.RACE.replay.viewer_and_kinds",
            0x3d3,
        ),
        ("alt2axcode", "input.owner.INPUT.devices", 0x18),
        ("fizpatch", "race.owner.RACE.shared.work", 0x1718),
        (
            "patchmaxdrawdistance",
            "race.owner.RACE.shared.work",
            0x171c,
        ),
        ("patchstickygrass", "race.owner.RACE.shared.work", 0x1720),
        ("patchgravity", "race.owner.RACE.shared.work", 0x1724),
        ("patchdamages", "race.owner.RACE.shared.work", 0x1728),
        ("patchbrakegrip", "race.owner.RACE.shared.work", 0x172c),
        ("patchaccelgrip", "race.owner.RACE.shared.work", 0x1730),
        ("patchslidegrip", "race.owner.RACE.shared.work", 0x1734),
        ("patchgripgrass", "race.owner.RACE.shared.work", 0x1738),
        ("patchgriptarmac", "race.owner.RACE.shared.work", 0x173c),
        ("patchgripsnow", "race.owner.RACE.shared.work", 0x1740),
        ("patchgripmud", "race.owner.RACE.shared.work", 0x1744),
        ("patchgripgravel", "race.owner.RACE.shared.work", 0x1748),
        (
            "lexcontrol",
            "strings.maybe_compile_lexicon.RECOMPILE_LEVEL",
            0x0,
        ),
        ("sparksmode", "menu.owner.MENUS.screens.event", 0x18),
    ];

    #[test]
    fn targets_are_the_variables_the_games_table_names() {
        for (key, _, target) in ENTRIES {
            let name = String::from_utf8_lossy(key);
            let cell: *const () = match target {
                Target::Dword(cell) => std::ptr::from_ref(cell(Owners::the())).cast(),
                Target::Loose(cell) => std::ptr::from_ref(cell(Owners::the())).cast(),
                // Their cells are bytes of a tail: checked by writing, in the test below.
                Target::Unaligned(_) => continue,
            };
            let (_, store, offset) = PLACES
                .iter()
                .find(|(k, _, _)| k.as_bytes() == key)
                .unwrap_or_else(|| panic!("no place for {name}"));
            let first = crate::test_support::state::store(store).first().addr();
            assert_eq!(cell.addr(), first + offset, "{name}");
        }
    }

    /// The two unaligned settings write the four bytes at their address (and only those).
    #[test]
    fn unaligned_targets_write_their_bytes() {
        use crate::test_support::state::{Snapshot, store};
        let _lock = crate::store::test_lock();
        for (key, _, target) in ENTRIES {
            if !matches!(target, Target::Unaligned(_)) {
                continue;
            }
            let (_, name, offset) = PLACES.iter().find(|(k, _, _)| k.as_bytes() == key).unwrap();
            let snapshot = Snapshot::of(&[name]);
            let store = store(name);
            let read = |at: usize, len: usize| {
                let mut bytes = vec![0; len];
                store.load(at, &mut bytes).unwrap();
                bytes
            };
            let neighbours = read(offset - 4, 12);
            target.set(Owners::the(), 0x0A0B_0C0D);
            let after = read(*offset, 4);
            let neighbours_after = read(offset - 4, 12);
            snapshot.restore();
            assert_eq!(
                after,
                0x0A0B_0C0Du32.to_le_bytes(),
                "{}",
                String::from_utf8_lossy(key)
            );
            assert_eq!(&neighbours[..4], &neighbours_after[..4]);
            assert_eq!(&neighbours[8..], &neighbours_after[8..]);
        }
    }

    #[test]
    fn table_is_the_games() {
        let bytes = super::table_bytes();
        crate::test_support::first_values::assert_matches(
            "settings.read_settings.TABLE/1256",
            &bytes,
        );
        assert_eq!(
            crate::test_support::first_values::bytes("settings.read_settings.PATH"),
            &super::PATH[..]
        );
        // The empty key that ends the table.
        assert_eq!(
            crate::test_support::first_values::bytes("settings.read_settings.TABLE_END/1"),
            [0]
        );
    }

    #[test]
    fn table_reads_whole() {
        let table = { super::settings() }.unwrap();
        assert_eq!(table.len(), super::ENTRIES.len());
    }
}
