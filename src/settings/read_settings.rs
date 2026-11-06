//! `settings::read_settings` (0x4114A6): reads `var\ral.cfg` into the game's settings.
//!
//! In:  nothing
//! Out: every register preserved
//! `read_text` on `var\ral.cfg`; if it reads, `sub_4114FF` (0x4114FF, one caller, here as
//! `apply_settings`): for each of the 77 entries of the table at 0x518C09 (a key, then the
//! address of its variable), if `find_key` finds the key, the variable becomes
//! `parse_number` of its value. A key that is not in the file leaves its variable alone;
//! a key without a number makes it 0. Two keys share a variable twice (`noaudio` and
//! `nosound`, `allsecretcars` and `secretcar`): the later one in the table wins.
//! Callers: `start` and `sub_411239`.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::race::owner::race;
use crate::session::{Dword, Loose};
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;

// ---- game data ----

/// Where a setting's variable lives now: its store's cell. The table keeps the variable's
/// IDA address too (the table's bytes are the image's; `tests::targets_are_the_addresses`
/// proves each cell is what `crate::moved::place` answers for that address).
#[derive(Clone, Copy)]
enum Target {
    /// A whole aligned dword of a store (`dword!`).
    Dword(fn() -> &'static Dword),
    /// A dword at an address that is not a multiple of 4 (`loose!`).
    Loose(fn() -> &'static Loose),
    /// A dword that straddles two cells of its store's untyped tail: the store's own
    /// setter writes the four bytes (`race::viewer_screen_store`'s).
    Unaligned(fn(u32)),
}

impl Target {
    /// The four bytes of the variable set.
    fn set(self, number: u32) {
        match self {
            Target::Dword(cell) => cell().set(number),
            Target::Loose(cell) => cell().set(number),
            Target::Unaligned(set) => set(number),
        }
    }
}

/// `dword!(STATIC.field)`: the dword as a `Target` (a `fn` item, so the table stays `const`).
macro_rules! dword {
    ($($path:tt)+) => {
        Target::Dword({
            fn cell() -> &'static Dword {
                &$($path)+
            }
            cell
        })
    };
}

/// `loose!(STATIC.field)`: an unaligned dword as a `Target`.
macro_rules! loose {
    ($($path:tt)+) => {
        Target::Loose({
            fn cell() -> &'static Loose {
                &$($path)+
            }
            cell
        })
    };
}

/// `wrreplayenable`: `dword_5C2CEB` (unaligned, in the viewer panel's tail).
fn set_developer_keys(number: u32) {
    crate::race::viewer_screen_store::VIEWER_PANEL.set_developer_keys(number);
}

