//! Reads `var\ral.cfg` and stores each setting it names in its variable; this file holds the table
//! of those variables, the key list and parser are `rc2k_core::files::settings`.

use crate::errors::BadInput;
use crate::memory::session::DwordLike;
use crate::owners::Owners;
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;

// ---- game data ----

/// Where a setting's variable lives: its store's cell
/// (`tests::targets_are_the_variables_the_games_table_names` checks each cell).
#[derive(Clone, Copy)]
enum Target {
    /// A whole aligned dword of a store (`dword!`).
    Dword(fn(Owners<'static>) -> &'static dyn DwordLike),
    /// A field of a store of plain values, read and written by functions (`plain!`).
    Plain {
        get: fn(Owners<'static>) -> u32,
        set: fn(Owners<'static>, u32),
    },
    /// A dword that straddles two cells of its store's untyped tail: the store's own
    /// setter writes the four bytes (`race::viewer_screen_store`'s).
    Unaligned(fn(Owners<'static>, u32)),
    /// A key that is read and has no variable: a setting for the 1999 game's machines (its
    /// own heap, the process's priority, CD audio, the search for 3D cards, the processor's
    /// extensions).
    Ignored,
}

impl Target {
    /// The variable's number now; none for a key that has no variable here.
    fn get(self, o: Owners<'static>) -> Option<u32> {
        match self {
            Target::Dword(cell) => Some(cell(o).get()),
            Target::Plain { get, .. } => Some(get(o)),
            Target::Unaligned(_) | Target::Ignored => None,
        }
    }

    /// The four bytes of the variable set.
    fn set(self, o: Owners<'static>, number: u32) {
        match self {
            Target::Dword(cell) => cell(o).set(number),
            Target::Plain { set, .. } => set(o, number),
            Target::Unaligned(set) => set(o, number),
            Target::Ignored => {}
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

/// `plain_cell!(o => o.owner.field)`: a plain `Cell<u32>` as a `Target`.
macro_rules! plain_cell {
    ($o:ident => $($path:tt)+) => {
        Target::Plain {
            get: {
                fn get($o: Owners<'static>) -> u32 {
                    $($path)+.get()
                }
                get
            },
            set: {
                fn set($o: Owners<'static>, number: u32) {
                    $($path)+.set(number);
                }
                set
            },
        }
    };
}

/// `plain!(o => o.owner.store, field)`: a field of a store that keeps plain values behind
/// `read()` / `write()`, as a `Target`.
macro_rules! plain {
    ($o:ident => $store:expr; $($field:tt)+) => {
        Target::Plain {
            get: {
                fn get($o: Owners<'static>) -> u32 {
                    $store.read().$($field)+
                }
                get
            },
            set: {
                fn set($o: Owners<'static>, number: u32) {
                    $store.write().$($field)+ = number;
                }
                set
            },
        }
    };
}

/// `framecounter`: whether the frame rate is shown.
fn frame_counter_shown(o: Owners<'static>) -> u32 {
    o.race.frame.counter().show
}

fn set_frame_counter_shown(o: Owners<'static>, number: u32) {
    o.race.frame.counter_mut().show = number;
}

/// `key`: the first dword of the replay seal's key.
fn seal_key_first(o: Owners<'static>) -> u32 {
    o.race.replay.seal_work().key[0]
}

fn set_seal_key_first(o: Owners<'static>, number: u32) {
    o.race.replay.seal_work_mut().key[0] = number;
}

/// `wrreplayenable`: `ViewerAndKinds.viewer_panel.untyped_tail[0]` (unaligned, in the viewer panel's tail).
fn set_developer_keys(o: Owners<'static>, number: u32) {
    o.race.viewer_panel().set_developer_keys(number);
}

/// `wrreplayrate`: `ViewerAndKinds.viewer_panel.untyped_tail[1]` (unaligned, in the viewer panel's tail).
fn set_picture_rate(o: Owners<'static>, number: u32) {
    o.race.viewer_panel().set_picture_rate(number);
}

/// The settings file.
static PATH: &[u8; 12] = b"var\\ral.cfg\0";
/// The settings in `rc2k_core::files::settings::SETTINGS` order (`tests::table_matches_the_keys`
/// checks it): key and target.
const ENTRIES: [(&[u8], Target); 77] = [
    (
        b"notaskswitch",
        dword!(o => o.system.window_loop().no_task_switch),
    ),
    (b"noprichange", Target::Ignored),
    (b"ignoremmx", Target::Ignored),
    (b"ignorekatmai", Target::Ignored),
    (b"ignoreamdk3d", Target::Ignored),
    (b"altmemenable", Target::Ignored),
    (b"altmemheapsize", Target::Ignored),
    (b"noaudio", dword!(o => o.audio.state.no_audio)),
    (b"nosound", dword!(o => o.audio.state.no_audio)),
    (b"nocdaudio", Target::Ignored),
    (b"nomusic", dword!(o => o.audio.state.no_music)),
    (
        b"audiobasefreq",
        dword!(o => o.audio.state.mix_rate_setting),
    ),
    (b"ignoreds3d", dword!(o => o.audio.state.no_listener)),
    (b"ignoredsv", dword!(o => o.audio.state.ignore_dsv_setting)),
    (b"ignoreeax", dword!(o => o.audio.state.ignore_eax_setting)),
    (b"ignorea3d", dword!(o => o.audio.state.ignore_a3d_setting)),
    (b"wavheapsize", dword!(o => o.audio.state.sample_buffer_kb)),
    (
        b"bundleheapsize",
        plain_cell!(o => o.gfx.fonts.glyph_tables.units),
    ),
    (
        b"imageheapsize",
        dword!(o => o.strings.lexicon.memory.kilobytes),
    ),
    (
        b"gamegfxheapsize",
        plain_cell!(o => o.gfx.fonts.game_fonts_memory.units),
    ),
    (
        b"forcedx5",
        plain_cell!(o => o.gfx.d3d.cards.get().force_dx5),
    ),
    (b"forceno3dcard", Target::Ignored),
    (b"ignoreprimary", Target::Ignored),
    (b"ignoresecondary", Target::Ignored),
    (b"agpmode", plain_cell!(o => o.gfx.d3d.cards.get().agp_mode)),
    (
        b"force3dtrans",
        plain_cell!(o => o.gfx.d3d.cards.get().force_trans),
    ),
    (
        b"force3dalpha",
        plain_cell!(o => o.gfx.d3d.cards.get().force_alpha),
    ),
    (
        b"force3dshadow",
        plain_cell!(o => o.gfx.d3d.cards.get().force_shadow),
    ),
    (
        b"force3dfogging",
        plain_cell!(o => o.gfx.d3d.cards.get().force_fogging),
    ),
    (b"textflashmode", dword!(o => o.strings.text.blend_mode)),
    (
        b"videomodemain",
        plain_cell!(o => o.gfx.display.mode.menu_mode),
    ),
    (
        b"videomodegame",
        plain_cell!(o => o.gfx.display.mode.race_mode),
    ),
    (
        b"v3driverfix",
        plain_cell!(o => o.scene.drawing.v3_driver_fix.get()),
    ),
    (b"nopngdecode", dword!(o => o.gfx.png.setting)),
    (b"nojoy", dword!(o => o.input.devices.no_joysticks)),
    (b"joymode", dword!(o => o.input.devices.joy_mode)),
    (b"joycode", dword!(o => o.input.devices.joy_code)),
    (b"joylowerdz", dword!(o => o.input.devices.joy_dead_zone)),
    (b"joyupperdz", dword!(o => o.input.devices.joy_end_zone)),
    (
        b"noforcefeedback",
        dword!(o => o.input.devices.no_force_feedback),
    ),
    (b"joyffgain", dword!(o => o.input.devices.ff_gain)),
    (
        b"mergejoy",
        dword!(o => o.input.devices.two_devices_one_player),
    ),
    (b"nonetwork", dword!(o => o.net.vars.receive_done)),
    (b"inetport", dword!(o => o.net.setup().port)),
    (b"screensnapshot", plain_cell!(o => o.gfx.shots.on)),
    (
        b"framecounter",
        Target::Plain {
            get: frame_counter_shown,
            set: set_frame_counter_shown,
        },
    ),
    (
        b"noreplay",
        dword!(o => o.race.shared.replay_file().no_replay_switch),
    ),
    (
        b"replaykeys",
        dword!(o => o.race.shared.replay_file().viewer_editing),
    ),
    (
        b"logfile",
        dword!(_o => crate::logfile::write_line::state::LEVEL),
    ),
    (b"nofmv", dword!(o => o.system.machine().no_fmv)),
    (
        b"key",
        Target::Plain {
            get: seal_key_first,
            set: set_seal_key_first,
        },
    ),
    (
        b"allsecretcars",
        plain!(o => o.menus.play.cheats; all_secret_cars),
    ),
    (
        b"secretcar",
        plain!(o => o.menus.play.cheats; all_secret_cars),
    ),
    (
        b"usesavegamekeys",
        plain_cell!(o => o.saves.savegame_preview.use_savegame_keys),
    ),
    (
        b"sensiblesaving",
        plain!(o => o.menus.screens.state; menus.review_kind),
    ),
    (
        b"noprecipitation",
        dword!(o => o.race.shared.work.get().no_precipitation),
    ),
    (
        b"seasoncode",
        dword!(o => o.race.shared.replay_file().season_code),
    ),
    (
        b"noautoneutral",
        dword!(o => o.race.shared.physics_tables.get().no_auto_neutral),
    ),
    (
        b"nonlinearsteering",
        dword!(o => o.input.devices.nonlinear_steering),
    ),
    (b"wrreplayenable", Target::Unaligned(set_developer_keys)),
    (b"wrreplayrate", Target::Unaligned(set_picture_rate)),
    (b"alt2axcode", dword!(o => o.input.devices.alt_axes_code)),
    (
        b"fizpatch",
        dword!(o => o.race.shared.work.get().patches[0]),
    ),
    (
        b"patchmaxdrawdistance",
        dword!(o => o.race.shared.work.get().patches[1]),
    ),
    (
        b"patchstickygrass",
        dword!(o => o.race.shared.work.get().patches[2]),
    ),
    (
        b"patchgravity",
        dword!(o => o.race.shared.work.get().patches[3]),
    ),
    (
        b"patchdamages",
        dword!(o => o.race.shared.work.get().patches[4]),
    ),
    (
        b"patchbrakegrip",
        dword!(o => o.race.shared.work.get().patches[5]),
    ),
    (
        b"patchaccelgrip",
        dword!(o => o.race.shared.work.get().patches[6]),
    ),
    (
        b"patchslidegrip",
        dword!(o => o.race.shared.work.get().patches[7]),
    ),
    (
        b"patchgripgrass",
        dword!(o => o.race.shared.work.get().patches[8]),
    ),
    (
        b"patchgriptarmac",
        dword!(o => o.race.shared.work.get().patches[9]),
    ),
    (
        b"patchgripsnow",
        dword!(o => o.race.shared.work.get().patches[10]),
    ),
    (
        b"patchgripmud",
        dword!(o => o.race.shared.work.get().patches[11]),
    ),
    (
        b"patchgripgravel",
        dword!(o => o.race.shared.work.get().patches[12]),
    ),
    (
        b"lexcontrol",
        dword!(o => o.strings.lexicon.recompile_level),
    ),
    (
        b"sparksmode",
        plain!(o => o.menus.screens.event; sparks_mode),
    ),
];

/// The settings of a `var\ral.cfg` text applied to their variables. Everything is parsed
/// first, so nothing is written if a key is refused.
pub fn apply_settings(o: Owners<'static>, text: &[u8]) -> Result<(), BadInput> {
    let parsed = rc2k_core::files::settings::parse_settings(text, &case_table())?;
    for (setting, number) in parsed {
        ENTRIES[setting.0].1.set(o, number);
    }
    Ok(())
}

/// The number of the variable `key` names (lower case) now, if it has one here.
pub fn value_of(o: Owners<'static>, key: &str) -> Option<u32> {
    let (_, target) = ENTRIES.iter().find(|(k, _)| *k == key.as_bytes())?;
    target.get(o)
}

/// Sets the variable `key` names to `number`; false when there is no such key.
pub fn set_value(o: Owners<'static>, key: &str, number: u32) -> bool {
    match ENTRIES.iter().find(|(k, _)| *k == key.as_bytes()) {
        Some((_, target)) => {
            target.set(o, number);
            true
        }
        None => false,
    }
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
    const PLACES: [(&str, &str, &str); 31] = [
        (
            "notaskswitch",
            "owners.system.window_loop",
            ".no_task_switch",
        ),
        ("imageheapsize", "owners.strings.lexicon.kilobytes", ""),
        ("textflashmode", "owners.strings.text.blend_mode", ""),
        ("nopngdecode", "owners.gfx.png.setting", ""),
        ("nonetwork", "owners.net.vars.receive_done", ""),
        ("inetport", "owners.net.main.setup.port", ""),
        (
            "noreplay",
            "owners.race.shared.replay_file.no_replay_switch",
            "",
        ),
        (
            "replaykeys",
            "owners.race.shared.replay_file.viewer_editing",
            "",
        ),
        ("logfile", "logfile.write_line.state.LEVEL", ""),
        ("nofmv", "owners.system.machine", ".no_fmv"),
        (
            "sensiblesaving",
            "owners.menus.screens.state",
            ".menus.review_kind",
        ),
        (
            "noprecipitation",
            "owners.race.shared.work",
            ".no_precipitation",
        ),
        (
            "seasoncode",
            "owners.race.shared.replay_file.season_code",
            "",
        ),
        (
            "noautoneutral",
            "owners.race.shared.physics_tables",
            ".no_auto_neutral",
        ),
        (
            "wrreplayenable",
            "owners.race.replay.viewer_panel.untyped_tail",
            "",
        ),
        (
            "wrreplayrate",
            "owners.race.replay.viewer_panel.untyped_tail",
            "",
        ),
        ("fizpatch", "owners.race.shared.work", ".patches[0]"),
        (
            "patchmaxdrawdistance",
            "owners.race.shared.work",
            ".patches[1]",
        ),
        ("patchstickygrass", "owners.race.shared.work", ".patches[2]"),
        ("patchgravity", "owners.race.shared.work", ".patches[3]"),
        ("patchdamages", "owners.race.shared.work", ".patches[4]"),
        ("patchbrakegrip", "owners.race.shared.work", ".patches[5]"),
        ("patchaccelgrip", "owners.race.shared.work", ".patches[6]"),
        ("patchslidegrip", "owners.race.shared.work", ".patches[7]"),
        ("patchgripgrass", "owners.race.shared.work", ".patches[8]"),
        ("patchgriptarmac", "owners.race.shared.work", ".patches[9]"),
        ("patchgripsnow", "owners.race.shared.work", ".patches[10]"),
        ("patchgripmud", "owners.race.shared.work", ".patches[11]"),
        ("patchgripgravel", "owners.race.shared.work", ".patches[12]"),
        ("lexcontrol", "owners.strings.lexicon.recompile_level", ""),
        ("sparksmode", "owners.menus.screens.event", ".sparks_mode"),
    ];

    /// The settings whose variables are atomic fields of the audio and input threads' shared
    /// records, which no registry store lists: the table's functions are their only places.
    const THREAD_SHARED: [&str; 19] = [
        "noaudio",
        "nosound",
        "nomusic",
        "audiobasefreq",
        "ignoreds3d",
        "ignoredsv",
        "ignoreeax",
        "ignorea3d",
        "wavheapsize",
        "nojoy",
        "joymode",
        "joycode",
        "joylowerdz",
        "joyupperdz",
        "noforcefeedback",
        "joyffgain",
        "mergejoy",
        "nonlinearsteering",
        "alt2axcode",
    ];

    #[test]
    fn targets_are_the_variables_the_games_table_names() {
        for (key, target) in ENTRIES {
            let name = String::from_utf8_lossy(key);
            let cell: *const () = match target {
                Target::Dword(cell) => std::ptr::from_ref(cell(Owners::the())).cast(),
                // Plain fields have no cell; the tests below check them by writing.
                Target::Plain { .. } => continue,
                // Their cells are bytes of a tail: checked by writing, in the test below.
                Target::Unaligned(_) | Target::Ignored => continue,
            };
            if THREAD_SHARED.contains(&&*name) {
                continue;
            }
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
        for (key, target) in ENTRIES {
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
        for (n, (key, target)) in ENTRIES.iter().enumerate() {
            let setting = rc2k_core::files::settings::Setting(n);
            assert_eq!(setting.name(), *key);
            let kind = match target {
                Target::Dword(_) | Target::Plain { .. } => Kind::Dword,
                Target::Unaligned(_) => Kind::Unaligned,
                Target::Ignored => Kind::Dword,
            };
            assert_eq!(setting.kind(), kind, "{}", String::from_utf8_lossy(key));
        }
    }

    /// The variables of every setting that is a whole dword, as they are now.
    fn dword_values(o: Owners<'static>) -> Vec<Option<u32>> {
        ENTRIES.iter().map(|(_, target)| target.get(o)).collect()
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
            for (n, (key, target)) in ENTRIES.iter().enumerate() {
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
                for (m, (other, _)) in ENTRIES.iter().enumerate() {
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
                    Target::Dword(_) | Target::Plain { .. } => {
                        // A key whose number equals the variable's start value changes nothing.
                        assert!(changed.contains(&name), "{name} did not reach its variable");
                        assert_eq!(now_unaligned, start_unaligned, "{name}");
                    }
                    Target::Unaligned(_) => {
                        assert!(changed.is_empty(), "{name} moved {changed:?}");
                        assert_ne!(now_unaligned, start_unaligned, "{name}");
                    }
                    Target::Ignored => {
                        assert!(changed.is_empty(), "{name} moved {changed:?}");
                        assert_eq!(now_unaligned, start_unaligned, "{name}");
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
                for (m, (_, other)) in ENTRIES.iter().enumerate() {
                    if let Some(value) = start[m] {
                        other.set(o, value);
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
        super::apply_settings(o, b"nomusic=7\r\nforcedx5\r\nsparksmode 3\r\n\0").unwrap();
        let now = dword_values(o);
        let find = |key: &[u8]| ENTRIES.iter().position(|(k, _)| *k == key).unwrap();
        assert_eq!(now[find(b"nomusic")], Some(7));
        assert_eq!(now[find(b"forcedx5")], Some(0));
        assert_eq!(now[find(b"sparksmode")], Some(3));
        let untouched = [b"notaskswitch".as_slice(), b"noaudio", b"agpmode"];
        for key in untouched {
            assert_eq!(
                now[find(key)],
                start[find(key)],
                "{}",
                String::from_utf8_lossy(key)
            );
        }
        for (m, (_, target)) in ENTRIES.iter().enumerate() {
            if let Some(value) = start[m] {
                target.set(o, value);
            }
        }
        assert_eq!(dword_values(o), start);
        assert_eq!(unaligned_values(o), start_unaligned);
    }
}
