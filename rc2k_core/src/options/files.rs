//! Which options file is read and written when (see the module header of `options`).

use super::{Options, OptionsError, old_file, toml_file};
use std::path::{Path, PathBuf};

/// The two files' places.
#[derive(Clone, Debug)]
pub struct Paths {
    /// `var/rc2k.toml`.
    pub new: PathBuf,
    /// `var/ral.dat`.
    pub old: PathBuf,
}

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
                "--load-old-options" => asked.load_old = true,
                "--save-old-options" => asked.save_old = true,
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

fn read_file(path: &Path) -> Result<Option<Vec<u8>>, OptionsError> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(OptionsError::Io(format!("{}: {e}", path.display()))),
    }
}

/// A file written whole or not at all: to a name beside it, then put in place.
fn write_file(path: &Path, bytes: &[u8]) -> Result<(), OptionsError> {
    let io = |e: std::io::Error| OptionsError::Io(format!("{}: {e}", path.display()));
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    std::fs::write(&temporary, bytes).map_err(io)?;
    std::fs::rename(&temporary, path).map_err(io)
}

/// The options from the old file, if it is there and is one.
pub fn load_old(paths: &Paths) -> Result<Option<Options>, OptionsError> {
    match read_file(&paths.old)? {
        Some(bytes) => old_file::read(&bytes),
        None => Ok(None),
    }
}

/// The options from the new file, the values it lacks taken from `base`.
pub fn load_new(paths: &Paths, base: &Options) -> Result<Option<Options>, OptionsError> {
    match read_file(&paths.new)? {
        Some(bytes) => {
            let text = String::from_utf8(bytes)
                .map_err(|_| OptionsError::Text(format!("{} is not UTF-8", paths.new.display())))?;
            toml_file::read(&text, base).map(Some)
        }
        None => Ok(None),
    }
}

/// Reads the old file by the caller's own way.
pub type OldReader<'a> = &'a mut dyn FnMut() -> Result<Option<Options>, OptionsError>;
/// Writes the old file by the caller's own way.
pub type OldWriter<'a> = &'a mut dyn FnMut(&Options) -> Result<(), OptionsError>;

/// The options to start with: the new file if it is there (unless the old is asked for), else
/// the old file once (the new file is written from it first), else nothing (`base` stays,
/// the game's own values for what a file lacks). `read_old` reads the old file (`load_old` is
/// the plain way). A file from when the game had Aureal's A3D loads as the nearest mode it
/// has now (`Options::without_a3d`).
pub fn load(
    paths: &Paths,
    asked: Asked,
    base: &Options,
    read_old: OldReader,
) -> Result<Option<(Options, Source)>, OptionsError> {
    let mut read_old = || {
        read_old().map(|read| {
            read.map(|mut options| {
                options.without_a3d();
                options
            })
        })
    };
    if asked.load_old {
        return Ok(read_old()?.map(|o| (o, Source::OldAsked)));
    }
    if let Some(mut options) = load_new(paths, base)? {
        options.without_a3d();
        return Ok(Some((options, Source::New)));
    }
    match read_old()? {
        Some(options) => {
            save_new(paths, &options)?;
            Ok(Some((options, Source::OldMigrated)))
        }
        None => Ok(None),
    }
}

/// Writes the new file (keeping the unknown keys it holds).
pub fn save_new(paths: &Paths, options: &Options) -> Result<(), OptionsError> {
    let existing = read_file(&paths.new)?.and_then(|b| String::from_utf8(b).ok());
    let text = toml_file::write(options, existing.as_deref())?;
    write_file(&paths.new, text.as_bytes())
}

/// Writes the old file, `time` the game's packed local time for its head.
pub fn save_old(paths: &Paths, options: &Options, time: u32) -> Result<(), OptionsError> {
    write_file(&paths.old, &old_file::write(options, time))
}

