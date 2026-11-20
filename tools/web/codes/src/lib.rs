//! The save code for the web build's saves page: the player's state read out of the files the
//! page gathers from the browser's storage, made into the code's parts and QR modules, and a
//! code read back into those files. The code is the game's own (`rc2k_core::saves::code`), so
//! the QR codes this makes are the ones the game's save code page shows and takes in.
//!
//! The files are named as the page names them: lower case, `/` between folders
//! (`savedata/savegame/svgame03`, `savedata/carsetup/carset00`, `var/hst.dat`, `var/rc2k.toml`).
//! The saved games and set-ups are read from their files, the best times from `var/hst.dat`
//! (the six tables the game keeps there), and the rally times and stage records from the
//! options in `var/rc2k.toml`, as the game reads them. The settings are not part of a code.

mod ffi;
pub mod files;

use files::Files;
use rc2k_core::files::settings;
use rc2k_core::options::{Options, toml_file};
use rc2k_core::saves::code::best::{self, DriverNames};
use rc2k_core::saves::code::parts::{Assembly, Density, Part, Taken};
use rc2k_core::saves::code::{
    BestTimes, GAME_STATE_LEN, Game, NAME_LEN, PlayerState, SETUP_VALUES, SLOTS, Setup, Summary, qr,
};
use rc2k_core::saves::{kept, tables};
use rc2k_core::strings::case::CaseTable;
use rc2k_core::strings::lexicon::{self, compile};
use rc2k_core::strings::table::StringTable;

const HST: &str = "var/hst.dat";
/// The game's language file (`files/<locale>/*.txt`, the first), which the page adds under this
/// name: the 180 drivers' names are in it, and the made-up best times carry them.
pub const LEXICON: &str = ":lexicon";
/// The file that renames drivers.
const NAME_CFG: &str = "var/name.cfg";
/// The lexicon compiler's memory, as the game gives it.
const LEXICON_MEMORY: u32 = 0x400 * 0x400;
const DRIVERS: u32 = 180;
/// The first of the drivers' name strings.
const FIRST_NAME_ID: u32 = 0x5A5;
const OPTIONS: &str = "var/rc2k.toml";
/// The tables of `var/hst.dat` the code carries, by their place among the game's nine.
const FILE_TABLES: [usize; 6] = [1, 2, 3, 4, 7, 8];
const GAME_FILE_LEN: usize = 0x8000;
const GAME_HEAD_LEN: usize = 0x30;

/// The names of the game's 180 drivers: the language file's, and those `var/name.cfg` gives
/// instead. `None` without the language file (the best times cannot then tell the game's made-up
/// entries from the player's).
fn driver_names(files: &Files) -> Result<Option<DriverNames>, String> {
    let Some(text) = files.get(LEXICON) else {
        return Ok(None);
    };
    let bad = |e: rc2k_core::Invalid| format!("the game's language file: {}", e.0);
    let mut table = compile(text, LEXICON_MEMORY).map_err(bad)?.table;
    lexicon::correct(&mut table).map_err(bad)?;
    let strings = StringTable::new(&table);
    let mut names = Vec::new();
    for n in 0..DRIVERS {
        names.push(strings.text(FIRST_NAME_ID + n).map_err(bad)?);
    }
    if let Some(cfg) = files.get(NAME_CFG) {
        let case = CaseTable::for_language(&strings.text(2).map_err(bad)?).map_err(bad)?;
        let renamed = settings::read_driver_names(cfg, &case).map_err(bad)?;
        for (name, entry) in names.iter_mut().zip(renamed.entries) {
            if let Some(new) = entry.and_then(|e| e.name) {
                *name = new;
            }
        }
    }
    Ok(Some(DriverNames(names)))
}

fn slot_path(folder: &str, name: &str, slot: usize) -> String {
    format!("savedata/{folder}/{name}{slot:02}")
}

/// The options in `var/rc2k.toml` (the game's own for what it lacks); the default without the file.
fn options_of(files: &Files) -> Result<Options, String> {
    let base = Options::default();
    let Some(bytes) = files.get(OPTIONS) else {
        return Ok(base);
    };
    let text = std::str::from_utf8(bytes).map_err(|_| "var/rc2k.toml is not text".to_owned())?;
    match toml_file::read(text, &base) {
        Ok(Some(read)) => Ok(read.options),
        Ok(None) => Ok(base),
        Err(e) => Err(format!("var/rc2k.toml: {e:?}")),
    }
}