/// `wrreplayrate`: `dword_5C2CEF` (unaligned, in the viewer panel's tail).
fn set_picture_rate(number: u32) {
    crate::race::viewer_screen_store::VIEWER_PANEL.set_picture_rate(number);
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
        dword!(crate::session::FLAGS.no_task_switch),
    ),
    (
        b"noprichange",
        0x518388,
        dword!(crate::session::FLAGS.keep_priority),
    ),
    (
        b"ignoremmx",
        0x5183AC,
        dword!(crate::session::MACHINE.ignore_mmx),
    ),
    (
        b"ignorekatmai",
        0x5183B4,
        dword!(crate::session::MACHINE.ignore_katmai),
    ),
    (
        b"ignoreamdk3d",
        0x5183BC,
        dword!(crate::session::MACHINE.ignore_3dnow),
    ),
    (
        b"altmemenable",
        0x4E0584,
        dword!(crate::memory::state::OWN_HEAP),
    ),
    (
        b"altmemheapsize",
        0x4E0588,
        dword!(crate::memory::state::OWN_HEAP_KB),
    ),
    (
        b"noaudio",
        0x4F82B4,
        dword!(crate::audio::store::AUDIO.no_audio),
    ),
    (
        b"nosound",
        0x4F82B4,
        dword!(crate::audio::store::AUDIO.no_audio),
    ),
    (
        b"nocdaudio",
        0x518104,
        dword!(crate::sound::owner::sound().menu.cd_audio.no_cd_audio),
    ),
    (
        b"nomusic",
        0x5069BC,
        dword!(crate::audio::store::AUDIO.no_music),
    ),
    (
        b"audiobasefreq",
        0x4F82D4,
        dword!(crate::audio::store::AUDIO.mix_rate_setting),
    ),
    (
        b"ignoreds3d",
        0x506B42,
        loose!(crate::audio::store::AUDIO.no_listener),
    ),
    (
        b"ignoredsv",
        0x506C50,
        dword!(crate::audio::store::AUDIO.ignore_dsv_setting),
    ),
    (
        b"ignoreeax",
        0x506C6C,
        dword!(crate::audio::store::AUDIO.ignore_eax_setting),
    ),
    (
        b"ignorea3d",
        0x506C5C,
        dword!(crate::audio::store::AUDIO.no_a3d_setting),
    ),
    (
        b"wavheapsize",
        0x5068EC,
        dword!(crate::audio::store::AUDIO.sample_buffer_kb),
    ),
    (
        b"bundleheapsize",
        0x51B980,
        dword!(crate::gfx::glyph_tables::TABLES.units),
    ),
    (
        b"imageheapsize",
        0x519554,
        dword!(crate::strings::compile_lexicon::LEXICON.kilobytes),
    ),
    (
        b"gamegfxheapsize",
        0x5BE894,
        dword!(crate::gfx::build_game_fonts::GAME_FONTS.units),
    ),
    (
        b"forcedx5",
        0x5AB958,
        dword!(crate::d3d::cards_store::CARDS_3D.force_dx5),
    ),
    (
        b"forceno3dcard",
        0x5AB94C,
        dword!(crate::d3d::cards_store::CARDS_3D.no_search),
    ),
    (
        b"ignoreprimary",
        0x5AB950,
        dword!(crate::d3d::cards_store::CARDS_3D.not_primary),
    ),
    (
        b"ignoresecondary",
        0x5AB954,
        dword!(crate::d3d::cards_store::CARDS_3D.ignore_secondary),
    ),
    (
        b"agpmode",
        0x5AB95C,
        dword!(crate::d3d::cards_store::CARDS_3D.agp_mode),
    ),
    (
        b"force3dtrans",
        0x5AB97C,
        dword!(crate::d3d::cards_store::CARDS_3D.force_trans),
    ),
    (
        b"force3dalpha",
        0x5AB984,
        dword!(crate::d3d::cards_store::CARDS_3D.force_alpha),
    ),
    (
        b"force3dshadow",
        0x5AB980,
        dword!(crate::d3d::cards_store::CARDS_3D.force_shadow),
    ),
    (
        b"force3dfogging",
        0x5AB988,
        dword!(crate::d3d::cards_store::CARDS_3D.force_fogging),
    ),
    (
        b"textflashmode",
        0x4F3F74,
        dword!(crate::text::blend_font::BLEND_MODE),
    ),
    (
        b"videomodemain",
        0x4F4944,
        dword!(crate::gfx::owner::gfx().display.mode.menu_mode),
    ),
    (
        b"videomodegame",
        0x4F4948,
        dword!(crate::gfx::owner::gfx().display.mode.race_mode),
    ),
    (
        b"v3driverfix",
        0x624238,
        dword!(crate::scene::draw_store::DRAW.v3_driver_fix),
    ),
    (
        b"nopngdecode",
        0x51B97C,
        dword!(crate::gfx::owner::GFX.png.setting),
    ),
    (
        b"nojoy",
        0x4F68DC,
        dword!(crate::input::store::DEVICES.no_joysticks),
    ),
    (
        b"joymode",
        0x4F68E0,
        dword!(crate::input::store::DEVICES.joy_mode),
    ),
    (
        b"joycode",
        0x4F68E4,
        dword!(crate::input::store::DEVICES.joy_code),
    ),
    (
        b"joylowerdz",
        0x4F68E8,
        dword!(crate::input::store::DEVICES.joy_dead_zone),
    ),
    (
        b"joyupperdz",
        0x4F68EC,
        dword!(crate::input::store::DEVICES.joy_end_zone),
    ),
    (
        b"noforcefeedback",
        0x4F68F4,
        dword!(crate::input::store::DEVICES.no_force_feedback),
    ),
    (
        b"joyffgain",
        0x4F68F8,
        dword!(crate::input::store::DEVICES.ff_gain),
    ),
    (
        b"mergejoy",
        0x4F68FC,
        dword!(crate::input::store::DEVICES.two_devices_one_player),
    ),
    (
        b"nonetwork",
        0x52A4B0,
        dword!(crate::net::store::NET.receive_done),
    ),
    (b"inetport", 0x52AC24, dword!(crate::net::store::NET.port)),
    (
        b"screensnapshot",
        0x4F52B4,
        dword!(crate::gfx::owner::gfx().shots.on),
    ),
    (b"framecounter", 0x5EB100, dword!(race().frame.counter.show)),
    (
        b"noreplay",
        0x5BEC7C,
        dword!(crate::race::replay_file_store::REPLAY_FILE.no_replay_switch),
    ),
    (
        b"replaykeys",
        0x5BECA8,
        dword!(crate::race::replay_file_store::REPLAY_FILE.viewer_editing),
    ),
    (
        b"logfile",
        0x4E0098,
        dword!(crate::logfile::write_line::state::LEVEL),
    ),
    (b"nofmv", 0x5BE870, dword!(crate::film::NO_FMV)),
    (b"key", 0x4E052C, dword!(race().replay.seal_work.key[0])),
    (
        b"allsecretcars",
        0x758710,
        dword!(crate::menu::owner::menus().play.cheats.all_secret_cars),
    ),
    (
        b"secretcar",
        0x758710,
        dword!(crate::menu::owner::menus().play.cheats.all_secret_cars),
    ),
    (
        b"usesavegamekeys",
        0x755144,
        dword!(
            crate::saves::owner::saves()
                .savegame_preview
                .use_savegame_keys
        ),
    ),
    (
        b"sensiblesaving",
        0x75AAD8,
        dword!(crate::menu::owner::menus().screens.state.menus.review_kind),
    ),
    (
        b"noprecipitation",
        0x71C800,
        dword!(crate::race::work_store::RACE_WORK.no_precipitation),
    ),
    (
        b"seasoncode",
        0x5BECAC,
        dword!(crate::race::replay_file_store::REPLAY_FILE.season_code),
    ),
    (
        b"noautoneutral",
        0x61416C,
        dword!(crate::physics::tables_store::PHYSICS.no_auto_neutral),
    ),
    (
        b"nonlinearsteering",
        0x4F6900,
        dword!(crate::input::store::DEVICES.nonlinear_steering),
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
        dword!(crate::input::store::DEVICES.alt_axes_code),
    ),
    (
        b"fizpatch",
        0x71C6D0,
        dword!(crate::race::work_store::RACE_WORK.patch_switch),
    ),
    (
        b"patchmaxdrawdistance",
        0x71C6D4,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[0]),
    ),
    (
        b"patchstickygrass",
        0x71C6D8,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[1]),
    ),
    (
        b"patchgravity",
        0x71C6DC,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[2]),
    ),
    (
        b"patchdamages",
        0x71C6E0,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[3]),
    ),
    (
        b"patchbrakegrip",
        0x71C6E4,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[4]),
    ),
    (
        b"patchaccelgrip",
        0x71C6E8,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[5]),
    ),
    (
        b"patchslidegrip",
        0x71C6EC,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[6]),
    ),
    (
        b"patchgripgrass",
        0x71C6F0,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[7]),
    ),
    (
        b"patchgriptarmac",
        0x71C6F4,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[8]),
    ),
    (
        b"patchgripsnow",
        0x71C6F8,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[9]),
    ),
    (
        b"patchgripmud",
        0x71C6FC,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[10]),
    ),
    (
        b"patchgripgravel",
        0x71C700,
        dword!(crate::race::work_store::RACE_WORK.patch_settings[11]),
    ),
    (
        b"lexcontrol",
        0x4E0758,
        dword!(crate::strings::maybe_compile_lexicon::RECOMPILE_LEVEL),
    ),
    (
        b"sparksmode",
        0x7558DC,
        dword!(crate::menu::owner::menus().screens.event.sparks_mode),
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

// ---- Rust version ----

/// `sub_4114FF`: the settings a text names. Converted: `rc2k_core::files::settings::table` and
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

pub fn apply_settings(text: GamePtr) -> Result<(), BadInput> {
    if text.is_null() {
        return Err(BadInput("no settings text or no key"));
    }
    let table = settings()?;
    let keys: Vec<&[u8]> = table.iter().map(|(key, _)| key.as_slice()).collect();
    // Found first, stored after: nothing is written if a key is refused.
    let found =
        rc2k_core::files::settings::apply(&text.bytes("a settings text")?, &keys, &case_table())?;
    for (n, number) in found {
        ENTRIES[n].2.set(number);
    }
    Ok(())
}

pub fn read_settings() -> Result<(), BadInput> {
    match read_text(&PATH[..PATH.len() - 1])? {
        Some((text, _)) => apply_settings(text),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::{ENTRIES, Target};
    use crate::moved::image_bytes;

    #[test]
    fn targets_are_the_addresses() {
        for (key, ida, target) in ENTRIES {
            let name = String::from_utf8_lossy(key);
            let here = crate::moved::place(ida as usize) as u32;
            let cell = match target {
                Target::Dword(cell) => cell().addr(),
                Target::Loose(cell) => crate::session::address_of(cell()),
                // Their cells are bytes of a tail: checked by writing, in the test below.
                Target::Unaligned(_) => continue,
            };
            assert_eq!(cell, here, "{name} at {ida:#X}");
        }
    }

    /// The two unaligned settings write the four bytes at their address (and only those).
    #[test]
    fn unaligned_targets_write_their_bytes() {
        use crate::pointers::GamePtr;
        let _lock = crate::store::test_lock();
        for (key, ida, target) in ENTRIES {
            if !matches!(target, Target::Unaligned(_)) {
                continue;
            }
            let at = GamePtr::new(crate::moved::place(ida as usize) as u32);
            let around = GamePtr::new(crate::moved::place(ida as usize - 4) as u32);
            let before: u32 = at.read("the variable").unwrap();
            let neighbours: Vec<u8> = around.read_bytes(12, "its neighbours").unwrap();
            target.set(0x0A0B_0C0D);
            let after: u32 = at.read("the variable").unwrap();
            let neighbours_after: Vec<u8> = around.read_bytes(12, "its neighbours").unwrap();
            target.set(before);
            assert_eq!(after, 0x0A0B_0C0D, "{}", String::from_utf8_lossy(key));
            assert_eq!(&neighbours[..4], &neighbours_after[..4]);
            assert_eq!(&neighbours[8..], &neighbours_after[8..]);
        }
    }

    #[test]
    fn table_is_the_games() {
        let bytes = super::table_bytes();
        assert_eq!(bytes, image_bytes(0x518C09, bytes.len()));
        assert_eq!(bytes.len(), 0x5190F1 - 0x518C09);
        assert_eq!(&image_bytes(0x5295CC, 12)[..], &super::PATH[..]);
        // The empty key that ends the table.
        assert_eq!(image_bytes(0x5190F1, 1), [0]);
    }

    #[test]
    fn table_reads_whole() {
        let table = { super::settings() }.unwrap();
        assert_eq!(table.len(), super::ENTRIES.len());
    }
}