/// Saves as asked: the new file always, the old one too only when asked (`write_old`
/// writes it; `save_old` is the plain way).
pub fn save(
    paths: &Paths,
    asked: Asked,
    options: &Options,
    write_old: OldWriter,
) -> Result<(), OptionsError> {
    save_new(paths, options)?;
    if asked.save_old {
        write_old(options)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dir(PathBuf);

    impl Dir {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("rc2k_options_{name}_{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
        fn paths(&self) -> Paths {
            Paths {
                new: self.0.join("rc2k.toml"),
                old: self.0.join("ral.dat"),
            }
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
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
        let dir = Dir::new("a3d");
        let p = dir.paths();
        let mut saved = options(40);
        saved.sound.race_sound_mode = 0x11_2221;
        saved.pages.audio.0 = vec![1, 0, 0, 1, 0, 1];
        let mut wanted = saved.clone();
        wanted.sound.race_sound_mode = 0x11_1221;
        wanted.pages.audio.0 = vec![1, 1, 0, 0, 0, 1];
        // From the old file (migrated), then from the new file written with the A3D mode.
        save_old(&p, &saved, 9).unwrap();
        let (o, from) = load(&p, Asked::default(), &Options::default(), &mut || {
            load_old(&p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o, from), (wanted.clone(), Source::OldMigrated));
        save_new(&p, &saved).unwrap();
        let (o, from) = load(&p, Asked::default(), &Options::default(), &mut || {
            load_old(&p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o, from), (wanted, Source::New));
    }

    #[test]
    fn nothing_there_loads_nothing() {
        let dir = Dir::new("nothing");
        let got = load(
            &dir.paths(),
            Asked::default(),
            &Options::default(),
            &mut || load_old(&dir.paths()),
        )
        .unwrap();
        assert_eq!(got, None);
        assert!(!dir.paths().new.exists());
    }

    #[test]
    fn the_old_file_alone_is_migrated_once() {
        let dir = Dir::new("migrate");
        let p = dir.paths();
        save_old(&p, &options(40), 9).unwrap();
        let (o, from) = load(&p, Asked::default(), &Options::default(), &mut || {
            load_old(&p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o, from), (options(40), Source::OldMigrated));
        assert!(p.new.exists());
        // The old file is not read again: the new wins even if the old changes.
        save_old(&p, &options(55), 9).unwrap();
        let (o, from) = load(&p, Asked::default(), &Options::default(), &mut || {
            load_old(&p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o, from), (options(40), Source::New));
    }

    #[test]
    fn the_new_file_wins_when_both_exist() {
        let dir = Dir::new("wins");
        let p = dir.paths();
        save_old(&p, &options(1), 0).unwrap();
        save_new(&p, &options(2)).unwrap();
        let (o, from) = load(&p, Asked::default(), &Options::default(), &mut || {
            load_old(&p)
        })
        .unwrap()
        .unwrap();
        assert_eq!((o.sound.master_volume, from), (2, Source::New));
    }

    #[test]
    fn the_old_file_is_read_when_asked() {
        let dir = Dir::new("asked");
        let p = dir.paths();
        save_old(&p, &options(1), 0).unwrap();
        save_new(&p, &options(2)).unwrap();
        let asked = Asked::from_args(["rc2k.exe", "--load-old-options"]);
        let (o, from) = load(&p, asked, &Options::default(), &mut || load_old(&p))
            .unwrap()
            .unwrap();
        assert_eq!((o.sound.master_volume, from), (1, Source::OldAsked));
        // Asked for, with no old file there: nothing, the new file is not used instead.
        std::fs::remove_file(&p.old).unwrap();
        assert_eq!(
            load(&p, asked, &Options::default(), &mut || load_old(&p)).unwrap(),
            None
        );
    }

    #[test]
    fn saving_writes_the_old_file_only_when_asked() {
        let dir = Dir::new("save");
        let p = dir.paths();
        save(&p, Asked::default(), &options(3), &mut |o| {
            save_old(&p, o, 7)
        })
        .unwrap();
        assert!(p.new.exists() && !p.old.exists());
        let asked = Asked::from_args(["--save-old-options"]);
        assert_eq!(
            asked,
            Asked {
                load_old: false,
                save_old: true
            }
        );
        save(&p, asked, &options(4), &mut |o| save_old(&p, o, 7)).unwrap();
        let bytes = std::fs::read(&p.old).unwrap();
        assert_eq!(bytes, old_file::write(&options(4), 7));
        assert_eq!(load_new(&p, &Options::default()).unwrap(), Some(options(4)));
    }

    #[test]
    fn a_broken_new_file_is_an_error_not_a_fallback() {
        let dir = Dir::new("broken");
        let p = dir.paths();
        save_old(&p, &options(1), 0).unwrap();
        std::fs::write(&p.new, "[display]\nmenu_mode = 'x'\n").unwrap();
        assert!(
            load(&p, Asked::default(), &Options::default(), &mut || load_old(
                &p
            ))
            .is_err()
        );
    }
}