/// The six best-times tables out of the bytes of `var/hst.dat`, sized for a championship of the
/// options' record table; `None` when the file is not there or is not a best-times file of this
/// size.
fn file_tables(hst: &[u8], options: &Options) -> Option<[Vec<u8>; 6]> {
    if !kept::head_ok(hst, kept::BEST_TIMES) {
        return None;
    }
    let championship = &options.records.championship.0;
    let tag = |n: u32| {
        let at = n as usize * best::RECORD_LEN;
        championship.get(at..at + 4).map_or(u32::MAX, |b| {
            u32::from_le_bytes(b.try_into().expect("4 bytes"))
        })
    };
    let sizes = tables::sizes(tables::count_stages(tag).ok()?);
    let mut at = kept::HEAD_LEN;
    let mut out: [Vec<u8>; 6] = Default::default();
    for (table, n) in out.iter_mut().zip(FILE_TABLES) {
        let len = sizes[n] as usize;
        *table = hst.get(at..at + len)?.to_vec();
        at += len;
    }
    Some(out)
}

fn hst_file(time: u32, tables: &[Vec<u8>; 6]) -> Vec<u8> {
    let mut file = kept::head(kept::BEST_TIMES, time).to_vec();
    for t in tables {
        file.extend_from_slice(t);
    }
    file
}

/// The car set-ups the files hold, by slot.
fn setups(files: &Files) -> Vec<Setup> {
    let mut out = Vec::new();
    for slot in 0..SLOTS {
        let Some(bytes) = files.get(&slot_path("carsetup", "carset", slot)) else {
            continue;
        };
        let Ok(file) =
            <&[u8; kept::SETUP_LEN]>::try_from(bytes.get(..kept::SETUP_LEN).unwrap_or(&[]))
        else {
            continue;
        };
        let Some(name) = kept::setup_name(file) else {
            continue;
        };
        let mut values = [0u32; SETUP_VALUES];
        for (n, v) in values.iter_mut().enumerate() {
            *v = u32::from_le_bytes(
                file[0x30 + 4 * n..0x34 + 4 * n]
                    .try_into()
                    .expect("4 bytes"),
            );
        }
        out.push(Setup {
            slot: slot as u8,
            name,
            values,
        });
    }
    out
}

/// The saved games the files hold, by slot (a file is a whole 0x8000 bytes, as the game reads it).
fn games(files: &Files) -> Vec<Game> {
    let mut out = Vec::new();
    for slot in 0..SLOTS {
        let Some(file) = files.get(&slot_path("savegame", "svgame", slot)) else {
            continue;
        };
        if file.len() < GAME_FILE_LEN {
            continue;
        }
        let Some(name) = kept::slot_name(file, kept::SAVEGAME, kept::SAVEGAME_VERSION) else {
            continue;
        };
        out.push(Game {
            slot: slot as u8,
            name,
            state: file[GAME_HEAD_LEN..GAME_HEAD_LEN + GAME_STATE_LEN].to_vec(),
        });
    }
    out
}

/// The whole state now: the saved games, the set-ups and, with the best times' file there, the
/// best times. `now` is in seconds from 1970.
pub fn gather(files: &Files, now: u32) -> Result<PlayerState, String> {
    let options = options_of(files)?;
    let names = driver_names(files)?;
    let best = names
        .zip(file_tables(files.get(HST).unwrap_or(&[]), &options))
        .map(|(names, tables)| {
            let rallies: Vec<&[u8]> = options.records.rallies.iter().map(|b| &b.0[..]).collect();
            let events: Vec<&[u8]> = options.records.events.iter().map(|b| &b.0[..]).collect();
            let players: Vec<Vec<u8>> = options.players.names.iter().map(|n| n.0.clone()).collect();
            let mut tyre_name = [0u8; NAME_LEN];
            tyre_name.copy_from_slice(&options.records.tyre_change_holder.0[..NAME_LEN]);
            BestTimes {
                real: best::extract(&tables, &names),
                rallies: best::extract_rallies(&rallies, &names),
                records: best::extract_records(&options.records.championship.0, &events, &players),
                tyre_change: Some((tyre_name, options.records.tyre_change_time)),
            }
        });
    Ok(PlayerState {
        made: now,
        program: format!("web {}", env!("CARGO_PKG_VERSION")),
        games: Some(games(files)),
        setups: Some(setups(files)),
        best,
    })
}

