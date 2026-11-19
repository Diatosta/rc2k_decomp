//! Which options file is read and written when (see the module header of `options`).

use super::{Options, OptionsError, old_file, toml_file};
use std::path::{Path, PathBuf};

/// Where the two files are kept: the program's side of reading and writing them.
pub trait Store {
    /// The file's bytes; `None` when there is no such file.
    fn read(&self, path: &Path) -> std::io::Result<Option<Vec<u8>>>;
    /// The file written whole or not at all (a reader never sees half of it).
    fn write_whole(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>;
}

/// The two files' places.
#[derive(Clone, Debug)]
pub struct Paths {
    /// `var/rc2k.toml`.
    pub new: PathBuf,
    /// `var/ral.dat`.
    pub old: PathBuf,
}

/// The switches `Asked::from_args` reads.
pub const SWITCHES: [&str; 2] = ["--load-old-options", "--save-old-options"];

/// What the player asked for on the command line (or with the settings page's button).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Asked {
    /// `--load-old-options`: read the old file whatever the new one holds.
    pub load_old: bool,
    /// `--save-old-options`: write the old file too.
    pub save_old: bool,
}

impl Asked {
    /// The switches among a program's arguments.
    pub fn from_args<S: AsRef<str>>(args: impl IntoIterator<Item = S>) -> Self {
        let mut asked = Self::default();
        for arg in args {
            match arg.as_ref() {
                a if a == SWITCHES[0] => asked.load_old = true,
                a if a == SWITCHES[1] => asked.save_old = true,
                _ => {}
            }
        }
        asked
    }
}

/// Where loaded options came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// The new file.
    New,
    /// The old file, asked for.
    OldAsked,
    /// The old file, because the new one was not there yet; the new file is written from it.
    OldMigrated,
}

fn read_file(store: &dyn Store, path: &Path) -> Result<Option<Vec<u8>>, OptionsError> {
    store
        .read(path)
        .map_err(|e| OptionsError::Io(format!("{}: {e}", path.display())))
}

fn write_file(store: &dyn Store, path: &Path, bytes: &[u8]) -> Result<(), OptionsError> {
    store
        .write_whole(path, bytes)
        .map_err(|e| OptionsError::Io(format!("{}: {e}", path.display())))
}

/// The options from the old file, if it is there and is one.
pub fn load_old(store: &dyn Store, paths: &Paths) -> Result<Option<Options>, OptionsError> {
    match read_file(store, &paths.old)? {
        Some(bytes) => old_file::read(&bytes),
        None => Ok(None),
    }
}

/// The options from the new file, the values it lacks taken from `base`; `None` when it is not
/// there or is of an older version.
pub fn load_new(
    store: &dyn Store,
    paths: &Paths,
    base: &Options,
) -> Result<Option<toml_file::Read>, OptionsError> {
    match read_file(store, &paths.new)? {
        Some(bytes) => {
            let text = String::from_utf8(bytes)
                .map_err(|_| OptionsError::Text(format!("{} is not UTF-8", paths.new.display())))?;
            toml_file::read(&text, base)
        }
        None => Ok(None),
    }
}

/// Reads the old file by the caller's own way.
pub type OldReader<'a> = &'a mut dyn FnMut() -> Result<Option<Options>, OptionsError>;
/// Writes the old file by the caller's own way.
pub type OldWriter<'a> = &'a mut dyn FnMut(&Options) -> Result<(), OptionsError>;

/// The options `load` found.
#[derive(Clone, Debug, PartialEq)]
pub struct Loaded {
    /// The options to start with.
    pub options: Options,
    /// Which file they came from.
    pub source: Source,
    /// The keys of the new file whose values were bad and are at their defaults.
    pub skipped: Vec<String>,
}

/// The options to start with: the new file if it is there (unless the old is asked for), else
/// the old file once (the new file is written from it first), else nothing (`base` stays,
/// the game's own values for what a file lacks). `read_old` reads the old file (`load_old` is
/// the plain way). A file from when the game had Aureal's A3D loads as the nearest mode it
/// has now (`Options::without_a3d`).
pub fn load(
    store: &dyn Store,
    paths: &Paths,
    asked: Asked,
    base: &Options,
    read_old: OldReader,
) -> Result<Option<Loaded>, OptionsError> {
    let mut read_old = || {
        read_old().map(|read| {
            read.map(|mut options| {
                options.without_a3d();
                options
            })
        })
    };
    if asked.load_old {
        return Ok(read_old()?.map(|options| Loaded {
            options,
            source: Source::OldAsked,
            skipped: Vec::new(),
        }));
    }
    if let Some(read) = load_new(store, paths, base)? {
        let mut options = read.options;
        options.without_a3d();
        return Ok(Some(Loaded {
            options,
            source: Source::New,
            skipped: read.skipped,
        }));
    }
    match read_old()? {
        Some(options) => {
            save_new(store, paths, &options)?;
            Ok(Some(Loaded {
                options,
                source: Source::OldMigrated,
                skipped: Vec::new(),
            }))
        }
        None => Ok(None),
    }
}