/// The texts the QR codes of the state carry, in order (one, or the numbered parts).
pub fn parts(files: &Files, now: u32, density: Density) -> Result<Vec<String>, String> {
    let whole = gather(files, now)?.encode();
    rc2k_core::saves::code::parts::split(&whole, density)
        .ok_or_else(|| "The saves are too much for a save code.".to_owned())
}

/// The modules of one QR code.
pub fn modules(text: &str) -> Option<qr::Modules> {
    qr::encode(text)
}

/// A QR code read out of a camera frame (RGBA), as the text it carries, if there is one.
pub fn read_frame(width: usize, height: usize, rgba: &[u8]) -> Option<String> {
    let size = width.checked_mul(height)?.checked_mul(4)?;
    if rgba.len() < size {
        return None;
    }
    let grey: Vec<u8> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .take(width * height)
        .map(|p| ((u32::from(p[0]) * 77 + u32::from(p[1]) * 151 + u32::from(p[2]) * 28) >> 8) as u8)
        .collect();
    qr::read_camera(width, height, &grey).map(|(text, _)| text)
}

/// The parts of a code being scanned, and the whole once it is complete.
#[derive(Default)]
pub struct Scan {
    assembly: Assembly,
    single: Option<String>,
}

/// What a text read from a QR code was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Read {
    /// A part not held yet, or a whole code.
    New,
    /// A part already held.
    Again,
    /// A part of another code than the one being collected.
    Other,
    /// Not the game's save code.
    Not,
}

impl Scan {
    pub fn take(&mut self, text: &str) -> Read {
        if let Some(part) = Part::read(text) {
            self.single = None;
            return match self.assembly.take(part) {
                Taken::New => Read::New,
                Taken::Again => Read::Again,
                Taken::Other => Read::Other,
            };
        }
        let text = text.trim();
        if text.starts_with(rc2k_core::saves::code::START) && PlayerState::decode(text).is_ok() {
            self.assembly = Assembly::default();
            self.single = Some(text.to_owned());
            return Read::New;
        }
        Read::Not
    }

    /// (parts held, parts the code has); a code of one part counts 1 of 1.
    pub fn progress(&self) -> (usize, usize) {
        if self.single.is_some() {
            (1, 1)
        } else {
            (self.assembly.numbers().len(), self.assembly.count())
        }
    }

    /// The code's whole text once every part is held (an error when the parts do not fit).
    pub fn whole(&self) -> Option<Result<String, String>> {
        if let Some(text) = &self.single {
            return Some(Ok(text.clone()));
        }
        self.assembly.whole().map(|r| r.map_err(|e| e.to_string()))
    }
}

/// What a code holds, as lines of text for the question before it is taken in.
pub fn describe(whole: &str) -> Result<Vec<String>, String> {
    let state = PlayerState::decode(whole).map_err(|e| e.to_string())?;
    Ok(describe_state(&state.summary()))
}

fn describe_state(s: &Summary) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(games) = &s.games {
        lines.push(format!("{} saved games", games.len()));
    }
    if let Some(n) = s.setups {
        lines.push(format!("{n} car set-ups"));
    }
    if let Some(n) = s.best_times {
        lines.push(format!("{n} best times and records"));
    }
    lines
}