/// Writes the new file (keeping the unknown keys it holds).
pub fn save_new(store: &dyn Store, paths: &Paths, options: &Options) -> Result<(), OptionsError> {
    let existing = read_file(store, &paths.new)?.and_then(|b| String::from_utf8(b).ok());
    let text = toml_file::write(options, existing.as_deref())?;
    write_file(store, &paths.new, text.as_bytes())
}

/// Writes the old file, `time` the game's packed local time for its head.
pub fn save_old(
    store: &dyn Store,
    paths: &Paths,
    options: &Options,
    time: u32,
) -> Result<(), OptionsError> {
    write_file(store, &paths.old, &old_file::write(options, time))
}

/// Saves as asked: the new file always, the old one too only when asked (`write_old`
/// writes it; `save_old` is the plain way).
pub fn save(
    store: &dyn Store,
    paths: &Paths,
    asked: Asked,
    options: &Options,
    write_old: OldWriter,
) -> Result<(), OptionsError> {
    save_new(store, paths, options)?;
    if asked.save_old {
        write_old(options)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// The two files in memory.
    #[derive(Default)]
    struct Memory(RefCell<HashMap<PathBuf, Vec<u8>>>);

    impl Store for Memory {
        fn read(&self, path: &Path) -> std::io::Result<Option<Vec<u8>>> {
            Ok(self.0.borrow().get(path).cloned())
        }
        fn write_whole(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
            self.0.borrow_mut().insert(path.to_owned(), bytes.to_vec());
            Ok(())
        }
    }

    impl Memory {
        fn paths(&self) -> Paths {
            Paths {
                new: "var/rc2k.toml".into(),
                old: "var/ral.dat".into(),
            }
        }
        fn put(&self, path: &Path, text: &str) {
            self.0
                .borrow_mut()
                .insert(path.to_owned(), text.as_bytes().to_vec());
        }
        fn has(&self, path: &Path) -> bool {
            self.0.borrow().contains_key(path)
        }
        fn get(&self, path: &Path) -> Vec<u8> {
            self.0.borrow()[path].clone()
        }
        fn remove(&self, path: &Path) {
            self.0.borrow_mut().remove(path);
        }
    }

    fn options(volume: u32) -> Options {
        let mut o = Options::default();
        o.sound.master_volume = volume;
        o.players.names[1].0[0] = b'Q';
        o
    }

    #[test]
    fn an_a3d_mode_loads_as_directsound_3d() {
        let dir = Memory::default();
        let p = dir.paths();
        let mut saved = options(40);
        saved.sound.race_sound_mode = 0x11_2221;
        saved.pages.audio.0 = vec![1, 0, 0, 1, 0, 1];
        let mut wanted = saved.clone();
        wanted.sound.race_sound_mode = 0x11_1221;
        wanted.pages.audio.0 = vec![1, 1, 0, 0, 0, 1];
        // From the old file (migrated), then from the new file written with the A3D mode.
        save_old(&dir, &p, &saved, 9).unwrap();
        let Loaded {
            options: o,
            source: from,
            ..
        } = load(&dir, &p, Asked::default(), &Options::default(), &mut || {
            load_old(&dir, &p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o, from), (wanted.clone(), Source::OldMigrated));
        save_new(&dir, &p, &saved).unwrap();
        let Loaded {
            options: o,
            source: from,
            ..
        } = load(&dir, &p, Asked::default(), &Options::default(), &mut || {
            load_old(&dir, &p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o, from), (wanted, Source::New));
    }

    #[test]
    fn nothing_there_loads_nothing() {
        let dir = Memory::default();
        let got = load(
            &dir,
            &dir.paths(),
            Asked::default(),
            &Options::default(),
            &mut || load_old(&dir, &dir.paths()),
        )
        .unwrap();
        assert_eq!(got, None);
        assert!(!dir.has(&dir.paths().new));
    }

    #[test]
    fn the_old_file_alone_is_migrated_once() {
        let dir = Memory::default();
        let p = dir.paths();
        save_old(&dir, &p, &options(40), 9).unwrap();
        let Loaded {
            options: o,
            source: from,
            ..
        } = load(&dir, &p, Asked::default(), &Options::default(), &mut || {
            load_old(&dir, &p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o, from), (options(40), Source::OldMigrated));
        assert!(dir.has(&p.new));
        // The old file is not read again: the new wins even if the old changes.
        save_old(&dir, &p, &options(55), 9).unwrap();
        let Loaded {
            options: o,
            source: from,
            ..
        } = load(&dir, &p, Asked::default(), &Options::default(), &mut || {
            load_old(&dir, &p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o, from), (options(40), Source::New));
    }

    #[test]
    fn a_new_file_of_the_first_version_is_passed_over() {
        let dir = Memory::default();
        let p = dir.paths();
        dir.put(&p.new, "version = 1\n[sound]\nmaster_volume = 99\n");
        // With no old file: the defaults, the file left as it is.
        let none = load(&dir, &p, Asked::default(), &Options::default(), &mut || {
            load_old(&dir, &p)
        })
        .unwrap();
        assert_eq!(none, None);
        // With an old file: its values, brought over into a new file of this version.
        save_old(&dir, &p, &options(40), 9).unwrap();
        let Loaded {
            options: o,
            source: from,
            ..
        } = load(&dir, &p, Asked::default(), &Options::default(), &mut || {
            load_old(&dir, &p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o, from), (options(40), Source::OldMigrated));
        let text = String::from_utf8(dir.get(&p.new)).unwrap();
        assert!(text.starts_with("version = 2\n") && !text.contains("99"));
    }

    #[test]
    fn the_new_file_wins_when_both_exist() {
        let dir = Memory::default();
        let p = dir.paths();
        save_old(&dir, &p, &options(1), 0).unwrap();
        save_new(&dir, &p, &options(2)).unwrap();
        let Loaded {
            options: o,
            source: from,
            ..
        } = load(&dir, &p, Asked::default(), &Options::default(), &mut || {
            load_old(&dir, &p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o.sound.master_volume, from), (2, Source::New));
    }

    #[test]
    fn the_old_file_is_read_when_asked() {
        let dir = Memory::default();
        let p = dir.paths();
        save_old(&dir, &p, &options(1), 0).unwrap();
        save_new(&dir, &p, &options(2)).unwrap();
        let asked = Asked::from_args(["rc2k.exe", "--load-old-options"]);
        let Loaded {
            options: o,
            source: from,
            ..
        } = load(&dir, &p, asked, &Options::default(), &mut || {
            load_old(&dir, &p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o.sound.master_volume, from), (1, Source::OldAsked));
        // Asked for, with no old file there: nothing, the new file is not used instead.
        dir.remove(&p.old);
        assert_eq!(
            load(&dir, &p, asked, &Options::default(), &mut || load_old(
                &dir, &p
            ))
            .unwrap(),
            None
        );
    }

    #[test]
    fn saving_writes_the_old_file_only_when_asked() {
        let dir = Memory::default();
        let p = dir.paths();
        save(&dir, &p, Asked::default(), &options(3), &mut |o| {
            save_old(&dir, &p, o, 7)
        })
        .unwrap();
        assert!(dir.has(&p.new) && !dir.has(&p.old));
        let asked = Asked::from_args(["--save-old-options"]);
        assert_eq!(
            asked,
            Asked {
                load_old: false,
                save_old: true
            }
        );
        save(&dir, &p, asked, &options(4), &mut |o| {
            save_old(&dir, &p, o, 7)
        })
        .unwrap();
        let bytes = dir.get(&p.old);
        assert_eq!(bytes, old_file::write(&options(4), 7));
        assert_eq!(
            load_new(&dir, &p, &Options::default())
                .unwrap()
                .map(|read| read.options),
            Some(options(4))
        );
    }

    #[test]
    fn a_broken_new_file_is_an_error_not_a_fallback() {
        let dir = Memory::default();
        let p = dir.paths();
        save_old(&dir, &p, &options(1), 0).unwrap();
        dir.put(&p.new, "[display\nmenu_mode = 'x'\n");
        assert!(
            load(&dir, &p, Asked::default(), &Options::default(), &mut || {
                load_old(&dir, &p)
            })
            .is_err()
        );
        // A bad value is reset to its default; the old file is still not the fallback.
        dir.put(&p.new, "[display]\nmenu_mode = 'x'\n");
        let Loaded {
            options: o,
            source: from,
            ..
        } = load(&dir, &p, Asked::default(), &Options::default(), &mut || {
            load_old(&dir, &p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o, from), (Options::default(), Source::New));
    }

    #[test]
    fn the_keys_reset_in_the_new_file_come_back_with_the_options() {
        let dir = Memory::default();
        let p = dir.paths();
        dir.put(&p.new, "[display]\nmenu_mode = 'x'\n");
        let loaded = load(&dir, &p, Asked::default(), &Options::default(), &mut || {
            load_old(&dir, &p)
        })
        .unwrap()
        .unwrap();
        assert_eq!(loaded.skipped, ["display.menu_mode"]);
        assert_eq!(loaded.source, Source::New);
    }
}