/// The files a code writes: the saved games and set-ups into their slots, the best times
/// merged into the tables of `var/hst.dat` and the rally times and stage records into the
/// options of `var/rc2k.toml`, ranked in as the game ranks a new best time. Nothing else of
/// `files` is changed. The lines say what was done. `packed_time` is the files' stamp.
pub fn import(
    whole: &str,
    files: &Files,
    packed_time: u32,
) -> Result<(Files, Vec<String>), String> {
    let state = PlayerState::decode(whole).map_err(|e| e.to_string())?;
    let mut out = Files::default();
    let mut report = Vec::new();
    if let Some(times) = &state.best {
        let mut options = options_of(files)?;
        let names = driver_names(files)?;
        match names.zip(file_tables(files.get(HST).unwrap_or(&[]), &options)) {
            None => report.push(
                "The best times were not taken in: this browser has no best times file or no language file yet (start the game once)."
                    .to_owned(),
            ),
            Some((names, mut mine)) => {
                best::merge(&mut mine, &times.real, &names).map_err(|e| e.to_string())?;
                let records = &mut options.records;
                let mut rallies: Vec<&mut [u8]> =
                    records.rallies.iter_mut().map(|b| b.0.as_mut_slice()).collect();
                best::merge_rallies(&mut rallies, &times.rallies, &names)
                    .map_err(|e| e.to_string())?;
                let mut events: Vec<&mut [u8]> =
                    records.events.iter_mut().map(|b| b.0.as_mut_slice()).collect();
                best::merge_records(&mut records.championship.0, &mut events, &times.records);
                if let Some((name, time)) = times.tyre_change {
                    records.tyre_change_holder.0 = name.to_vec();
                    records.tyre_change_time = time;
                }
                let existing = files
                    .get(OPTIONS)
                    .and_then(|b| std::str::from_utf8(b).ok());
                let text = toml_file::write(&options, existing).map_err(|e| format!("{e:?}"))?;
                out.set(OPTIONS, text.into_bytes());
                out.set(HST, hst_file(packed_time, &mine));
                report.push(format!("{} best times and records", times.count()));
            }
        }
    }
    if let Some(list) = &state.setups {
        for s in list {
            let file = kept::setup_file(packed_time, &s.name, s.values);
            out.set(
                &slot_path("carsetup", "carset", usize::from(s.slot)),
                file.to_vec(),
            );
        }
        report.push(format!("{} car set-ups", list.len()));
    }
    if let Some(list) = &state.games {
        for g in list {
            let mut file = vec![0u8; GAME_FILE_LEN];
            file[..GAME_HEAD_LEN].copy_from_slice(&kept::slot_head(
                kept::SAVEGAME,
                kept::SAVEGAME_VERSION,
                packed_time,
                &g.name,
            ));
            file[GAME_HEAD_LEN..GAME_HEAD_LEN + g.state.len()].copy_from_slice(&g.state);
            out.set(&slot_path("savegame", "svgame", usize::from(g.slot)), file);
        }
        report.push(format!("{} saved games", list.len()));
    }
    Ok((out, report))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(text: &[u8]) -> [u8; NAME_LEN] {
        let mut n = [0; NAME_LEN];
        n[..text.len()].copy_from_slice(text);
        n
    }

    /// A browser's files with a few saved games and set-ups.
    fn synthetic() -> Files {
        let mut files = Files::default();
        for slot in [0usize, 7, 99] {
            let mut file = vec![0u8; GAME_FILE_LEN];
            file[..GAME_HEAD_LEN].copy_from_slice(&kept::slot_head(
                kept::SAVEGAME,
                kept::SAVEGAME_VERSION,
                5,
                &name(format!("game {slot}").as_bytes()),
            ));
            for (n, b) in file[GAME_HEAD_LEN..GAME_HEAD_LEN + GAME_STATE_LEN]
                .iter_mut()
                .enumerate()
            {
                *b = (n * 7 + slot) as u8;
            }
            files.set(&slot_path("savegame", "svgame", slot), file);
        }
        for slot in [1usize, 2] {
            let file = kept::setup_file(5, &name(b"set"), [1, 2, 3, 4, 5, slot as u32]);
            files.set(&slot_path("carsetup", "carset", slot), file.to_vec());
        }
        files
    }

    /// The parts as QR pictures, read back and collected.
    fn scanned(parts: &[String]) -> Scan {
        let mut scan = Scan::default();
        for part in parts {
            let m = modules(part).expect("a code");
            let (width, grey) = qr::render(&m, 4);
            let rgba: Vec<u8> = grey.iter().flat_map(|&g| [g, g, g, 255]).collect();
            let text = read_frame(width, width, &rgba).expect("read");
            assert_eq!(&text, part);
            assert_eq!(scan.take(&text), Read::New);
        }
        scan
    }

    #[test]
    fn a_state_goes_through_qr_pictures_and_back() {
        let files = synthetic();
        for density in [Density::Small, Density::Dense] {
            let parts = parts(&files, 1234, density).unwrap();
            assert!(parts.len() > 1, "{density:?}");
            let scan = scanned(&parts);
            let Some(Ok(whole)) = scan.whole() else {
                panic!("complete");
            };
            assert_eq!(
                describe(&whole).unwrap(),
                ["3 saved games", "2 car set-ups"]
            );
            let (out, report) = import(&whole, &Files::default(), 9).unwrap();
            assert_eq!(report, ["2 car set-ups", "3 saved games"]);
            for (path, data) in files.iter() {
                let got = out.get(path).unwrap_or_else(|| panic!("{path}"));
                // The files' stamp is the import's.
                assert_eq!(got[..12], data[..12], "{path}");
                assert_eq!(got[16..], data[16..], "{path}");
                assert_eq!(got[12..16], 9u32.to_le_bytes(), "{path}");
            }
        }
    }

    /// The game's own install, when this machine has one: its real state makes a code that
    /// goes through QR pictures and imports into the same files unchanged.
    #[test]
    fn the_games_install_round_trips() {
        let Some(home) = std::env::var_os("HOME") else {
            return;
        };
        let root = std::path::Path::new(&home).join(".local/share/rally-championship");
        if !root.join("var/hst.dat").exists() {
            return;
        }
        let mut files = Files::default();
        let mut walk = vec![root.join("savedata")];
        while let Some(dir) = walk.pop() {
            for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk.push(p);
                } else if let Ok(rel) = p.strip_prefix(&root) {
                    let rel = rel.to_string_lossy().to_lowercase();
                    if rel.contains("/savegame/") || rel.contains("/carsetup/") {
                        files.set(&rel, std::fs::read(&p).unwrap());
                    }
                }
            }
        }
        for f in ["var/hst.dat", "var/rc2k.toml", "var/name.cfg"] {
            if let Ok(b) = std::fs::read(root.join(f)) {
                files.set(f, b);
            }
        }
        let lex = std::fs::read_dir(root.join("files/locale"))
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("txt")))
            .min();
        files.set(LEXICON, std::fs::read(lex.unwrap()).unwrap());
        let state = gather(&files, 1).unwrap();
        let best = state.best.as_ref().expect("best times");
        eprintln!(
            "install: {} saved games, {} set-ups, {} best times, {} rallies, {} records",
            state.games.as_ref().unwrap().len(),
            state.setups.as_ref().unwrap().len(),
            best.real.len(),
            best.rallies.len(),
            best.records.len()
        );
        let parts = parts(&files, 1, Density::Small).unwrap();
        eprintln!("install: {} small parts", parts.len());
        let scan = scanned(&parts);
        let Some(Ok(whole)) = scan.whole() else {
            panic!("complete");
        };
        let (out, report) = import(&whole, &files, 3).unwrap();
        eprintln!("install: {report:?}");
        for (path, data) in out.iter() {
            let was = files.get(path).unwrap_or_else(|| panic!("{path} is new"));
            if path == HST || path.contains("/save") || path.contains("/carset") {
                assert_eq!(was.len(), data.len(), "{path}");
                assert_eq!(was[..12], data[..12], "{path}");
                assert_eq!(was[16..], data[16..], "{path}");
            }
        }
    }

    /// A frame whose size does not fit the address space (a 32-bit module) is read as nothing,
    /// and one with fewer bytes than its size says as well.
    #[test]
    fn a_frame_of_an_impossible_size_reads_nothing() {
        assert_eq!(read_frame(usize::MAX / 2, 3, &[0; 16]), None);
        assert_eq!(read_frame(1 << 20, 1 << 20, &[0; 16]), None);
        assert_eq!(read_frame(8, 8, &[0; 16]), None);
    }
}
